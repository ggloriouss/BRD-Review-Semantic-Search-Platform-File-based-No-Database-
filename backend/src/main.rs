mod embedder;
mod handlers;
mod routes;
mod storage;
mod types;
mod spfresh;

use axum::Router;
use handlers::{AppState, Paths};
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing_subscriber::prelude::*;

// hyper / hyper-util (accept loop)
use hyper::{Request, body::Incoming};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;

// แปลง Incoming -> data stream ของ Bytes
use http_body_util::BodyExt as _;

// ใช้เรียก .call() บน service
use tower::Service;

fn count_lines(path: &str) -> usize {
    let p = Path::new(path);
    if !p.exists() {
        return 0;
    }
    match File::open(p).map(BufReader::new) {
        Ok(reader) => reader.lines().count(),
        Err(_) => 0,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // -------- Initial file paths --------
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".into());
    std::fs::create_dir_all(&data_dir)?;
    let index_path =
        env::var("INDEX_FILE").unwrap_or_else(|_| format!("{}/reviews.index", &data_dir));
    let jsonl_path =
        env::var("METADATA_FILE").unwrap_or_else(|_| format!("{}/reviews.jsonl", &data_dir));
    let map_path =
        env::var("MAP_FILE").unwrap_or_else(|_| format!("{}/vector_map.jsonl", &data_dir));

    // -------- Open SPFresh index (FFI) --------
    let embed_dim: usize = env::var("EMBED_DIM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(384);
    let spf_params = env::var("SPFRESH_PARAMS").unwrap_or_else(|_| "PostingPageLimit=12".into());
    let index = spfresh::Spfresh::open(&data_dir, embed_dim, &spf_params)?;

    let next_vector_id = count_lines(&map_path);

    let state = AppState {
        index: Arc::new(RwLock::new(index)),
        paths: Arc::new(RwLock::new(Paths {
            index_path,
            jsonl_path,
            map_path: map_path.clone(),
        })),
        next_vector_id: Arc::new(RwLock::new(next_vector_id)),
    };

    // -------- CORS --------
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // -------- Static site --------
    let site_root_env = env::var("FRONTEND_DIST").unwrap_or_else(|_| "../frontend/dist".into());
    let site_root = PathBuf::from(site_root_env);

    let pkg_dir_owned: PathBuf = site_root.join("pkg");
    let assets_dir_owned: PathBuf = site_root.join("assets");
    let index_html_owned: PathBuf = site_root.join("index.html");

    if !index_html_owned.exists() {
        tracing::warn!(
            "index.html not found at {:?}. Did you run `trunk build --release`?",
            index_html_owned
        );
    }

    // -------- Build Router<AppState> เดียว (รวม API + static) --------
    let app: Router<AppState> = routes::register_routes(state.clone())
        .nest_service("/pkg",    ServeDir::new(pkg_dir_owned.clone()))
        .nest_service("/assets", ServeDir::new(assets_dir_owned.clone()))
        .nest_service("/",       ServeDir::new(site_root.clone()))
        .fallback_service(ServeFile::new(index_html_owned.clone()))
        .layer(cors.clone());

    // -------- Server via hyper-util accept loop --------
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on http://0.0.0.0:8000");
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // เตรียม state ไว้ใส่ลง request.extensions
    let svc_state = state.clone();

    loop {
        let (stream, peer) = listener.accept().await?;
        tracing::debug!("accepted connection from {:?}", peer);

        let app_clone = app.clone();
        let svc_state = svc_state.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);

            // Hyper Service: Request<Incoming> -> Response<axum::body::Body>
            let hyper_service = hyper::service::service_fn(move |req: Request<Incoming>| {
                let app2 = app_clone.clone();
                let svc_state = svc_state.clone();

                async move {
                    // 1) แตก parts + body (Incoming)
                    let (parts, incoming) = req.into_parts();

                    // 2) แปลง Incoming -> stream ของ Bytes
                    let data_stream = incoming.into_data_stream();

                    // 3) ประกอบ Request<axum::body::Body> ใหม่
                    let body = axum::body::Body::from_stream(data_stream);
                    let mut req2 = axum::http::Request::from_parts(parts, body);

                    // 4) ฝัง AppState เข้า request.extensions เพื่อให้ State extractor ใช้ได้
                    req2.extensions_mut().insert(svc_state.clone());

                    // 5) ทำให้ Router กลายเป็น service แบบ state = ()
                    //    (ตรงกับ impl ที่มีอยู่: RouterIntoService<_, ()>: Service<Request<_>>)
                    let mut svc = app2.clone().with_state::<()>(svc_state.clone()).into_service();

                    // 6) เรียก .call() ได้เลย
                    let resp = Service::call(&mut svc, req2).await.unwrap();

                    Ok::<_, std::convert::Infallible>(resp)
                }
            });

            let res = AutoBuilder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(io, hyper_service)
                .await;

            if let Err(err) = res {
                tracing::error!("serve connection error: {err}");
            }
        });
    }
}
