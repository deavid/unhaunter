mod config;
mod hub_comm;
mod manager;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "unprocman=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config_path = "procman_config.ron";
    let config = config::load_config(config_path).await?;
    let manager = Arc::new(manager::ServerManager::new(config));

    manager.scan_library().await?;

    let manager_clone = manager.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = manager_clone.maintain_pool().await {
                tracing::error!("Error maintaining pool: {}", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    #[cfg(unix)]
    {
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            use tokio::signal::unix::{SignalKind, signal};

            let mut sighup = match signal(SignalKind::hangup()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to register SIGHUP handler: {}", e);
                    return;
                }
            };

            loop {
                if sighup.recv().await.is_none() {
                    tracing::warn!("SIGHUP stream ended unexpectedly");
                    break;
                }
                tracing::info!("Received SIGHUP, rescanning library");
                if let Err(e) = manager_clone.scan_library().await {
                    tracing::error!("Library rescan failed after SIGHUP: {}", e);
                }
            }
        });
    }

    hub_comm::run_hub_comm(manager).await?;

    Ok(())
}
