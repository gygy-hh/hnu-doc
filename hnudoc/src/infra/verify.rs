//! 复用 spider_2024 的 CAS 密码验证

use spider_2024::dtos::pt::CasPasswordStatus;

use crate::result::{AppError, AppResult, ErrCode};

/// 验证个人门户密码。失败时直接返回带错误代码的 [`AppError::Biz`]。
pub async fn verify_password(
    stu_id: &str,
    password: &str,
) -> AppResult<()> {
    let status =
        spider_2024::pt::check_password_handler(stu_id, password)
            .await?;
    match status {
        CasPasswordStatus::Success => Ok(()),
        CasPasswordStatus::Fail => Err(AppError::biz(
            ErrCode::PasswordError,
            "密码错误",
        )),
        CasPasswordStatus::ShouldChange => Err(AppError::biz(
            ErrCode::PasswordError,
            "请前往个人门户修改密码后重试",
        )),
        CasPasswordStatus::Lock => Err(AppError::biz(
            ErrCode::PasswordError,
            "账号被锁定，请暂停使用 10 分钟后重试",
        )),
    }
}

/// 通过爬虫从学工系统获取个人信息（含姓名）
pub async fn fetch_person_info(
    stu_id: &str,
) -> AppResult<spider_2024::dtos::xgxt::PersonInfo> {
    let info =
        spider_2024::xgxt::get_person_info_handler(stu_id).await?;
    Ok(info)
}
