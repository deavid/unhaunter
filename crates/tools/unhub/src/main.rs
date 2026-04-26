mod api;
mod config;
mod procman;
mod state;
mod stats;

use crate::state::HubState;
use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "unhub=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config_path = "hub_config.ron";
    let config = config::load_config(config_path).await?;

    let api_bind = config.api_bind.clone();
    let procman_bind = config.procman_bind.clone();
    let stats_log_path = config.stats_log_path.clone().map(std::path::PathBuf::from);

    let state = HubState::new(config);

    // Write a boot line to the stats log immediately.
    if let Some(ref path) = stats_log_path {
        stats::write_stats("boot", &state, path);
    }

    // Spawn hourly stats task.
    if let Some(path) = stats_log_path.clone() {
        let stats_state = state.clone();
        tokio::spawn(async move {
            // First tick fires immediately; skip it so we don't log twice on boot.
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(3600));
            interval.tick().await;
            loop {
                interval.tick().await;
                stats::write_stats("hourly", &stats_state, &path);
            }
        });
    }

    // Start ProcMan listener
    let procman_addr: SocketAddr = procman_bind.parse()?;
    let procman_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = procman::run_procman_listener(procman_state, procman_addr).await {
            tracing::error!("ProcMan listener error: {}", e);
        }
    });

    // REST API
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/v1/ping", post(api::ping))
        .route("/v1/challenge", post(api::challenge))
        .route("/v1/rooms/create", post(api::create_room))
        .route("/v1/rooms/join/{code}", post(api::join_room))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let api_addr: SocketAddr = api_bind.parse()?;
    tracing::info!("Hub API listening on {}", api_addr);
    let listener = tokio::net::TcpListener::bind(api_addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
