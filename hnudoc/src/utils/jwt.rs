// JWT（校验过期）

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

// 签发
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

// 解析学号
pub fn parse(token: &str) -> AppResult<String> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &VALIDATION,
    )?;
    Ok(utils::format_stuid(&res.claims.stu_id))
}

// Authorization：支持裸 token 或 Bearer
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
