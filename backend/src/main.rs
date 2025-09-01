mod embedder;
mod handlers;
mod routes;
mod storage;
mod types;

use tracing_subscriber::prelude::*;
use axum::{serve};
// use axum::{serve, Router}; // Add serve, remove hyper::Server
use handlers::AppState;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    std::fs::create_dir_all("data")?;
    let index_path = "data/reviews.index";
    let jsonl_path = "data/reviews.jsonl";
    let map_path = "data/vector_map.jsonl";

    let index = storage::SpFreshIndex::open(index_path)?;
    let state = AppState {
        index: Arc::new(index),
        jsonl_path: jsonl_path.to_string(),
        map_path: map_path.to_string(),
    };

    let app = routes::register_routes(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}