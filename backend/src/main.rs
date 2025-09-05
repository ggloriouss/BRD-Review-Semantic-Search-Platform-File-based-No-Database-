mod embedder;
mod handlers;
mod routes;
mod storage;
mod types;

use axum::{serve, Router};
use handlers::AppState;
use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::prelude::*;


let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

// --- static site (Trunk build at ../frontend/dist) ---
let site_root = std::path::PathBuf::from("../frontend/dist");
let app = Router::new()
    // API lives under /api
    .nest("/api", routes::register_routes(state))
    // Static assets
    .nest_service("/pkg", ServeDir::new(site_root.join("pkg")))
    .nest_service("/assets", ServeDir::new(site_root.join("assets")))
    .nest_service("/", ServeDir::new(&site_root))
    // SPA fallback
    .fallback_service(ServeFile::new(site_root.join("index.html")))
    // Apply CORS to the whole app (covers preflight and 404s too)
    .layer(cors);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // -----------------------------
    // Data/index paths (your current setup)
    // -----------------------------
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".into());
    std::fs::create_dir_all(&data_dir)?;
    let index_path =
        env::var("INDEX_FILE").unwrap_or_else(|_| format!("{}/reviews.index", data_dir));
    let jsonl_path =
        env::var("METADATA_FILE").unwrap_or_else(|_| format!("{}/reviews.jsonl", data_dir));
    let map_path =
        env::var("MAP_FILE").unwrap_or_else(|_| format!("{}/vector_map.jsonl", data_dir));

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

    // -----------------------------
    // Static site serving (Leptos + Trunk)
    // -----------------------------
    // FRONTEND_DIST can override where we read the built site from.
    // By default, assume the binary runs from backend/ and the dist is at ../frontend/dist
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

    // Serve real files BEFORE SPA fallback so JS/WASM get correct MIME types
    let static_site = Router::new()
        // WASM/JS bundle from Trunk
        .nest_service("/pkg", ServeDir::new(&pkg_dir))
        // Optional assets dir if you have images/fonts (safe even if absent)
        .nest_service("/assets", ServeDir::new(site_root.join("assets")))
        // Serve everything else in dist
        .nest_service("/", ServeDir::new(&site_root))
        // SPA fallback: only when no static file matched
        .fallback_service(ServeFile::new(index_html));

    // -----------------------------
    // API routes (your existing router)
    // -----------------------------
    // Keep paths as-is (/health, /reviews, /search, etc.) so you don't need to
    // change the frontend API client. If you prefer, you can nest under "/api".
    let api = routes::register_routes(state);

    // Compose: API + Static site
    let app = api.merge(static_site);

    // -----------------------------
    // Server
    // -----------------------------
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on http://localhost:8000");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}