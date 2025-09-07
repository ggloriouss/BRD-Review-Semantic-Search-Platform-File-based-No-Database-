use crate::embedder::Embedder;
use crate::storage::{
    append_review_line, append_vector_map_line, load_all_reviews, SpFreshIndex,
};
use crate::types::{
    BulkReviews, ReviewInput, SearchRequest, SearchResponse, StoredReview, SearchHit,
};
use axum::{extract::State, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::error;

// ---- Health ----
pub async fn health_handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

// ---- Runtime-configurable paths ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paths {
    pub index_path: String,
    pub jsonl_path: String,
    pub map_path: String,
}

#[derive(Clone)]
pub struct AppState {
    pub index: Arc<RwLock<SpFreshIndex>>,
    pub paths: Arc<RwLock<Paths>>,
}

// GET /api/config/paths
pub async fn get_paths_handler(State(state): State<AppState>) -> Result<Json<Paths>, (StatusCode, String)> {
    let p = state.paths.read().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;
    Ok(Json(p.clone()))
}

// POST /api/config/paths  { index_path, jsonl_path, map_path }
pub async fn set_paths_handler(
    State(state): State<AppState>,
    Json(newp): Json<Paths>,
) -> Result<Json<Paths>, (StatusCode, String)> {
    // Create parent dirs if needed
    for path in [&newp.index_path, &newp.jsonl_path, &newp.map_path] {
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir).map_err(|e| (StatusCode::BAD_REQUEST, format!("create dir failed: {e}")))?;
        }
    }

    // Open (or create) index at new location
    let new_index = crate::storage::SpFreshIndex::open(&newp.index_path)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("open index failed: {e}")))?;

    // Swap index atomically
    {
        let mut idx_guard = state.index.write().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?;
        *idx_guard = new_index;
    }

    // Update paths atomically
    {
        let mut p = state.paths.write().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;
        *p = newp.clone();
    }

    Ok(Json(newp))
}

// ---- Insert one ----
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

    let vector_id = state
        .index
        .write()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?
        .append_vector(&vec)
        .map_err(|e| {
            error!("index append error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "index append failed".to_string())
        })?;

    let stored = StoredReview::from_input(payload, vector_id);

    let p = state.paths.read().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;

    append_review_line(&p.jsonl_path, &stored).map_err(|e| {
        error!("write metadata error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "metadata write failed".to_string())
    })?;

    append_vector_map_line(&p.map_path, vector_id, &stored.id).map_err(|e| {
        error!("write vector_map error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "vector_map write failed".to_string())
    })?;

    Ok(Json(stored))
}

// ---- Bulk insert ----
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
    if vectors.len() != texts.len() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Embedding count mismatch".into()));
    }

    let p = state.paths.read().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;

    let stored_all: Result<Vec<StoredReview>, (StatusCode, String)> = items
        .into_iter()
        .zip(vectors.into_iter())
        .map(|(input, vec)| {
            let vector_id = state
                .index
                .write()
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?
                .append_vector(&vec)
                .map_err(|e| {
                    error!("index append error: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "index append failed".to_string())
                })?;

            let stored = StoredReview::from_input(input, vector_id);

            append_review_line(&p.jsonl_path, &stored).map_err(|e| {
                error!("write metadata error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "metadata write failed".to_string())
            })?;

            append_vector_map_line(&p.map_path, vector_id, &stored.id).map_err(|e| {
                error!("write vector_map error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "vector_map write failed".to_string())
            })?;

            Ok(stored)
        })
        .collect();

    Ok(Json(stored_all?))
}

// ---- Search ----
pub async fn search_handler(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    if req.query.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "query empty".into()));
    }

    const TOP_N: usize = 5;

    let embedder = Embedder::get().map_err(|e| {
        error!("embedder init error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding init failed".to_string())
    })?;

    let qvec = embedder.embed_one(&req.query).map_err(|e| {
        error!("embed query failed: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding failed".to_string())
    })?;

    let ann_k = req.top_k.unwrap_or(TOP_N).max(TOP_N).min(200);

    let hits = state
        .index
        .read()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?
        .search(&qvec, ann_k);

    let p = state.paths.read().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;
    let reviews = load_all_reviews(&p.jsonl_path).map_err(|e| {
        error!("read metadata error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "read metadata failed".to_string())
    })?;

    let by_vec: HashMap<usize, &crate::types::StoredReview> =
        reviews.iter().map(|r| (r.vector_id, r)).collect();

    let mut out: Vec<SearchHit> = hits
        .into_iter()
        .filter_map(|(vid, score)| {
            by_vec.get(&vid).map(|r| SearchHit {
                review: (*r).clone(),
                score,
            })
        })
        .collect();

    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    if out.len() > TOP_N {
        out.truncate(TOP_N);
    }

    Ok(Json(SearchResponse { hits: out }))
}
