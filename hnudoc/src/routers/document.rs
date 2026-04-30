//! `/document` 路由：上传 / 下载 / 列表 / 审核

use std::path::{Path, PathBuf};

use salvo::{
    Request, Response, Router, handler, macros::Extractible,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::fs;

use crate::{
    config::CFG,
    result::{AppError, ErrCode, RouterResult},
    service,
    utils::{self, serde::empty_string_as_none},
};

pub fn routers() -> Router {
    Router::with_path("document")
        // POST /document
        .post(upload_doc)
        .push(
            Router::with_path("download")
                .post(create_pow)
                .get(get_download_link),
        )
        .push(
            Router::with_path("file/{token}").get(download_file),
        )
        .push(
            Router::with_path("pending")
                .get(list_pending)
                .push(
                    Router::with_path("{id}")
                        .get(review_pending)
                        .put(update_pending),
                ),
        )
}

// ============================================================
// 解析 multipart 字段的辅助
// ============================================================

/// 提取 form 字段为字符串
fn field_str<'a>(
    form: &'a salvo::http::form::FormData,
    key: &str,
) -> Option<&'a str> {
    form.fields.get(key).map(|s| s.as_str())
}

/// HnuDoc.md 中的 `date` 字段定义：JSON 格式的对象 `{typ, year}`，
/// 也可以传 `""` / `null` 表示未知年份。
fn parse_date_field(
    raw: Option<&str>,
) -> Result<(Option<String>, Option<i32>), AppError> {
    let Some(s) = raw else { return Ok((None, None)) };
    let s = s.trim();
    if s.is_empty() || s == "null" {
        return Ok((None, None));
    }
    let v: Value = serde_json::from_str(s).map_err(|_| {
        AppError::biz(ErrCode::BadRequest, "date 字段不是合法 JSON")
    })?;
    if v.is_null() {
        return Ok((None, None));
    }
    let typ = v
        .get("typ")
        .and_then(|x| x.as_str())
        .map(|x| x.to_string());
    let year = v.get("year").and_then(|x| x.as_i64()).map(|x| x as i32);
    if typ.is_none() && year.is_none() {
        Ok((None, None))
    } else if let (Some(t), Some(y)) = (typ, year) {
        Ok((Some(t), Some(y)))
    } else {
        Err(AppError::biz(
            ErrCode::BadRequest,
            "date 必须同时提供 typ 与 year",
        ))
    }
}

fn parse_string_array(raw: Option<&str>) -> Result<Vec<String>, AppError> {
    let Some(s) = raw else { return Ok(vec![]) };
    let s = s.trim();
    if s.is_empty() {
        return Ok(vec![]);
    }
    serde_json::from_str::<Vec<String>>(s).map_err(|_| {
        AppError::biz(
            ErrCode::BadRequest,
            "tags / categories 必须是字符串数组的 JSON",
        )
    })
}

fn parse_bool(raw: Option<&str>) -> Result<bool, AppError> {
    match raw.map(str::trim) {
        Some("true" | "1" | "yes") => Ok(true),
        Some("false" | "0" | "no" | "") | None => Ok(false),
        _ => Err(AppError::biz(
            ErrCode::BadRequest,
            "answer 字段不是合法的 boolean",
        )),
    }
}

fn extract_extension(path: &Path, file_name: Option<&str>) -> String {
    if let Some(name) = file_name
        && let Some(dot) = name.rfind('.')
    {
        return name[dot + 1..].to_ascii_lowercase();
    }
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default()
}

/// 一次性从 multipart 中读出所有需要的字段
struct ParsedDocForm {
    bytes: Option<Vec<u8>>,
    extension: Option<String>,
    name: String,
    typ: String,
    date_typ: Option<String>,
    date_year: Option<i32>,
    answer: bool,
    tags: Vec<String>,
    comment: Option<String>,
    categories: Vec<String>,
}

