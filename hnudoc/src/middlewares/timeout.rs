use std::time::Duration;

use salvo::{
    Depot, FlowCtrl, Request, Response, handler,
    http::headers::{Connection, HeaderMapExt},
};

use crate::result::AppError;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[handler]
pub async fn timeout_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    tokio::select! {
        _ = ctrl.call_next(req, depot, res) => {}
        _ = tokio::time::sleep(REQUEST_TIMEOUT) => {
            res.headers_mut().typed_insert(Connection::close());
            res.render(AppError::TimeoutError);
            ctrl.skip_rest();
        }
    }
}
