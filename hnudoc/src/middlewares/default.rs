// 兜底 JSON

use anyhow::anyhow;
use salvo::{
    Depot, FlowCtrl, Request, Response, handler, writing::Json,
};
use serde_json::json;

use crate::result::AppError;

#[handler]
pub async fn default_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    ctrl.call_next(req, depot, res).await;
    let body_size = res.body.size().unwrap_or(0);
    if body_size > 0 {
        return;
    }
    match res.status_code {
        None => res.render(AppError::AnyHow(anyhow!(
            "服务器未返回有效信息"
        ))),
        Some(code) => {
            res.stuff(
                code,
                Json(json!({
                    "status": code.canonical_reason()
                        .unwrap_or("UNKNOWN")
                        .to_uppercase()
                        .replace(' ', "_"),
                    "data": null,
                    "msg": code.canonical_reason().unwrap_or("未知错误"),
                })),
            );
        }
    }
}
