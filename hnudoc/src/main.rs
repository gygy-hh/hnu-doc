#![forbid(unsafe_code)]
#![warn(clippy::too_many_lines)]

mod config;
mod infra;
mod middlewares;
mod result;
mod routers;
mod service;
mod utils;

use crate::{
    config::CFG,
    middlewares::{
        catch_panic::catch_panic_middleware,
        cors::cors_middleware,
        default::default_middleware,
        logging::logging_middleware,
        timeout::timeout_middleware,
    },
};
use salvo::Service;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    run().await
}

async fn run() {
    let _guard = clia_tracing_config::build()
        .filter_level(&CFG.log.filter_level)
        .with_ansi(CFG.log.with_ansi)
        .to_stdout(CFG.log.to_stdout)
        .directory(&CFG.log.directory)
        .file_name(&CFG.log.file_name)
        .rolling(&CFG.log.rolling)
        .with_source_location(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_target(false)
        .format(&CFG.log.format)
        .init();

    tracing::info!("📓 日志级别: {}", &CFG.log.filter_level);
    tracing::info!("🚀 {} 启动中", &CFG.server.name);
    tracing::info!("🔄 监听地址: {}", &CFG.server.address);

    if let Err(e) = infra::storage::ensure_dir().await {
        tracing::error!("创建上传目录失败: {e}");
        std::process::exit(1);
    }

    let listener = TcpListener::new(&CFG.server.address).bind().await;
    let routers = routers::routers();
    let service = Service::new(routers)
        .hoop(catch_panic_middleware)
        .hoop(default_middleware)
        .hoop(logging_middleware)
        .hoop(cors_middleware())
        .hoop(timeout_middleware);
    Server::new(listener).serve(service).await;
}
