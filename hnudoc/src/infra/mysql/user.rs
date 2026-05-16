// users

use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::Row;

use super::pool;
use crate::{result::AppResult, utils};

// 行数据（含密文）
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UserRow {
    pub stu_id: String,
    pub name: String,
    // AES 密文
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

// upsert：改姓名/密码，权限默认 search,download,upload
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

// 更新 permissions（无 HTTP）
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
