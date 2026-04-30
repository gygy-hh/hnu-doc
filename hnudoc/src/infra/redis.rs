//! Redis 连接，与参考后端一致使用 ConnectionManager

use std::time::Duration;

use redis::{
    AsyncCommands, Client,
    aio::{ConnectionManager, ConnectionManagerConfig},
};
use tokio::sync::OnceCell;

use crate::{config::CFG, result::AppResult};

static MGR: OnceCell<ConnectionManager> = OnceCell::const_new();

async fn conn() -> ConnectionManager {
    MGR.get_or_init(|| async {
        ConnectionManager::new_with_config(
            Client::open(CFG.redis.redis_url.clone())
                .expect("解析 Redis URL 失败"),
            ConnectionManagerConfig::new()
                .set_connection_timeout(Duration::from_secs(3))
                .set_response_timeout(Duration::from_secs(3)),
        )
        .await
        .expect("建立 Redis 连接失败")
    })
    .await
    .clone()
}

#[allow(dead_code)]
pub async fn set(key: &str, value: &str) -> AppResult<()> {
    let mut c = conn().await;
    let _: () = c.set(key, value).await?;
    Ok(())
}

pub async fn set_with_expire(
    key: &str,
    value: &str,
    expire_secs: u64,
) -> AppResult<()> {
    let mut c = conn().await;
    let _: () = c.set_ex(key, value, expire_secs).await?;
    Ok(())
}

pub async fn get(key: &str) -> AppResult<Option<String>> {
    let mut c = conn().await;
    let v: Option<String> = c.get(key).await?;
    Ok(v)
}

pub async fn del(key: &str) -> AppResult<()> {
    let mut c = conn().await;
    let _: () = c.del(key).await?;
    Ok(())
}
