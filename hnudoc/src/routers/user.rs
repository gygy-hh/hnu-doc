use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{
    result::{AppError, ErrCode, RouterResult},
    service, utils,
};

pub fn routers() -> Router {
    Router::with_path("user")
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("logout").get(logout))
        .push(Router::with_path("whoami").get(whoami))
}

#[handler]
async fn login(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    struct LoginReq {
        pub stuid: String,
        pub password: String,
    }
    let LoginReq { stuid, password } = req.extract().await?;
    if stuid.trim().is_empty() || password.is_empty() {
        return Err(AppError::biz(
            ErrCode::BadRequest,
            "stuid 和 password 不能为空",
        ));
    }
    let res = service::user::login(&stuid, &password).await?;
    Ok(res.into())
}

#[handler]
async fn logout(_req: &mut Request) -> RouterResult {
    // JWT 无状态，前端丢弃 token 即可
    Ok("OK".into())
}

#[handler]
async fn whoami(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let user = service::user::whoami(&stu_id).await?;
    Ok(user.into())
}
