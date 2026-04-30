pub mod collection;
pub mod document;
pub mod pending;
pub mod user;

use crate::config::CFG;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<MySqlPool> = OnceCell::const_new();

/// 获取全局 MySQL 连接池，连接失败时退出进程
pub async fn pool() -> &'static MySqlPool {
    DB_POOL
        .get_or_init(|| async {
            match MySqlPoolOptions::new()
                .max_connections(CFG.database.max_connections)
                .acquire_timeout(Duration::from_secs(3))
                .connect(&CFG.database.database_url)
                .await
            {
                Ok(p) => {
                    tracing::info!("MySQL连接成功");
                    p
                }
                Err(e) => {
                    tracing::error!("MySQL连接失败: {e}");
                    std::process::exit(1);
                }
            }
        })
        .await
}
