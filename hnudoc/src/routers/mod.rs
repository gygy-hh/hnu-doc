mod collection;
mod document;
mod ping;
mod search;
mod user;

use salvo::Router;

pub fn routers() -> Router {
    Router::new()
        .push(ping::routers())
        .push(user::routers())
        .push(collection::routers())
        .push(search::routers())
        .push(document::routers())
}
