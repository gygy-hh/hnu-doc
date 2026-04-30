//! 待审核试卷业务

use serde::Serialize;

use crate::{
    infra,
    infra::mysql::document::{Document, build_date},
    result::{AppError, AppResult, ErrCode},
};

/// 与前端一致的 PendingDocument 响应结构
#[derive(Debug, Serialize)]
pub struct PendingDocumentDto {
    pub id: u32,
    pub item: Document,
    pub status: String,
    pub stu_id: String,
    pub comment: Option<String>,
    pub create_time: String,
    pub update_time: String,
    pub target: Option<u32>,
}

fn fmt_dt(t: chrono::NaiveDateTime) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn to_dto(
    p: infra::mysql::pending::PendingDocumentRow,
) -> PendingDocumentDto {
    let item = Document {
        // pending 中的 id 不是正式库 id，这里返回 target（如果已批准）
        // 否则给 0 表示尚未在正式库中存在
        id: p.target.unwrap_or(0),
        date: build_date(p.date_typ.clone(), p.date_year),
        typ: p.typ.clone(),
        name: p.name.clone(),
        answer: p.answer,
        page: p.page,
        tags: p.tags.clone(),
        comment: p.comment.clone(),
        md5: p.md5.clone(),
        categories: p.categories.clone(),
    };
    PendingDocumentDto {
        id: p.id,
        item,
        status: p.status,
        stu_id: p.stu_id,
        comment: p.audit_comment,
        create_time: fmt_dt(p.create_time),
        update_time: fmt_dt(p.update_time),
        target: p.target,
    }
}

pub async fn list_by_status(
    status: &str,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<PendingDocumentDto>> {
    let rows = infra::mysql::pending::list_by_status(
        status, page, page_size,
    )
    .await?;
    Ok(rows.into_iter().map(to_dto).collect())
}

pub async fn get_by_id(
    id: u32,
) -> AppResult<infra::mysql::pending::PendingDocumentRow> {
    infra::mysql::pending::get_by_id(id).await?.ok_or_else(|| {
        AppError::biz(ErrCode::NotFound, "记录不存在")
    })
}

/// 评审：accepted -> 把记录复制到 documents、设置 target；rejected -> 仅写状态
pub async fn review(
    id: u32,
    new_status: &str,
    audit_comment: Option<&str>,
) -> AppResult<()> {
    let p = get_by_id(id).await?;
    if p.status != "pending" {
        return Err(AppError::biz(
            ErrCode::BadRequest,
            "只能审批 pending 状态的记录",
        ));
    }
    match new_status {
        "accepted" => {
            let doc_id =
                crate::service::document::promote_pending(&p)
                    .await?;
            infra::mysql::pending::set_status(
                p.id,
                "accepted",
                None,
                Some(doc_id),
            )
            .await?;
        }
        "rejected" => {
            infra::mysql::pending::set_status(
                p.id,
                "rejected",
                audit_comment,
                None,
            )
            .await?;
        }
        _ => {
            return Err(AppError::biz(
                ErrCode::BadRequest,
                "无效的 status，仅支持 accepted/rejected",
            ));
        }
    }
    Ok(())
}
