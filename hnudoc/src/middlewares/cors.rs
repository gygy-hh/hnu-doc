use salvo::cors::{Cors, CorsHandler};
use salvo::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use salvo::http::Method;

pub fn cors_middleware() -> CorsHandler {
    Cors::new()
        .allow_origin([
            "http://localhost:3000",
            "http://127.0.0.1:3000",
        ])
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PUT,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
        .into_handler()
}
