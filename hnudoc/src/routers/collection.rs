use salvo::{Request, Router, handler};

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("collection").get(list)
}

#[handler]
async fn list(req: &mut Request) -> RouterResult {
    let _ = utils::jwt::auth(req)?;
    let res = service::collection::list_all().await?;
    Ok(res.into())
}
