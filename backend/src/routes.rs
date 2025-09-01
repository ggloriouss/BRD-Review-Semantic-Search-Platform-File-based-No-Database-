use axum::{routing::post, Router};
use crate::handlers::{bulk_insert_handler, insert_review_handler, search_handler, AppState};

pub fn register_routes(state: AppState) -> Router {
    Router::new()
        .route("/reviews", post(insert_review_handler))
        .route("/reviews/bulk", post(bulk_insert_handler))
        .route("/search", post(search_handler))
        .with_state(state)
}