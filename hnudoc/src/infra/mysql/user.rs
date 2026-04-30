//! `users` 表的访问

use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::Row;

use super::pool;
use crate::{result::AppResult, utils};

/// `users` 表的完整记录
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UserRow {
    pub stu_id: String,
    pub name: String,
    /// 加密密文，业务层未读取时由编译器标为未使用
    pub password: String,
    pub permissions: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

fn parse_permissions(v: Value) -> Vec<String> {
    serde_json::from_value(v).unwrap_or_default()
}

pub async fn get_by_stu_id(
    stu_id: &str,
) -> AppResult<Option<UserRow>> {
    let row = sqlx::query(
        r#"
        SELECT stu_id, name, password, permissions, created_at, updated_at
        FROM users
        WHERE stu_id = ?
        "#,
    )
    .bind(stu_id)
    .fetch_optional(pool().await)
    .await?;

    Ok(row.map(|r| UserRow {
        stu_id: r.get("stu_id"),
        name: r.get::<String, _>("name"),
        password: r.get("password"),
        permissions: parse_permissions(r.get::<Value, _>("permissions")),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

/// 不存在则插入；存在则更新姓名 / 密码（保持 permissions 不变）。
/// 默认权限：`["search","download","upload"]`。
pub async fn upsert(
    stu_id: &str,
    name: &str,
    encrypted_password: &str,
) -> AppResult<()> {
    let now = utils::time::now();
    let default_perms = serde_json::json!([
        "search", "download", "upload"
    ]);

    sqlx::query(
        r#"
        INSERT INTO users
            (stu_id, name, password, permissions, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            password = VALUES(password),
            updated_at = VALUES(updated_at)
        "#,
    )
    .bind(stu_id)
    .bind(name)
    .bind(encrypted_password)
    .bind(default_perms)
    .bind(now)
    .bind(now)
    .execute(pool().await)
    .await?;
    Ok(())
}

/// 给管理员或脚本调整权限时调用（当前未接 HTTP 接口）
#[allow(dead_code)]
pub async fn update_permissions(
    stu_id: &str,
    permissions: &[String],
) -> AppResult<()> {
    let v = serde_json::to_value(permissions)?;
    sqlx::query(
        r#"
        UPDATE users SET permissions = ?, updated_at = ?
        WHERE stu_id = ?
        "#,
    )
    .bind(v)
    .bind(utils::time::now())
    .bind(stu_id)
    .execute(pool().await)
    .await?;
    Ok(())
}
