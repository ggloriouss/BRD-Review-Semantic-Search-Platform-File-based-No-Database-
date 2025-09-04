mod embedder;
mod handlers;
mod routes;
mod storage;
mod types;

use axum::serve;
use handlers::AppState;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configurable paths (Addtional implementation #02)
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".into());
    std::fs::create_dir_all(&data_dir)?;
    let index_path = env::var("INDEX_FILE").unwrap_or_else(|_| format!("{}/reviews.index", data_dir));
    let jsonl_path = env::var("METADATA_FILE").unwrap_or_else(|_| format!("{}/reviews.jsonl", data_dir));
    let map_path = env::var("MAP_FILE").unwrap_or_else(|_| format!("{}/vector_map.jsonl", data_dir));

    let index = storage::SpFreshIndex::open(&index_path)?;
    let state = AppState {
        index: Arc::new(index),
        jsonl_path: jsonl_path.clone(),
        map_path: map_path.clone(),
    };

    // Optional integrity check (warning only)
    if let Err(e) = storage::verify_alignment(&state.index, &state.jsonl_path) {
        tracing::warn!("Alignment check: {}", e);
    }

    let app = routes::register_routes(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}