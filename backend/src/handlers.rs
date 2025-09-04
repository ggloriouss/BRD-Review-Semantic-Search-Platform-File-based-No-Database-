use crate::embedder::Embedder;
use crate::storage::{
    append_review_line, append_vector_map_line, load_all_reviews, SpFreshIndex,
};
use crate::types::{
    BulkReviews, ReviewInput, SearchRequest, SearchResponse, StoredReview, SearchHit,
};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use std::sync::Arc;
use tracing::{error};

pub async fn health_handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

#[derive(Clone)]
pub struct AppState {
    pub index: Arc<SpFreshIndex>,
    pub jsonl_path: String,
    pub map_path: String,
}

pub async fn insert_review_handler(
    State(state): State<AppState>,
    Json(payload): Json<ReviewInput>,
) -> Result<Json<StoredReview>, (StatusCode, String)> {
    payload.validate().map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let embedder = Embedder::get().map_err(|e| {
        error!("embedder init error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding init failed".to_string())
    })?;
    let vec = embedder.embed_one(&payload.review).map_err(|e| {
        error!("embed error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding failed".to_string())
    })?;
    let vector_id = state.index.append_vector(&vec).map_err(|e| {
        error!("index append error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "index append failed".to_string())
    })?;
    let stored = StoredReview::from_input(payload, vector_id);
    append_review_line(&state.jsonl_path, &stored).map_err(|e| {
        error!("write metadata error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "metadata write failed".to_string())
    })?;
    append_vector_map_line(&state.map_path, vector_id, &stored.id).map_err(|e| {
        error!("write vector_map error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "vector_map write failed".to_string())
    })?;
    Ok(Json(stored))
}

pub async fn bulk_insert_handler(
    State(state): State<AppState>,
    Json(BulkReviews(items)): Json<BulkReviews>,
) -> Result<Json<Vec<StoredReview>>, (StatusCode, String)> {
    if items.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Bulk list empty".into()));
    }
    for r in &items {
        r.validate().map_err(|e| (StatusCode::BAD_REQUEST, e.clone()))?;
    }
    let embedder = Embedder::get().map_err(|e| {
        error!("embedder init error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding init failed".to_string())
    })?;
    let texts: Vec<String> = items.iter().map(|p| p.review.clone()).collect();
    let vectors = embedder.embed(&texts).map_err(|e| {
        error!("embed error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding failed".to_string())
    })?;
    if vectors.len() != items.len() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Embedding count mismatch".into()));
    }
    let mut stored_all = Vec::with_capacity(items.len());
    for (input, vec) in items.into_iter().zip(vectors.into_iter()) {
        let vector_id = state.index.append_vector(&vec).map_err(|e| {
            error!("index append error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "index append failed".to_string())
        })?;
        let stored = StoredReview::from_input(input, vector_id);
        append_review_line(&state.jsonl_path, &stored).map_err(|e| {
            error!("write metadata error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "metadata write failed".to_string())
        })?;
        append_vector_map_line(&state.map_path, vector_id, &stored.id).map_err(|e| {
            error!("write vector_map error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "vector_map write failed".to_string())
        })?;
        stored_all.push(stored);
    }
    Ok(Json(stored_all))
}

pub async fn search_handler(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    if req.query.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "query empty".into()));
    }
    let top_k = req.top_k.unwrap_or(10).min(200);
    let embedder = Embedder::get().map_err(|e| {
        error!("embedder init error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding init failed".to_string())
    })?;
    let qvec = embedder.embed_one(&req.query).map_err(|e| {
        error!("embed query failed: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding failed".to_string())
    })?;
    let hits = state.index.search(&qvec, top_k);
    let reviews = load_all_reviews(&state.jsonl_path).map_err(|e| {
        error!("read metadata error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "read metadata failed".to_string())
    })?;
    let mut out = Vec::new();
    for (vid, score) in hits {
        if let Some(r) = reviews.iter().find(|r| r.vector_id == vid) {
            out.push(SearchHit {
                review: r.clone(),
                score,
            });
        }
    }
    Ok(Json(SearchResponse { hits: out }))
}
