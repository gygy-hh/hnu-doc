//! JWT：用学号生成 / 解析 token
//!
//! 与参考后端不同的是：HnuDoc 接口要求验证 token 是否过期。

use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use salvo::Request;
use serde::{Deserialize, Serialize};
use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    config::CFG,
    result::{AppError, AppResult, ErrCode},
    utils,
};

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    iss: String,
    exp: usize,
    sub: String,
    iat: usize,
    stu_id: String,
}

static VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
    let mut v = Validation::default();
    v.validate_exp = true;
    v
});

fn now_secs() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
}

/// 用学号生成 token
pub fn generate_jwt(stu_id: &str) -> AppResult<String> {
    let now = now_secs();
    let claims = Claims {
        iss: "hnudoc".into(),
        exp: now + CFG.jwt.expire_secs,
        sub: "user".into(),
        iat: now,
        stu_id: stu_id.into(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CFG.jwt.secret.as_bytes()),
    )?;
    Ok(token)
}

/// 解析 token 获得学号，过期 / 签名错误均返回 Unauthorized
pub fn parse(token: &str) -> AppResult<String> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &VALIDATION,
    )?;
    Ok(utils::format_stuid(&res.claims.stu_id))
}

/// 从请求头里读取 Authorization 并验证。
///
/// HnuDoc 的前端可能把 token 直接放在 `Authorization` 里，
/// 也可能写成 `Bearer <token>`，两种都要兼容。
pub fn auth(req: &mut Request) -> AppResult<String> {
    let raw = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| {
            AppError::biz(ErrCode::Unauthorized, "未携带 token")
        })?
        .to_str()
        .map_err(|_| {
            AppError::biz(ErrCode::Unauthorized, "token 格式错误")
        })?;
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .unwrap_or(raw)
        .trim();
    parse(token).map_err(|_| {
        AppError::biz(ErrCode::Unauthorized, "token 无效或已过期")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let stu_id = "S20231234";
        let token = generate_jwt(stu_id).expect("encode failed");
        let got = parse(&token).expect("decode failed");
        assert_eq!(got, stu_id);
    }
}
