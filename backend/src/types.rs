use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ReviewId = String;

/// Central schema version for metadata evolution.
pub const SCHEMA_VERSION: &str = "v1";

/// Input struct (single review). Extendable: add new optional fields here.
/// Validation rules implemented in `impl ReviewInput { validate() }`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReviewInput {
    pub review: String,
    pub rating: i32,
}

impl ReviewInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.review.trim().is_empty() {
            return Err("review cannot be empty".into());
        }
        if self.rating < 0 || self.rating > 5 {
            return Err("rating must be 0..=5".into());
        }
        Ok(())
    }
}

/// Internal stored representation (append-only).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredReview {
    pub id: ReviewId,
    pub review: String,
    pub rating: i32,
    pub schema_version: String,
    pub vector_id: usize,
}

impl StoredReview {
    pub fn from_input(input: ReviewInput, vector_id: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            review: input.review,
            rating: input.rating,
            schema_version: SCHEMA_VERSION.to_string(),
            vector_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BulkReviews(pub Vec<ReviewInput>);

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub review: StoredReview,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}