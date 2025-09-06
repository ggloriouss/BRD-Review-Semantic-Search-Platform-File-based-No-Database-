// Purpose:
// Frontend HTTP client helpers for communicating with the backend API.
// - Encapsulates request/response types used by the UI.
// - Provides create_review, create_bulk, search, and health functions.
// Callers:
// - components::insert_review::InsertReview -> create_review, create_bulk
// - components::search::Search -> search
// - Other UI code may import StoredReview / SearchResponse for rendering.

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

/// If you need to point to a different origin (e.g. https://api.example.com),
/// build with: RUSTFLAGS='--cfg=web_sys_unstable_apis' BACKEND_URL=https://api.example.com trunk build
/// Otherwise we default to same-origin "/api".
fn api_base() -> String {
    option_env!("BACKEND_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/api".to_string())
}

/// ReviewInput: payload sent to backend when inserting a review.
/// Used by InsertReview component (single and bulk insert).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReviewInput {
    pub review: String,
    pub rating: i32,
}

/// StoredReview: representation of a review returned from the backend.
/// Rendered by the Search component and shown after inserting a review.
#[derive(Deserialize, Debug, Clone)]
pub struct StoredReview {
    pub id: String,
    pub review: String,
    pub rating: i32,
    #[allow(dead_code)]
    pub schema_version: String,
    pub vector_id: usize,
}

/// SearchRequest: body format for search queries.
/// Constructed by the Search component.
#[derive(Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

/// SearchHit: single search result with the StoredReview and score.
/// Returned by the backend and consumed by the Search component.
#[derive(Deserialize, Debug, Clone)]
pub struct SearchHit {
    pub review: StoredReview,
    pub score: f32,
}

/// SearchResponse: overall search response containing hits.
#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

/// GET /api/health
pub async fn health() -> Result<String, JsValue> {
    let url = format!("{}/health", api_base());
    let res = Request::get(&url)
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() {
        return Err(JsValue::from_str(&format!("Error: {}", res.status())));
    }
    res.text()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// POST /api/reviews
pub async fn create_review(r: &ReviewInput) -> Result<StoredReview, JsValue> {
    let url = format!("{}/reviews", api_base());
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(r).unwrap())
        .unwrap()
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if !res.ok() {
        return Err(JsValue::from_str(&format!("Error: {}", res.status())));
    }

    res.json::<StoredReview>()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// POST /api/reviews/bulk
/// Payload: JSON array of { review, rating }
pub async fn create_bulk(rs: &[ReviewInput]) -> Result<Vec<StoredReview>, JsValue> {
    let url = format!("{}/reviews/bulk", api_base());
    let payload = serde_json::to_string(&rs).unwrap();
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(payload)
        .unwrap()
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if !res.ok() {
        return Err(JsValue::from_str(&format!("Error: {}", res.status())));
    }

    res.json::<Vec<StoredReview>>()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// POST /api/search
pub async fn search(req: &SearchRequest) -> Result<SearchResponse, JsValue> {
    let url = format!("{}/search", api_base());
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(req).unwrap())
        .unwrap()
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if !res.ok() {
        return Err(JsValue::from_str(&format!("Error: {}", res.status())));
    }

    res.json::<SearchResponse>()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
