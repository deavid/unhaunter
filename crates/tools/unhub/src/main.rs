mod api;
mod config;
mod procman;
mod state;
mod tickets;

use crate::state::HubState;
use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
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
    let state = HubState::new(config);

    // Start ProcMan listener
    let procman_addr: SocketAddr = "0.0.0.0:11000".parse()?;
    let procman_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = procman::run_procman_listener(procman_state, procman_addr).await {
            tracing::error!("ProcMan listener error: {}", e);
        }
    });

    // REST API
    let app = Router::new()
        .route("/health", get(api::health))
        .route("/v1/challenge", post(api::challenge))
        .route("/v1/rooms/create", post(api::create_room))
        .route("/v1/rooms/join/{code}", post(api::join_room))
        .with_state(state);

    let api_addr: SocketAddr = "0.0.0.0:3000".parse()?;
    tracing::info!("Hub API listening on {}", api_addr);
    let listener = tokio::net::TcpListener::bind(api_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
