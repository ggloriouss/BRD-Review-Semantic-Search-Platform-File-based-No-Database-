use crate::embedder;
use crate::storage::{append_metadata, append_vector_map, read_metadata_by_review_ids, SpFreshIndex};
use crate::types::{BulkInsertRequest, InsertReviewRequest, InsertReviewResponse, Review, SearchRequest, SearchResult};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct AppState {
    pub index: Arc<SpFreshIndex>,
    pub jsonl_path: String,
    pub map_path: String,
}

async fn process_single_insert(
    state: &AppState,
    payload: &InsertReviewRequest,
) -> Result<String, (StatusCode, String)> {
    if payload.body.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "body empty".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let review = Review {
        id: id.clone(),
        title: payload.title.clone(),
        body: payload.body.clone(),
        rating: payload.rating,
        created_at: now,
    };

    // build embed input
    let embed_input = format!("{} {}", review.title.clone().unwrap_or_default(), review.body);
    let vector = embedder::embed_text(&embed_input).map_err(|e| {
        error!("embed error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding failed".to_string())
    })?;

    debug!("vector len {}", vector.len());
    let vector_id = state.index.append_vector(&vector).map_err(|e| {
        error!("index append error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "index append failed".to_string())
    })?;
    info!(vec_id = vector_id, "vector appended");

    append_metadata(&state.jsonl_path, &review).map_err(|e| {
        error!("write metadata error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "metadata write failed".to_string())
    })?;

    append_vector_map(&state.map_path, vector_id, &id).map_err(|e| {
        error!("write vector_map error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "vector_map write failed".to_string())
    })?;

    Ok(id)
}

/// POST /reviews
pub async fn insert_review_handler(
    State(state): State<AppState>,
    Json(payload): Json<InsertReviewRequest>,
) -> Result<(StatusCode, Json<InsertReviewResponse>), (StatusCode, String)> {
    let id = process_single_insert(&state, &payload).await?;
    Ok((StatusCode::CREATED, Json(InsertReviewResponse { id })))
}

/// POST /reviews/bulk
pub async fn bulk_insert_handler(
    State(state): State<AppState>,
    Json(payload): Json<BulkInsertRequest>,
) -> Result<(StatusCode, Json<Vec<InsertReviewResponse>>), (StatusCode, String)> {
    if payload.reviews.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "no reviews provided".into()));
    }
    // process sequentially; in production consider parallel/batched approach
    let mut out = Vec::new();
    for r in &payload.reviews {
        match process_single_insert(&state, r).await {
            Ok(id) => out.push(InsertReviewResponse { id }),
            Err((code, msg)) => {
                error!("bulk insert - item failed: {}", msg);
                // return partial failure? Here we abort whole operation
                return Err((code, msg));
            }
        }
    }
    Ok((StatusCode::CREATED, Json(out)))
}

/// POST /search
pub async fn search_handler(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> Result<(StatusCode, Json<Vec<SearchResult>>), (StatusCode, String)> {
    if payload.query.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "query empty".into()));
    }
    let top_k = payload.top_k.unwrap_or(10);
    let qvec = embedder::embed_text(&payload.query).map_err(|e| {
        error!("embed query failed: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "embedding failed".to_string())
    })?;

    let neighbors = state.index.search(&qvec, top_k).map_err(|e| {
        error!("search error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "search failed".to_string())
    })?;

    // neighbors: Vec<(vector_id, score)> -> load vector_map to map to review_ids
    let map = crate::storage::read_vector_map(&state.map_path).map_err(|e| {
        error!("read map failed: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "vector map read failed".to_string())
    })?;

    // build vector_id -> review_id map
    let id_map: std::collections::HashMap<u64, String> =
        map.into_iter().map(|(v, r)| (v, r)).collect();

    // produce ordered review_ids as per neighbors
    let mut review_ids: Vec<String> = Vec::new();
    let mut scores_map: std::collections::HashMap<u64, f32> = std::collections::HashMap::new();
    for (vid, score) in &neighbors {
        if let Some(rid) = id_map.get(vid) {
            review_ids.push(rid.clone());
            scores_map.insert(*vid, *score);
        } else {
            // missing mapping -> skip
            debug!("missing mapping for vector id {}", vid);
        }
    }

    // fetch metadata for those review_ids
    let metas = read_metadata_by_review_ids(&state.jsonl_path, &review_ids).map_err(|e| {
        error!("read metadata error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "read metadata failed".to_string())
    })?;

    // build result preserving neighbors order
    let mut results = Vec::new();
    for (i, (vid, score)) in neighbors.into_iter().enumerate() {
        if let Some(rid) = id_map.get(&vid) {
            // find meta by id
            if let Some(meta) = metas.iter().find(|m| &m.id == rid) {
                results.push(SearchResult {
                    id: meta.id.clone(),
                    score,
                    metadata: meta.clone(),
                });
            } else {
                debug!("metadata missing for review id {}", rid);
            }
        } else {
            debug!("no review id for vector id {}", vid);
        }
    }

    Ok((StatusCode::OK, Json(results)))
}