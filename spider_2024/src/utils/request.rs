// HTTP 客户端（见 mod re-export）

use reqwest::{
    Client,
    header::{GetAll, HeaderValue},
    redirect::Policy,
};
use serde_json::Value;
use std::{sync::LazyLock, time::Duration};

use super::cache::{CACHE, CacheEnum};

// Debug 下占位学号；本地须校园网 DNS、关代理
#[cfg(debug_assertions)]
#[cfg_attr(not(test), expect(unused))]
pub static STU_ID: LazyLock<String> =
    LazyLock::new(|| "".to_string());

pub static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        // 图书馆证书异常；HTTP ticket 受限故仍走 HTTPS
        .danger_accept_invalid_certs(true)
        .connection_verbose(false)
        // 统一超时，配合网关超时中间件
        .timeout(Duration::from_secs(6))
        .connect_timeout(Duration::from_secs(2))
        .pool_idle_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(2000) // 生产按 FD 上限收紧
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36") // CAS 校验需要浏览器 UA
        // .user_agent("reqwest/0.12.8")
        .no_proxy()
        .redirect(Policy::none()) // 部分登录流程手写重定向
        // .http1_title_case_headers()
        .build()
        .expect("构建client失败")
});

// Set-Cookie → key=value 列表
#[inline]
pub fn cookie_parser(cookie: GetAll<HeaderValue>) -> Vec<String> {
    cookie
        .iter()
        .filter_map(cookie_parser_inner)
        .collect::<Vec<String>>()
}

#[inline]
fn cookie_parser_inner(cookie: &HeaderValue) -> Option<String> {
    let cookie = cookie
        .to_str()
        .expect("异常cookie")
        .split(';')
        .collect::<Vec<&str>>();
    let pair: Vec<&str> = cookie[0].split('=').collect();
    if pair[1].is_empty() {
        return None;
    }
    Some(format!("{}={}", pair[0], pair[1]))
}

pub trait CacheChecker {
    async fn check_gym(self, _stu_id: &str) -> Self
    where
        Self: Sized,
    {
        self
    }
}

impl CacheChecker for serde_json::Value {
    // 典型的异常response body：
    // {"data":[],"info":"登录失效","status":-1}
    async fn check_gym(self, stu_id: &str) -> Self {
        if let Some(Value::String(info)) = self.get("info")
            && info.contains("登录失效")
        {
            CACHE
                .invalidate(&(CacheEnum::GymCookie, stu_id.into()))
                .await;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_request() {
        let res =
            CLIENT.get("https://www.baidu.com").send().await.unwrap();
        assert!(res.status().is_success());
    }
}
