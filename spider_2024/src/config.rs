use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Clone, Debug, Deserialize)]
pub struct Configs {
    pub database: Database,
    pub redis: Redis,
    pub captcha: Captcha,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    pub database_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Redis {
    pub redis_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Captcha {
    pub captcha_url: String,
}

pub static CFG: Lazy<Configs> = Lazy::new(Configs::init);

fn try_config_file(config_file: &str) -> Result<Configs, String> {
    let cfg_contents = read_to_string(config_file).map_err(|e| {
        format!("Cannot read configuration file: {}", e)
    })?;

    toml::from_str(&cfg_contents).map_err(|e| {
        format!("Cannot parse configuration file: {}", e)
    })
}

impl Configs {
    pub fn init() -> Self {
        let config_file_candidates = vec![
            "config/config.toml",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/config/config.toml"
            ),
            "../config/config.toml",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../config/config.toml"
            ),
        ];

        for config_file in config_file_candidates {
            println!(
                "[?] Trying configuration file: {}",
                config_file
            );

            match try_config_file(config_file) {
                Ok(cfg) => {
                    println!(
                        "[i] Using configuration file: {}",
                        config_file
                    );
                    return cfg;
                }
                Err(e) => println!("[!] {}", e),
            }
        }

        // 若到达此处，说明 for 循环已完成且未返回，即所有候选配置文件都不可用
        panic!("[!] No valid configuration file found!");
    }
}
