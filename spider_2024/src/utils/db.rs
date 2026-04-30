use std::time::Duration;

use crate::{config::CFG, utils::crypto::decrypt};
use anyhow::Result;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use tokio::sync::OnceCell;

/// 获取数据库连接池的引用
#[inline]
pub async fn db_pool() -> &'static MySqlPool {
    pub static DB: OnceCell<MySqlPool> = OnceCell::const_new();
    DB.get_or_init(|| async {
        let options = MySqlPoolOptions::new()
            .max_connections(66)
            // 设置超时是重要的，避免超时中间件触发后任务仍在进行
            .acquire_timeout(Duration::from_secs(3));
        options.connect(&CFG.database.database_url).await.unwrap()
    })
    .await
}

pub async fn get_password_from_db(stu_id: &str) -> Result<String> {
    let res = sqlx::query!(
        "SELECT password FROM mini_bind WHERE stuId = ?",
        stu_id
    )
    .fetch_one(db_pool().await)
    .await?;
    let password_decrypted = decrypt(&res.password)?;
    Ok(password_decrypted)
}

pub async fn get_lab_password_from_db(
    stu_id: &str,
) -> Result<Option<String>> {
    let res = sqlx::query!(
        "SELECT labPass FROM mini_bind WHERE stuId = ?",
        stu_id
    )
    .fetch_one(db_pool().await)
    .await?;
    if let Some(lab_pass) = res.labPass {
        let password_decrypted = decrypt(&lab_pass)?;
        Ok(Some(password_decrypted))
    } else {
        Ok(None)
    }
}
