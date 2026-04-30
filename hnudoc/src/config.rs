//! 全局配置，懒加载，支持多个候选路径

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Deserialize, Debug)]
pub struct Configs {
    pub server: Server,
    pub database: Database,
    pub redis: Redis,
    pub jwt: Jwt,
    pub pow: Pow,
    pub log: Log,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub address: String,
    pub upload_dir: String,
    pub max_upload_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct Database {
    pub max_connections: u32,
    pub database_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Redis {
    pub redis_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Jwt {
    pub secret: String,
    pub expire_secs: usize,
}

#[derive(Deserialize, Debug)]
pub struct Pow {
    pub zero_bits: u32,
    pub ticket_ttl: u64,
    pub download_ttl: u64,
}

#[derive(Deserialize, Debug)]
pub struct Log {
    pub filter_level: String,
    pub with_ansi: bool,
    pub to_stdout: bool,
    pub directory: String,
    pub file_name: String,
    pub rolling: String,
    pub format: String,
}

pub static CFG: Lazy<Configs> = Lazy::new(Configs::init);

fn try_config_file(path: &str) -> Result<Configs, String> {
    let s = read_to_string(path)
        .map_err(|e| format!("读取配置文件失败: {e}"))?;
    toml::from_str(&s).map_err(|e| format!("解析配置文件失败: {e}"))
}

impl Configs {
    pub fn init() -> Self {
        let candidates = [
            "config/config.toml",
            concat!(env!("CARGO_MANIFEST_DIR"), "/config/config.toml"),
            "../config/config.toml",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../config/config.toml"
            ),
        ];
        for path in candidates {
            println!("[?] 尝试加载配置: {path}");
            match try_config_file(path) {
                Ok(cfg) => {
                    println!("[i] 使用配置文件: {path}");
                    return cfg;
                }
                Err(e) => println!("[!] {e}"),
            }
        }
        panic!("[!] 找不到任何可用的配置文件");
    }
}
