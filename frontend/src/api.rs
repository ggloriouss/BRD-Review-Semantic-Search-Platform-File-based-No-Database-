use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

// Base URL
fn api_base() -> String {
    option_env!("BACKEND_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/api".to_string())
}

// ----- Types already present -----
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReviewInput {
    pub review: String,
    pub rating: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StoredReview {
    pub id: String,
    pub review: String,
    pub rating: i32,
    #[serde(default)]
    pub category: Option<String>,
    #[allow(dead_code)]
    pub schema_version: String,
    pub vector_id: usize,
}

#[derive(Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SearchHit {
    pub review: StoredReview,
    pub score: f32,
}

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

// ----- New: runtime path config -----
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Paths {
    pub index_path: String,
    pub jsonl_path: String,
    pub map_path: String,
}

// GET /api/health
pub async fn health() -> Result<String, JsValue> {
    let url = format!("{}/health", api_base());
    let res = Request::get(&url).send().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() { return Err(JsValue::from_str(&format!("Error: {}", res.status()))); }
    res.text().await.map_err(|e| JsValue::from_str(&e.to_string()))
}

// POST /api/reviews
pub async fn create_review(r: &ReviewInput) -> Result<StoredReview, JsValue> {
    let url = format!("{}/reviews", api_base());
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(r).unwrap()).unwrap()
        .send().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() { return Err(JsValue::from_str(&format!("Error: {}", res.status()))); }
    res.json::<StoredReview>().await.map_err(|e| JsValue::from_str(&e.to_string()))
}

// POST /api/reviews/bulk
pub async fn create_bulk(rs: &[ReviewInput]) -> Result<Vec<StoredReview>, JsValue> {
    let url = format!("{}/reviews/bulk", api_base());
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&rs).unwrap()).unwrap()
        .send().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() { return Err(JsValue::from_str(&format!("Error: {}", res.status()))); }
    res.json::<Vec<StoredReview>>().await.map_err(|e| JsValue::from_str(&e.to_string()))
}

// POST /api/search
pub async fn search(req: &SearchRequest) -> Result<SearchResponse, JsValue> {
    let url = format!("{}/search", api_base());
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(req).unwrap()).unwrap()
        .send().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() { return Err(JsValue::from_str(&format!("Error: {}", res.status()))); }
    res.json::<SearchResponse>().await.map_err(|e| JsValue::from_str(&e.to_string()))
}

// ----- New: GET /api/config/paths -----
pub async fn get_paths() -> Result<Paths, JsValue> {
    let url = format!("{}/config/paths", api_base());
    let res = Request::get(&url).send().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() { return Err(JsValue::from_str(&format!("Error: {}", res.status()))); }
    res.json::<Paths>().await.map_err(|e| JsValue::from_str(&e.to_string()))
}

// ----- New: POST /api/config/paths -----
pub async fn set_paths(p: &Paths) -> Result<Paths, JsValue> {
    let url = format!("{}/config/paths", api_base());
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(p).unwrap()).unwrap()
        .send().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() { return Err(JsValue::from_str(&format!("Error: {}", res.status()))); }
    res.json::<Paths>().await.map_err(|e| JsValue::from_str(&e.to_string()))
}
