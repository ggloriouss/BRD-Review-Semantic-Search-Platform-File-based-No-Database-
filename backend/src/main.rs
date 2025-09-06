
mod embedder;
mod handlers;
mod routes;
mod storage;
mod types;

use axum::{serve, Router};
use handlers::AppState;
use std::{env, net::SocketAddr, path::PathBuf, sync::{Arc, RwLock}};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Data/index paths
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".into());
    std::fs::create_dir_all(&data_dir)?;
    let index_path = env::var("INDEX_FILE").unwrap_or_else(|_| format!("{}/reviews.index", data_dir));
    let jsonl_path = env::var("METADATA_FILE").unwrap_or_else(|_| format!("{}/reviews.jsonl", data_dir));
    let map_path = env::var("MAP_FILE").unwrap_or_else(|_| format!("{}/vector_map.jsonl", data_dir));

    let index = storage::SpFreshIndex::open(&index_path)?;
    let state = AppState {
        index: Arc::new(RwLock::new(index)),
        jsonl_path: jsonl_path.clone(),
        map_path: map_path.clone(),
    };

    // Optional integrity check (warning only)
    {
        let idx = state.index.read().expect("index RwLock poisoned");
        if let Err(e) = storage::verify_alignment(&*idx, &state.jsonl_path) {
            tracing::warn!("Alignment check: {}", e);
        }
    }

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Static site (Trunk dist)
    let site_root_env = env::var("FRONTEND_DIST").unwrap_or_else(|_| "../frontend/dist".into());
    let site_root = PathBuf::from(site_root_env);
    let pkg_dir = site_root.join("pkg");
    let index_html = site_root.join("index.html");

    if !index_html.exists() {
        tracing::warn!(
            "index.html not found at {:?}. Did you run `trunk build --release`?",
            index_html
        );
    }

    let static_site = Router::new()
        .nest_service("/pkg", ServeDir::new(&pkg_dir))
        .nest_service("/assets", ServeDir::new(site_root.join("assets")))
        .nest_service("/", ServeDir::new(&site_root))
        .fallback_service(ServeFile::new(&index_html))
        .layer(cors.clone())
        .with_state(state.clone());

    // API routes
    let api = routes::register_routes(state.clone());

    // Compose: API + Static site
    let app = api.merge(static_site);

    // Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on http://0.0.0.0:8000");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}