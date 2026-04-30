use std::time::{Duration, Instant};

use salvo::{
    Depot, FlowCtrl, Request, Response, handler,
    http::{ResBody, header::HeaderValue},
};
use tracing::Instrument;
use uuid::Uuid;

use crate::utils;

const SLOW_REQUEST_THRESHOLD: Duration = Duration::from_secs(3);

#[handler]
pub async fn logging_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let request_id = req
        .headers()
        .get("X-Request-Id")
        .and_then(|x| x.to_str().ok().map(|s| s.to_string()))
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let remote_addr = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|x| x.to_str().ok().map(|s| s.to_string()))
        .unwrap_or_else(|| req.remote_addr().to_string());

    let stu_id =
        utils::jwt::auth(req).unwrap_or_else(|_| "anon".into());

    let span = tracing::info_span!(
        "Request",
        request_id = %request_id,
        remote_addr = %remote_addr,
        method = %req.method(),
        path = %req.uri(),
        stu_id = %stu_id,
    );

    async move {
        if let Ok(v) = HeaderValue::from_str(&request_id) {
            res.headers_mut().insert("X-Request-Id", v);
        }
        let now = Instant::now();
        ctrl.call_next(req, depot, res).await;
        let duration = now.elapsed();

        let status = res.status_code.unwrap_or(match &res.body {
            ResBody::None => salvo::http::StatusCode::NOT_FOUND,
            ResBody::Error(e) => e.code,
            _ => salvo::http::StatusCode::OK,
        });

        if !status.is_success() {
            tracing::warn!(%status, ?duration, "Response");
        } else if duration > SLOW_REQUEST_THRESHOLD {
            tracing::warn!(%status, ?duration, "Slow Request");
        } else {
            tracing::info!(%status, ?duration, "Response");
        }
    }
    .instrument(span)
    .await
}
