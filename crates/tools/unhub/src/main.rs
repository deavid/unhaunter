mod api;
mod config;
mod procman;
mod state;

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

    let state = HubState::new(config);

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
