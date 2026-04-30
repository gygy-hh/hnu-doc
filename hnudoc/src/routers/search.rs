use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{
    result::{AppError, ErrCode, RouterResult},
    service,
    utils::{self, serde::empty_string_as_none},
};

pub fn routers() -> Router {
    Router::with_path("search")
        .get(search_subjects)
        .push(Router::with_path("subject").get(search_subject_docs))
}

/// `typ` 可以传多个，用 `,` 分隔，例如 `typ=final,mid`
fn parse_typs(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[handler]
async fn search_subjects(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct Req {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub key: Option<String>,
        pub typ: String,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page_size: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page: Option<u32>,
    }
    let _ = utils::jwt::auth(req)?;
    let q: Req = req.extract().await?;
    let typs = parse_typs(&q.typ);
    if typs.is_empty() {
        return Err(AppError::biz(
            ErrCode::BadRequest,
            "typ 不能为空",
        ));
    }
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).clamp(1, 100);
    let key = q.key.as_deref().filter(|s| !s.is_empty());
    let res =
        service::search::search_subjects(key, &typs, page, page_size)
            .await?;
    Ok(res.into())
}

#[handler]
async fn search_subject_docs(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct Req {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub name: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub typ: Option<String>,
    }
    let _ = utils::jwt::auth(req)?;
    let q: Req = req.extract().await?;
    let res = service::search::list_by_subject(
        q.name.as_deref(),
        q.typ.as_deref(),
    )
    .await?;
    Ok(res.into())
}
