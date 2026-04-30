//! 用户登录 / 信息相关业务

use serde::Serialize;

use crate::{
    infra,
    result::{AppError, AppResult, ErrCode},
    utils,
};

/// 与前端约定的 User 结构
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub stu_id: String,
    pub name: String,
    pub permissions: Vec<String>,
}

/// 登录后的返回数据
#[derive(Debug, Clone, Serialize)]
pub struct LoginResult {
    pub token: String,
    pub user: User,
}

/// 登录流程：
/// 1. 个人门户校验密码（spider_2024）
/// 2. 拉取学工系统的 PersonInfo 拿姓名（首次登录时）
/// 3. upsert 到本地 `users` 表
/// 4. 检查是否被禁用，签发 JWT
pub async fn login(
    raw_stu_id: &str,
    password: &str,
) -> AppResult<LoginResult> {
    let stu_id = utils::format_stuid(raw_stu_id);

    // 1. 验证个人门户密码
    infra::verify::verify_password(&stu_id, password).await?;

    // 2. 决定姓名：本地有就沿用，没有就拉一次 PersonInfo
    let existing = infra::mysql::user::get_by_stu_id(&stu_id).await?;
    let name = match existing.as_ref() {
        Some(u) if !u.name.is_empty() => u.name.clone(),
        _ => match infra::verify::fetch_person_info(&stu_id).await {
            Ok(info) => info.name,
            Err(e) => {
                // 姓名拿不到不影响登录
                tracing::warn!("拉取 PersonInfo 失败: {e}");
                String::new()
            }
        },
    };

    // 3. upsert
    infra::mysql::user::upsert(
        &stu_id,
        &name,
        &utils::crypto::encrypt(password),
    )
    .await?;

    // 重新读一遍拿到权限
    let user = infra::mysql::user::get_by_stu_id(&stu_id)
        .await?
        .ok_or_else(|| {
            AppError::AnyHow(anyhow::anyhow!("写入用户后查询失败"))
        })?;

    // 4. 黑名单：permissions 中有 "banned" 或为空数组都视为禁用
    if user.permissions.iter().any(|p| p == "banned") {
        return Err(AppError::biz(
            ErrCode::PermissionDenied,
            "该账号被禁止使用试卷库",
        ));
    }

    let token = utils::jwt::generate_jwt(&stu_id)?;

    Ok(LoginResult {
        token,
        user: User {
            stu_id: user.stu_id,
            name: user.name,
            permissions: user.permissions,
        },
    })
}

/// 通过学号取出 User
pub async fn whoami(stu_id: &str) -> AppResult<User> {
    let row = infra::mysql::user::get_by_stu_id(stu_id)
        .await?
        .ok_or_else(|| {
            AppError::biz(ErrCode::Unauthorized, "用户不存在")
        })?;
    Ok(User {
        stu_id: row.stu_id,
        name: row.name,
        permissions: row.permissions,
    })
}

/// 检查权限：用户必须拥有 `perm`
pub fn require_permission(
    user: &User,
    perm: &str,
) -> AppResult<()> {
    if user.permissions.iter().any(|p| p == perm) {
        Ok(())
    } else {
        Err(AppError::biz(
            ErrCode::PermissionDenied,
            format!("缺少权限: {perm}"),
        ))
    }
}
