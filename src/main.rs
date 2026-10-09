use anyhow::Ok;
use tokio::signal::ctrl_c;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::AppConfig;

mod config;

fn init_logging(level: &str) {
    // RUST_LOG
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::load("config/config.toml")?;
    init_logging(&cfg.log_level);
    info!("iot-gateway start, instance_id = {}", cfg.instance_id);
    wait_for_shutdown().await;
    Ok(())
}

// 监听Ctrl+C
async fn wait_for_shutdown() {

    // 信号
    let ctrl_c = async {
        ctrl_c().await.ok();
    };
    // 并发等待宏， 这里等待ctrlC完成
    tokio::select! {
        _ = ctrl_c => {}
    }
}
