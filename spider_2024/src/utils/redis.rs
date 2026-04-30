use std::time::Duration;

use anyhow::Result;
use log::error;
use redis::{
    AsyncCommands, Client,
    aio::{ConnectionManager, ConnectionManagerConfig},
};
use tokio::sync::OnceCell;

use crate::{config::CFG, utils::db::get_lab_password_from_db};

use super::db::get_password_from_db;

pub static REDIS_CONN_MGR: OnceCell<ConnectionManager> =
    OnceCell::const_new();

/// # Performance
///
/// 按照[`redis`]文档，异步Redis请求不需要连接池，可以使用多路复用。
/// > For async connections, connection pooling isn't necessary.
/// > The MultiplexedConnection is cloneable and can be used safely from multiple threads,
/// > so a single connection can be easily reused.
/// > For automatic reconnections consider using ConnectionManager with the connection-manager
/// > feature.
/// > Async cluster connections also don't require pooling and are thread-safe and reusable.
pub async fn redis_conn_mgr() -> &'static ConnectionManager {
    REDIS_CONN_MGR
        .get_or_init(|| async {
            ConnectionManager::new_with_config(
                Client::open(CFG.redis.redis_url.clone())
                    .expect("连接redis失败"),
                ConnectionManagerConfig::new()
                    // 设置超时是重要的，避免超时中间件触发后任务仍在进行
                    .set_connection_timeout(Duration::from_secs(3))
                    // 设置超时是重要的，避免超时中间件触发后任务仍在进行
                    .set_response_timeout(Duration::from_secs(3)),
            )
            .await
            .unwrap()
        })
        .await
}

async fn get_password_from_redis(stu_id: &str) -> Result<String> {
    let con = redis_conn_mgr().await;
    let key = format!("stu_pass_{stu_id}");
    let res: String = con.clone().get(&key).await?;
    // let res: String = redis::pipe().get(key).query_async(&mut con).await?;
    Ok(res)
}

async fn insert_password_to_redis(
    stu_id: &str,
    password: &str,
) -> Result<()> {
    let con = redis_conn_mgr().await;
    let key = format!("stu_pass_{stu_id}");
    // let _: () = con.set(&key, password).await?;
    let _: () = redis::pipe()
        .set(key, password)
        .query_async(&mut con.clone())
        .await?;
    Ok(())
}

pub async fn insert_lab_password_to_redis(
    stu_id: &str,
    password: &str,
) -> Result<()> {
    let con = redis_conn_mgr().await;
    let key = format!("lab_pwd_{stu_id}");
    let _: () = redis::pipe()
        .set(key, password)
        .query_async(&mut con.clone())
        .await?;
    Ok(())
}

async fn get_lab_password_from_redis(stu_id: &str) -> Result<String> {
    let con = redis_conn_mgr().await;
    let key = format!("lab_pwd_{stu_id}");
    let res: String = con.clone().get(&key).await?;
    // let res: String = redis::pipe().get(key).query_async(&mut con).await?;
    Ok(res)
}

pub async fn fetch_password(
    stu_id: &str,
) -> Result<String, crate::Error> {
    if let Ok(password) = get_password_from_redis(stu_id).await {
        Ok(password)
    } else {
        let password =
            get_password_from_db(stu_id).await.map_err(|e| {
                // 将具体错误记录到日志中，但是对于前端，让其重新绑定
                error!(
                    "从数据库中获取密码时错误，stu_id: {}, {}",
                    stu_id, e
                );
                crate::Error::PasswordError
            })?;
        // 缓存密码到redis
        insert_password_to_redis(stu_id, &password).await?;
        Ok(password)
    }
}

pub async fn fetch_lab_password(
    stu_id: &str,
) -> Result<Option<String>, crate::Error> {
    if let Ok(password) = get_lab_password_from_redis(stu_id).await {
        Ok(Some(password))
    } else {
        let password =
            get_lab_password_from_db(stu_id).await.map_err(|e| {
                error!(
                    "获取实验平台密码错误, stuid = {}, {}",
                    stu_id, e
                );
                crate::Error::PasswordError
            })?;
        if let Some(password) = password {
            // 缓存密码到redis
            insert_lab_password_to_redis(stu_id, &password).await?;
            Ok(Some(password))
        } else {
            Ok(None)
        }
    }
}

pub async fn add_cookie_to_redis(
    key: &str,
    cookie: &str,
    stu_id: &str,
    timeout: i64,
) -> Result<()> {
    let con = redis_conn_mgr().await;
    let key = format!("{}-{}", key, stu_id);
    let _: () = redis::pipe()
        .set(&key, cookie)
        .expire(&key, timeout)
        .query_async(&mut con.clone())
        .await?;
    Ok(())
}

pub async fn get_cookie_from_redis(
    key: &str,
    stu_id: &str,
) -> Result<String> {
    let con = redis_conn_mgr().await;
    let key = format!("{}-{}", key, stu_id);
    let res: String = con.clone().get(&key).await?;
    Ok(res)
}

#[expect(dead_code)]
pub async fn remove_cookie_from_redis(
    key: &str,
    stu_id: &str,
) -> Result<()> {
    let con = redis_conn_mgr().await;
    let key = format!("{}-{}", key, stu_id);
    let _: () = redis::pipe()
        .del(&key)
        .query_async(&mut con.clone())
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_redis_init() {
        let _ = redis_conn_mgr().await;
    }
}
