use salvo::{Router, handler};

use crate::result::RouterResult;

pub fn routers() -> Router {
    Router::with_path("ping").get(ping)
}

#[handler]
async fn ping() -> RouterResult {
    Ok("pong".into())
}
