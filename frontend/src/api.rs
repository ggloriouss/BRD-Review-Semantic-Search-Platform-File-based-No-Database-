use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use gloo_net::http::Request;

const BASE_URL: &str = "http://localhost:8000";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReviewInput {
    pub review: String,
    pub rating: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StoredReview {
    pub id: String,
    pub review: String,
    pub rating: i32,
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

fn backend_url() -> String {
    option_env!("BACKEND_URL")
        .unwrap_or(BASE_URL)
        .to_string()
}

pub async fn create_review(r: &ReviewInput) -> Result<StoredReview, JsValue> {
    let res = Request::post(&format!("{}/reviews", backend_url()))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(r).unwrap())
        .unwrap()
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() {
        return Err(JsValue::from_str(&format!("Error: {}", res.status())));
    }
    Ok(res
        .json::<StoredReview>()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?)
}

pub async fn create_bulk(rs: &[ReviewInput]) -> Result<Vec<StoredReview>, JsValue> {
    let payload = serde_json::to_string(&rs).unwrap();
    let res = Request::post(&format!("{}/reviews/bulk", backend_url()))
        .header("Content-Type", "application/json")
        .body(payload)
        .unwrap()
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() {
        return Err(JsValue::from_str(&format!("Error: {}", res.status())));
    }
    Ok(res
        .json::<Vec<StoredReview>>()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?)
}

pub async fn search(req: &SearchRequest) -> Result<SearchResponse, JsValue> {
    let res = Request::post(&format!("{}/search", backend_url()))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(req).unwrap())
        .unwrap()
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if !res.ok() {
        return Err(JsValue::from_str(&format!("Error: {}", res.status())));
    }
    Ok(res
        .json::<SearchResponse>()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?)
}