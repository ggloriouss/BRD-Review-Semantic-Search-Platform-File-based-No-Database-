use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{
    insert_review_handler,
    bulk_insert_handler,
    health_handler,
    search_handler,
    AppState,
};

pub fn register_routes(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/reviews", post(insert_review_handler))
        .route("/reviews/bulk", post(bulk_insert_handler))
        .route("/search", post(search_handler))
        .layer(cors)
        .with_state(state)
}