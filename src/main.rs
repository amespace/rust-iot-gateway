use std::time::Duration;

use tokio::signal::ctrl_c;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::AppConfig;
mod codec;
mod config;
mod error;
mod frames;
mod server;

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
    let tcp_addr = cfg.tcp_addr.clone();

    tracing::info!(
        instance_id = %cfg.instance_id,
        tcp  = %tcp_addr,
        "iot-gateway 启动"
    );

    // 跑TCP
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let mut sr_rx = shutdown_rx.clone();
    let server_task = tokio::spawn(async move {
        server::serve(&tcp_addr, async move {
            sr_rx.changed().await.ok();
        })
        .await
    });

    wait_for_shutdown().await;
    info!("收到停机信号");
    // 通知所有子系统收尾
    let _ = shutdown_tx.send(true);

    // 等待 TCP 服务结束
    match tokio::time::timeout(Duration::from_secs(5), server_task).await {
        Ok(Ok(res)) => res?,
        Ok(Err(e)) => tracing::warn!(?e, "server task join  error"),
        Err(_) => tracing::warn!("server 结束超时"),
    }

    info!("iot-gateway 已停机");
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
