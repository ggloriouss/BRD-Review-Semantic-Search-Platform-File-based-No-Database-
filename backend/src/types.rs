use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Review {
    pub id: String,
    pub title: Option<String>,
    pub body: String,
    pub rating: Option<u8>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertReviewRequest {
    pub title: Option<String>,
    pub body: String,
    pub rating: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertReviewResponse {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkInsertRequest {
    pub reviews: Vec<InsertReviewRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Review,
}