async fn parse_doc_form(
    req: &mut Request,
    file_required: bool,
) -> Result<ParsedDocForm, AppError> {
    let max = CFG.server.max_upload_size;
    // Salvo 0.80：先放宽 `form_data` 解析时的 body 上限，再取表单
    req.set_secure_max_size(max);
    let form = req
        .form_data()
        .await
        .map_err(AppError::SalvoParseError)?;

    // 先把所有字段拷出来
    let name = field_str(form, "name")
        .map(str::to_string)
        .ok_or_else(|| AppError::biz(ErrCode::BadRequest, "缺少 name"))?;
    let typ = field_str(form, "typ")
        .map(str::to_string)
        .ok_or_else(|| AppError::biz(ErrCode::BadRequest, "缺少 typ"))?;
    let date_raw = field_str(form, "date").map(str::to_string);
    let answer_raw = field_str(form, "answer").map(str::to_string);
    let tags_raw = field_str(form, "tags").map(str::to_string);
    let categories_raw = field_str(form, "categories").map(str::to_string);
    let comment = field_str(form, "comment")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // 文件（可能同名字段多个，取第一个）
    let (file_path, file_name): (Option<PathBuf>, Option<String>) =
        match form.files.get_vec("file").and_then(|v| v.first()) {
            Some(fp) => (
                Some(fp.path().clone()),
                fp.name().map(|s| s.to_string()),
            ),
            None => (None, None),
        };

    if file_required && file_path.is_none() {
        return Err(AppError::biz(
            ErrCode::BadRequest,
            "缺少 file 字段",
        ));
    }

    // 解析其余字段
    let (date_typ, date_year) = parse_date_field(date_raw.as_deref())?;
    let answer = parse_bool(answer_raw.as_deref())?;
    let tags = parse_string_array(tags_raw.as_deref())?;
    let categories = parse_string_array(categories_raw.as_deref())?;

    // 读文件内容（form 借用结束后再读）
    let (bytes, extension) = if let Some(p) = file_path {
        let bytes = fs::read(&p).await?;
        let ext = extract_extension(&p, file_name.as_deref());
        (Some(bytes), Some(ext))
    } else {
        (None, None)
    };

    Ok(ParsedDocForm {
        bytes,
        extension,
        name,
        typ,
        date_typ,
        date_year,
        answer,
        tags,
        comment,
        categories,
    })
}

// ============================================================
// POST /document   上传到待审核
// ============================================================

#[handler]
async fn upload_doc(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let user = service::user::whoami(&stu_id).await?;
    service::user::require_permission(&user, "upload")?;

    let parsed = parse_doc_form(req, true).await?;
    let bytes = parsed.bytes.expect("file_required=true");
    let ext = parsed.extension.unwrap_or_default();

    let id = service::document::save_pending_or_doc(
        &bytes,
        &ext,
        &parsed.name,
        &parsed.typ,
        parsed.date_typ.as_deref(),
        parsed.date_year,
        parsed.answer,
        &parsed.tags,
        parsed.comment.as_deref(),
        &parsed.categories,
        &stu_id,
    )
    .await?;

    Ok(serde_json::json!({ "pending_id": id }).into())
}

// ============================================================
// POST /document/download   创建 PoW 挑战
// ============================================================

#[handler]
async fn create_pow(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct Q {
        pub id: u32,
    }
    let _ = utils::jwt::auth(req)?;
    let Q { id } = req.extract().await?;
    let res = service::document::create_pow(id).await?;
    Ok(res.into())
}

// ============================================================
// GET /document/download   验证 PoW 拿到下载链接
// ============================================================

#[handler]
async fn get_download_link(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct Q {
        pub ticket: String,
        pub key: String,
    }
    let _ = utils::jwt::auth(req)?;
    let Q { ticket, key } = req.extract().await?;
    let url =
        service::document::consume_pow_and_issue_download(&ticket, &key)
            .await?;
    Ok(url.into())
}

// ============================================================
// GET /document/file/{token}   一次性下载链接
// ============================================================

