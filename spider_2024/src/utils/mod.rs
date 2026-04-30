pub(crate) mod cache;
pub(crate) mod captcha;
pub(crate) mod crypto;
pub(crate) mod db;
pub(crate) mod redis;
pub(crate) mod request;

// 发送请求的全局请求池，设置请求上限为1000个
pub(crate) use request::CLIENT as client;
// pub(crate) use request::MAP as map; // 用来存放缓存的表
