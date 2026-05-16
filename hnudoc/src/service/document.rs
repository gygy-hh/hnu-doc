// 上传、PoW 下载

use serde::Serialize;
use serde_json::json;

use crate::{
    config::CFG,
    infra,
    infra::mysql::document::{DocumentRow, NewDocument},
    result::{AppError, AppResult, ErrCode},
    utils,
};

const POW_TICKET_PREFIX: &str = "hnudoc:pow:ticket:";
const DOWNLOAD_TOKEN_PREFIX: &str = "hnudoc:download:token:";

// download 接口：ticket + zero_bits
#[derive(Debug, Serialize)]
pub struct PowChallenge {
    pub ticket: String,
    pub zero: u32,
}

// 写入 pending（重复 md5 → FILE_EXISTED）
#[allow(clippy::too_many_arguments)]
pub async fn save_pending_or_doc(
    bytes: &[u8],
    extension: &str,
    name: &str,
    typ: &str,
    date_typ: Option<&str>,
    date_year: Option<i32>,
    answer: bool,
    tags: &[String],
    comment: Option<&str>,
    categories: &[String],
    stu_id: &str,
) -> AppResult<u32> {
    if bytes.len() > CFG.server.max_upload_size {
        return Err(AppError::biz(
            ErrCode::FileSizeLimitExceeded,
            format!(
                "文件大小超出限制 ({} bytes)",
                CFG.server.max_upload_size
            ),
        ));
    }

    let md5 = infra::storage::md5_hex(bytes);

    // 已经在正式库中存在 -> FILE_EXISTED
    if let Some(existing) =
        infra::mysql::document::get_by_md5(&md5).await?
    {
        let dto = existing.into_dto();
        return Err(AppError::biz_with_data(
            ErrCode::FileExisted,
            "文件已经存在",
            json!(dto),
        ));
    }

    let page = utils::pdf::page_count_bytes(bytes).unwrap_or(0);
    let file_path =
        infra::storage::save_bytes(bytes, &md5, extension).await?;

    // 写入待审核表
    let id = infra::mysql::pending::insert(
        &infra::mysql::pending::NewPendingDocument {
            name,
            typ,
            date_typ,
            date_year,
            answer,
            page,
            tags,
            comment,
            md5: &md5,
            categories,
            file_path: &file_path,
            stu_id,
        },
    )
    .await?;
    Ok(id)
}

// 管理员更新 pending 附件 → (md5, page, path)
#[allow(clippy::too_many_arguments)]
pub async fn build_pending_update(
    new_bytes: Option<&[u8]>,
    new_extension: Option<&str>,
    md5_fallback: &str,
    file_path_fallback: &str,
    page_fallback: u32,
) -> AppResult<(String, u32, String)> {
    if let (Some(bytes), Some(ext)) = (new_bytes, new_extension) {
        if bytes.len() > CFG.server.max_upload_size {
            return Err(AppError::biz(
                ErrCode::FileSizeLimitExceeded,
                "文件大小超出限制",
            ));
        }
        let new_md5 = infra::storage::md5_hex(bytes);

        // 改完之后如果与正式库重复 -> FILE_EXISTED
        if new_md5 != md5_fallback
            && let Some(existing) =
                infra::mysql::document::get_by_md5(&new_md5).await?
        {
            let dto = existing.into_dto();
            return Err(AppError::biz_with_data(
                ErrCode::FileExisted,
                "文件已经存在",
                json!(dto),
            ));
        }

        let page =
            utils::pdf::page_count_bytes(bytes).unwrap_or(0);
        let file_path =
            infra::storage::save_bytes(bytes, &new_md5, ext).await?;
        Ok((new_md5, page, file_path))
    } else {
        Ok((
            md5_fallback.to_string(),
            page_fallback,
            file_path_fallback.to_string(),
        ))
    }
}

// pending → documents
pub async fn promote_pending(
    p: &infra::mysql::pending::PendingDocumentRow,
) -> AppResult<u32> {
    // 双重保险：可能 review 时被别人提前发布了
    if let Some(existing) =
        infra::mysql::document::get_by_md5(&p.md5).await?
    {
        return Ok(existing.id);
    }
    let id = infra::mysql::document::insert(&NewDocument {
        name: &p.name,
        typ: &p.typ,
        date_typ: p.date_typ.as_deref(),
        date_year: p.date_year,
        answer: p.answer,
        page: p.page,
        tags: &p.tags,
        comment: p.comment.as_deref(),
        md5: &p.md5,
        categories: &p.categories,
        file_path: &p.file_path,
    })
    .await?;
    Ok(id)
}

// PoW：ticket→redis→校验后发一次性下载 URL
pub async fn create_pow(doc_id: u32) -> AppResult<PowChallenge> {
    // 先确认 doc 存在
    if infra::mysql::document::get_by_id(doc_id).await?.is_none() {
        return Err(AppError::biz(
            ErrCode::NotFound,
            "试卷不存在",
        ));
    }
    let ticket = utils::pow::gen_ticket();
    infra::redis::set_with_expire(
        &format!("{POW_TICKET_PREFIX}{ticket}"),
        &doc_id.to_string(),
        CFG.pow.ticket_ttl,
    )
    .await?;
    Ok(PowChallenge {
        ticket,
        zero: CFG.pow.zero_bits,
    })
}

// PoW 通过后签发下载 token
pub async fn consume_pow_and_issue_download(
    ticket: &str,
    key: &str,
) -> AppResult<String> {
    let redis_key = format!("{POW_TICKET_PREFIX}{ticket}");
    let Some(doc_id_str) = infra::redis::get(&redis_key).await? else {
        return Err(AppError::biz(
            ErrCode::PowKeyInvalid,
            "ticket 不存在或已过期",
        ));
    };

    if !utils::pow::verify(key, ticket, CFG.pow.zero_bits) {
        // 失败时把 ticket 删掉，前端需要重新 POST 拿新的
        let _ = infra::redis::del(&redis_key).await;
        return Err(AppError::biz(
            ErrCode::PowKeyInvalid,
            "PoW key 校验失败",
        ));
    }

    let _ = infra::redis::del(&redis_key).await;

    let doc_id: u32 = doc_id_str.parse().map_err(|_| {
        AppError::AnyHow(anyhow::anyhow!("ticket 关联数据损坏"))
    })?;

    let token: String = uuid::Uuid::new_v4().simple().to_string();
    infra::redis::set_with_expire(
        &format!("{DOWNLOAD_TOKEN_PREFIX}{token}"),
        &doc_id.to_string(),
        CFG.pow.download_ttl,
    )
    .await?;
    Ok(format!("/document/file/{token}"))
}

// 消费 download token（一次性）
pub async fn consume_download_token(
    token: &str,
) -> AppResult<DocumentRow> {
    let key = format!("{DOWNLOAD_TOKEN_PREFIX}{token}");
    let Some(doc_id_str) = infra::redis::get(&key).await? else {
        return Err(AppError::biz(
            ErrCode::NotFound,
            "下载链接不存在或已过期",
        ));
    };
    let _ = infra::redis::del(&key).await;
    let doc_id: u32 = doc_id_str.parse().map_err(|_| {
        AppError::AnyHow(anyhow::anyhow!("download token 数据损坏"))
    })?;
    let row = infra::mysql::document::get_by_id(doc_id)
        .await?
        .ok_or_else(|| {
            AppError::biz(ErrCode::NotFound, "试卷不存在")
        })?;
    Ok(row)
}