#[handler]
async fn download_file(req: &mut Request, res: &mut Response) {
    // 这个接口不走 RouterResult，因为它需要直接发送二进制
    let token = match req.param::<String>("token") {
        Some(t) => t,
        None => {
            AppError::biz(ErrCode::BadRequest, "缺少 token").render_into(res);
            return;
        }
    };

    let row = match service::document::consume_download_token(&token).await
    {
        Ok(r) => r,
        Err(e) => {
            e.render_into(res);
            return;
        }
    };

    let path = crate::infra::storage::absolute_path(&row.file_path);
    if !path.exists() {
        AppError::biz(ErrCode::NotFound, "试卷文件不存在")
            .render_into(res);
        return;
    }

    // 用 salvo 自带的 NamedFile，自动处理 Range / Content-Type
    salvo::fs::NamedFile::builder(&path)
        .attached_name(format!("{}.{}", row.name, file_ext_from(&row.file_path)))
        .send(req.headers(), res)
        .await;
}

fn file_ext_from(rel: &str) -> String {
    rel.rsplit_once('.').map(|x| x.1.to_string()).unwrap_or_default()
}

/// 给 AppError 加一个直接 render 到 Response 的便捷方法
trait RenderInto {
    fn render_into(self, res: &mut Response);
}

impl RenderInto for AppError {
    fn render_into(self, res: &mut Response) {
        use salvo::Scribe;
        self.render(res);
    }
}

// ============================================================
// GET /document/pending   待审核列表（管理员；普通用户只能看自己的）
// ============================================================

#[handler]
async fn list_pending(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct Q {
        pub status: String,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page_size: Option<u32>,
    }
    let stu_id = utils::jwt::auth(req)?;
    let user = service::user::whoami(&stu_id).await?;
    let Q {
        status,
        page,
        page_size,
    } = req.extract().await?;

    if !["pending", "accepted", "rejected"]
        .contains(&status.as_str())
    {
        return Err(AppError::biz(
            ErrCode::BadRequest,
            "status 必须是 pending / accepted / rejected",
        ));
    }

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(10).clamp(1, 100);
    let mut list =
        service::pending::list_by_status(&status, page, page_size)
            .await?;

    // 普通用户只能看自己的提交
    if !user.permissions.iter().any(|p| p == "review") {
        list.retain(|x| x.stu_id == stu_id);
    }
    Ok(list.into())
}

// ============================================================
// PUT /document/pending/{id}   管理员更新已上传试卷信息
// ============================================================

#[handler]
async fn update_pending(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let user = service::user::whoami(&stu_id).await?;
    service::user::require_permission(&user, "review")?;

    let id: u32 = req.param::<u32>("id").ok_or_else(|| {
        AppError::biz(ErrCode::BadRequest, "缺少 id")
    })?;
    let exist = service::pending::get_by_id(id).await?;

    let parsed = parse_doc_form(req, false).await?;

    let (new_md5, new_page, new_path) =
        service::document::build_pending_update(
            parsed.bytes.as_deref(),
            parsed.extension.as_deref(),
            &exist.md5,
            &exist.file_path,
            exist.page,
        )
        .await?;

    crate::infra::mysql::pending::update_meta(
        id,
        &crate::infra::mysql::pending::UpdatePending {
            name: &parsed.name,
            typ: &parsed.typ,
            date_typ: parsed.date_typ.as_deref(),
            date_year: parsed.date_year,
            answer: parsed.answer,
            page: new_page,
            tags: &parsed.tags,
            comment: parsed.comment.as_deref(),
            md5: &new_md5,
            categories: &parsed.categories,
            file_path: parsed
                .bytes
                .as_ref()
                .map(|_| new_path.as_str()),
        },
    )
    .await?;

    Ok("OK".into())
}

// ============================================================
// GET /document/pending/{id}   管理员评审
// ============================================================

#[handler]
async fn review_pending(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct Q {
        pub status: String,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub comment: Option<String>,
    }

    let stu_id = utils::jwt::auth(req)?;
    let user = service::user::whoami(&stu_id).await?;
    service::user::require_permission(&user, "review")?;

    let id: u32 = req.param::<u32>("id").ok_or_else(|| {
        AppError::biz(ErrCode::BadRequest, "缺少 id")
    })?;
    let Q { status, comment } = req.extract().await?;

    if status == "rejected" && comment.is_none() {
        return Err(AppError::biz(
            ErrCode::BadRequest,
            "拒绝时必须提供 comment",
        ));
    }
    service::pending::review(id, &status, comment.as_deref()).await?;
    Ok("OK".into())
}
