use axum::{routing::post, Router};
use crate::handlers::{bulk_insert_handler, insert_review_handler, search_handler, AppState};
use reqwest::get;
// use axum::routing::get;
use crate::handlers::health_handler;

pub fn register_routes(state: AppState) -> Router {
    Router::new()
        // .route("/health", get(health_handler)) 
        .route("/reviews", post(insert_review_handler))
        .route("/reviews/bulk", post(bulk_insert_handler))
        .route("/search", post(search_handler))
        .with_state(state)
}