mod embedder;
mod handlers;
mod routes;
mod storage;
mod types;

use axum::{serve, Router};
use handlers::{AppState, Paths};
use std::{
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
};
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

    // -------- Initial file paths (env or defaults) --------
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|| "data".into());
    std::fs::create_dir_all(&data_dir)?;
    let index_path = env::var("INDEX_FILE").unwrap_or_else(|| format!("{}/reviews.index", &data_dir));
    let jsonl_path = env::var("METADATA_FILE").unwrap_or_else(|| format!("{}/reviews.jsonl", &data_dir));
    let map_path = env::var("MAP_FILE").unwrap_or_else(|| format!("{}/vector_map.jsonl", &data_dir));

    // Open index with current path
    let index = storage::SpFreshIndex::open(&index_path)?;

    // Shared state: index + runtime-changeable paths
    let state = AppState {
        index: Arc::new(RwLock::new(index)),
        paths: Arc::new(RwLock::new(Paths {
            index_path,
            jsonl_path,
            map_path,
        })),
    };

    // Optional integrity check (warning only)
    {
        let idx = state.index.read().expect("index RwLock poisoned");
        let p = state.paths.read().expect("paths RwLock poisoned");
        if let Err(e) = storage::verify_alignment(&*idx, &p.jsonl_path) {
            tracing::warn!("Alignment check: {}", e);
        }
    }

    // -------- CORS --------
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // -------- Static site (Trunk dist) --------
    let site_root_env = env::var("FRONTEND_DIST").unwrap_or_else(|| "../frontend/dist".into());
    let site_root = PathBuf::from(site_root_env);
    let pkg_dir = site_root.join("pkg");
    let index_html = site_root.join("index.html");

    if !index_html.exists() {
        tracing::warn!(
            "index.html not found at {:?}. Did you run `trunk build --release`?",
            index_html
        );
    }

    let static_site: Router<AppState> = Router::new()
        .nest_service("/pkg", ServeDir::new(&pkg_dir))
        .nest_service("/assets", ServeDir::new(site_root.join("assets")))
        .nest_service("/", ServeDir::new(&site_root))
        .fallback_service(ServeFile::new(&index_html))
        .layer(cors.clone())
        .with_state(state.clone());

    // -------- API routes --------
    let api = routes::register_routes(state.clone());

    // Compose: API + Static site
    let app = api.merge(static_site);

    // -------- Server --------
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on http://0.0.0.0:8000");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}
