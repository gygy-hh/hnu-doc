use std::time::Duration;

use crate::{config::CFG, utils::crypto::decrypt};
use anyhow::Result;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<MySqlPool> = OnceCell::const_new();

// 连接池
#[inline]
pub async fn db_pool() -> &'static MySqlPool {
    DB_POOL
        .get_or_init(|| async {
            let options = MySqlPoolOptions::new()
                .max_connections(66)
                // 配合网关超时
                .acquire_timeout(Duration::from_secs(3));
            options.connect(&CFG.database.database_url).await.unwrap()
        })
        .await
}

pub async fn get_password_from_db(stu_id: &str) -> Result<String> {
    let password_enc: String = sqlx::query_scalar(
        "SELECT password FROM mini_bind WHERE stuId = ?",
    )
    .bind(stu_id)
    .fetch_one(db_pool().await)
    .await?;
    let password_decrypted = decrypt(&password_enc)?;
    Ok(password_decrypted)
}

pub async fn get_lab_password_from_db(
    stu_id: &str,
) -> Result<Option<String>> {
    let lab_pass: Option<String> = sqlx::query_scalar(
        "SELECT labPass FROM mini_bind WHERE stuId = ?",
    )
    .bind(stu_id)
    .fetch_one(db_pool().await)
    .await?;
    match lab_pass {
        Some(s) => Ok(Some(decrypt(&s)?)),
        None => Ok(None),
    }
}
