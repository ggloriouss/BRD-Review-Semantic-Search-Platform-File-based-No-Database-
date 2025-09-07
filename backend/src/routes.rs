use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{
    AppState,
    bulk_insert_handler,
    get_paths_handler,
    health_handler,
    insert_review_handler,
    search_handler,
    set_paths_handler,
};

pub fn register_routes(state: AppState) -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    Router::new()
        .with_state(state)
        // health
        .route("/health", get(health_handler))
        // config
        .route("/config/paths", get(get_paths_handler).post(set_paths_handler))
        // data
        .route("/reviews", post(insert_review_handler))
        .route("/reviews/bulk", post(bulk_insert_handler))
        .route("/search", post(search_handler))
        .layer(cors)
}
