// 用户登录与查询

use serde::Serialize;

use crate::{
    config::CFG,
    infra,
    result::{AppError, AppResult, ErrCode},
    utils,
};

// 前端 User DTO
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub stu_id: String,
    pub name: String,
    pub permissions: Vec<String>,
}

// 登录响应
#[derive(Debug, Clone, Serialize)]
pub struct LoginResult {
    pub token: String,
    pub user: User,
}

pub async fn login(
    raw_stu_id: &str,
    password: &str,
) -> AppResult<LoginResult> {
    let stu_id = utils::format_stuid(raw_stu_id);
    let existing = infra::mysql::user::get_by_stu_id(&stu_id).await?;

    let mock_ok = CFG.dev.mock_login
        && !CFG.dev.mock_stu_id.trim().is_empty()
        && !CFG.dev.mock_password.is_empty()
        && stu_id == utils::format_stuid(CFG.dev.mock_stu_id.trim())
        && password == CFG.dev.mock_password;

    if mock_ok {
        // 跳过密码校验
    }
    else {
        match &existing {
            Some(row) => {
                infra::verify::verify_local_password(row, password)?;
            }
            None => {
                return Err(AppError::biz(
                    ErrCode::PasswordError,
                    "账号未在本系统注册，请联系管理员开通或使用 dev.mock_login",
                ));
            }
        }
    }

    let name = if mock_ok {
        let n = CFG.dev.mock_name.trim();
        if n.is_empty() {
            "测试管理员".to_string()
        }
        else {
            n.to_string()
        }
    }
    else {
        existing
            .as_ref()
            .map(|u| u.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_default()
    };

    infra::mysql::user::upsert(
        &stu_id,
        &name,
        &utils::crypto::encrypt(password),
    )
    .await?;

    if mock_ok {
        let perms: Vec<String> = if CFG.dev.mock_permissions.is_empty() {
            vec![
                "search".into(),
                "download".into(),
                "upload".into(),
                "review".into(),
            ]
        }
        else {
            CFG.dev.mock_permissions.clone()
        };
        infra::mysql::user::update_permissions(&stu_id, &perms).await?;
    }

    let user = infra::mysql::user::get_by_stu_id(&stu_id)
        .await?
        .ok_or_else(|| {
            AppError::AnyHow(anyhow::anyhow!("写入用户后查询失败"))
        })?;

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
