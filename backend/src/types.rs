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
    /// Optional review category (letters, numbers, space, '-' or '_', max 32 chars).
    /// Missing or empty-after-trim is treated as `None`.
    #[serde(default)]
    pub category: Option<String>,
}

impl ReviewInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.review.trim().is_empty() {
            return Err("review cannot be empty".into());
        }
        if self.rating < 0 || self.rating > 5 {
            return Err("rating must be 0..=5".into());
        }
        if let Some(raw) = &self.category {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err("category cannot be empty when provided".into());
            }
            if trimmed.len() > 32 {
                return Err("category must be at most 32 characters".into());
            }
            let ok = trimmed
                .chars()
                .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_');
            if !ok {
                return Err("category may only contain letters, numbers, spaces, '-' or '_'".into());
            }
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
    /// Stored optional category; absent in older rows.
    #[serde(default)]
    pub category: Option<String>,
    pub schema_version: String,
    pub vector_id: usize,
}

impl StoredReview {
    pub fn from_input(input: ReviewInput, vector_id: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            review: input.review,
            rating: input.rating,
            // normalize category to trimmed Some(...) or None
            category: input.category.and_then(|c| {
                let t = c.trim().to_string();
                if t.is_empty() { None } else { Some(t) }
            }),
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