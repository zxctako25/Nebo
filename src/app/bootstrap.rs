use log::info;
use std::sync::Arc;
use tokio::signal;

use crate::app::console::start_console;
use crate::config::ProxyConfig;
use crate::monitor;
use crate::network::server::ProxyServer;

pub async fn run() -> anyhow::Result<()> {
    if monitor::run_if_monitor()? {
        return Ok(());
    }
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = ProxyConfig::load_or_create();
    let config = Arc::new(config);

    let server = ProxyServer::new(config.clone()).await?;
    let server_handle = server.start().await?;
    if let Err(error) = monitor::spawn(std::process::id()) {
        log::warn!("Could not open resource monitor window: {error}");
    }

    let server_for_console = server.clone();
    tokio::spawn(async move {
        start_console(server_for_console).await;
    });

    let server_for_shutdown = server.clone();
    tokio::spawn(async move {
        if let Ok(_) = signal::ctrl_c().await {
            info!("Shutdown signal received...");
            server_for_shutdown.stop().await;
        }
    });

    server_handle.await?;
    Ok(())
}
