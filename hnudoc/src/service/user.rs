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

// CAS 验密 → PersonInfo 补姓名 → upsert → 禁用的是 banned → JWT
pub async fn login(
    raw_stu_id: &str,
    password: &str,
) -> AppResult<LoginResult> {
    let stu_id = utils::format_stuid(raw_stu_id);

    let mock_ok = CFG.dev.mock_login
        && !CFG.dev.mock_stu_id.trim().is_empty()
        && !CFG.dev.mock_password.is_empty()
        && stu_id == utils::format_stuid(CFG.dev.mock_stu_id.trim())
        && password == CFG.dev.mock_password;

    if !mock_ok {
        infra::verify::verify_password(&stu_id, password).await?;
    }

    let existing = infra::mysql::user::get_by_stu_id(&stu_id).await?;
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
        match existing.as_ref() {
            Some(u) if !u.name.is_empty() => u.name.clone(),
            _ => match infra::verify::fetch_person_info(&stu_id).await {
                Ok(info) => info.name,
                Err(e) => {
                    tracing::warn!("拉取 PersonInfo 失败: {e}");
                    String::new()
                }
            },
        }
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

// 当前用户
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

// 要求具备指定权限
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
