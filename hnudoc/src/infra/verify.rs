// 本地用户表密文密码校验（已不再调用校园网爬虫）

use crate::{
    infra::mysql::user::UserRow,
    result::{AppError, AppResult, ErrCode},
    utils,
};

pub fn verify_local_password(row: &UserRow, password: &str) -> AppResult<()> {
    let plain = utils::crypto::decrypt(&row.password).map_err(|e| {
        tracing::error!("解密存储密码失败: {e}");
        AppError::biz(ErrCode::PasswordError, "登录校验失败")
    })?;
    if plain != password {
        return Err(AppError::biz(ErrCode::PasswordError, "密码错误"));
    }
    Ok(())
}
