use crate::embedder::Embedder;
use crate::storage::{append_review_line, append_vector_map_line, load_all_reviews};
use crate::types::{BulkReviews, ReviewInput, SearchRequest, SearchResponse, StoredReview, SearchHit};

// ใช้ Spfresh (FFI) แทน SpFreshIndex เดิม
use crate::spfresh::Spfresh;

use axum::{extract::State, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::{error, warn};

/// อ่านจำนวนบรรทัดของไฟล์ (ใช้เมื่อเปลี่ยน paths ใหม่ เพื่อรีเซ็ต next_vector_id)
fn count_lines(path: &str) -> usize {
    let p = Path::new(path);
    if !p.exists() {
        return 0;
    }
    match File::open(p).map(BufReader::new) {
        Ok(reader) => reader.lines().count(),
        Err(_) => 0,
    }
}

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
    // เปลี่ยนชนิดเป็น Spfresh (FFI) + RwLock
    pub index: Arc<RwLock<Spfresh>>,
    pub paths: Arc<RwLock<Paths>>,
    // ตัวนับ ID เพื่อส่งให้ SPFresh (ตรงกับ vector_map.jsonl)
    pub next_vector_id: Arc<RwLock<usize>>,
}

// GET /api/config/paths
pub async fn get_paths_handler(
    State(state): State<AppState>
) -> Result<Json<Paths>, (StatusCode, String)> {
    let p = state
        .paths
        .read()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;
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
            std::fs::create_dir_all(dir)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("create dir failed: {e}")))?;
        }
    }

    // อ่าน ENV สำหรับเปิด SPFresh
    let embed_dim: usize = std::env::var("EMBED_DIM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(384);
    let spf_params =
        std::env::var("SPFRESH_PARAMS").unwrap_or_else(|_| "PostingPageLimit=12".into());

    // ใช้ "ไดเรกทอรี" ของ index (parent ของไฟล์ reviews.index) เพื่อเปิด SPFresh
    let index_dir = std::path::Path::new(&newp.index_path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_string_lossy()
        .to_string();

    // Open (or create) index at new location (SPFresh)
    let new_index = Spfresh::open(&index_dir, embed_dim, &spf_params)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("open SPFresh failed: {e}")))?;

    // Update next_vector_id จากไฟล์ map ใหม่
    let new_next_id = count_lines(&newp.map_path);

    // Swap index atomically
    {
        let mut idx_guard = state
            .index
            .write()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?;
        *idx_guard = new_index;
    }

    // Update paths atomically
    {
        let mut p = state
            .paths
            .write()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;
        *p = newp.clone();
    }

    // Update next_vector_id atomically
    {
        let mut idg = state
            .next_vector_id
            .write()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "id lock poisoned".into()))?;
        *idg = new_next_id;
    }

    Ok(Json(newp))
}

// ---- Insert one ----
pub async fn insert_review_handler(
    State(state): State<AppState>,
    Json(payload): Json<ReviewInput>,
) -> Result<Json<StoredReview>, (StatusCode, String)> {
    payload
        .validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let embedder = Embedder::get().map_err(|e| {
        error!("embedder init error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding init failed".to_string(),
        )
    })?;

    let vec = embedder.embed_one(&payload.review).map_err(|e| {
        error!("embed error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding failed".to_string(),
        )
    })?;

    // จอง vector_id หนึ่งค่า
    let vector_id: usize = {
        let mut g = state
            .next_vector_id
            .write()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "id lock poisoned".into()))?;
        let id = *g;
        *g += 1;
        id
    };

    // เพิ่มเวกเตอร์ลง SPFresh โดยส่ง id ชัดเจน
    {
        let idx = state
            .index
            .write()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?;
        // add_batch ต้องการ buffer ต่อเนื่องความยาว dim (n=1)
        idx.add_batch(&vec, Some(&[vector_id as i64]))
            .map_err(|e| {
                error!("index add_batch error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "index append failed".to_string(),
                )
            })?;
        // (ถ้าต้อง save/snapshot อาจเรียก idx.save()? แล้วแต่ flow)
    }

    let stored = StoredReview::from_input(payload, vector_id);

    let p = state
        .paths
        .read()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;

    append_review_line(&p.jsonl_path, &stored).map_err(|e| {
        error!("write metadata error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "metadata write failed".to_string(),
        )
    })?;

    append_vector_map_line(&p.map_path, vector_id, &stored.id).map_err(|e| {
        error!("write vector_map error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "vector_map write failed".to_string(),
        )
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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding init failed".to_string(),
        )
    })?;

    let texts: Vec<String> = items.iter().map(|p| p.review.clone()).collect();
    let vectors = embedder.embed(&texts).map_err(|e| {
        error!("embed error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding failed".to_string(),
        )
    })?;
    if vectors.len() != texts.len() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Embedding count mismatch".into(),
        ));
    }

    // เตรียม ids ต่อเนื่องตามจำนวนรายการ
    let (start_id, n) = {
        let mut g = state
            .next_vector_id
            .write()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "id lock poisoned".into()))?;
        let start = *g;
        let n = vectors.len();
        *g += n;
        (start, n)
    };
    let ids: Vec<i64> = (start_id..start_id + n).map(|x| x as i64).collect();

    // flatten vectors เป็น buffer ต่อเนื่อง [n * dim]
    let dim = vectors.get(0).map(|v| v.len()).unwrap_or(0);
    if dim == 0 {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Empty embedding dimension".into(),
        ));
    }
    let mut flat: Vec<f32> = Vec::with_capacity(n * dim);
    for v in &vectors {
        if v.len() != dim {
            warn!(
                "Inconsistent embedding dim: expected {}, got {}",
                dim,
                v.len()
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Inconsistent embedding dimension".into(),
            ));
        }
        flat.extend_from_slice(v);
    }

    // เพิ่มทั้งหมดทีเดียว
    {
        let idx = state
            .index
            .write()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?;
        idx.add_batch(&flat, Some(&ids)).map_err(|e| {
            error!("index add_batch error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "index append failed".to_string(),
            )
        })?;
        // (ถ้าต้อง save/snapshot อาจเรียก idx.save()? แล้วแต่ flow)
    }

    let p = state
        .paths
        .read()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;

    // เขียน metadata และ map ตามลำดับ
    let mut out = Vec::with_capacity(n);
    for ((input, _vec), vid_i64) in items.into_iter().zip(vectors.into_iter()).zip(ids.into_iter())
    {
        let vector_id: usize = usize::try_from(vid_i64).unwrap_or(0);
        let stored = StoredReview::from_input(input, vector_id);

        append_review_line(&p.jsonl_path, &stored).map_err(|e| {
            error!("write metadata error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "metadata write failed".to_string(),
            )
        })?;

        append_vector_map_line(&p.map_path, vector_id, &stored.id).map_err(|e| {
            error!("write vector_map error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "vector_map write failed".to_string(),
            )
        })?;

        out.push(stored);
    }

    Ok(Json(out))
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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding init failed".to_string(),
        )
    })?;

    let qvec = embedder.embed_one(&req.query).map_err(|e| {
        error!("embed query failed: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding failed".to_string(),
        )
    })?;

    let ann_k = req.top_k.unwrap_or(TOP_N).max(TOP_N).min(200);

    // เรียกค้นหา: ได้ (ids, scores)
    let (ids, scores) = state
        .index
        .read()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "index lock poisoned".into()))?
        .search(&qvec, ann_k)
        .map_err(|e| {
            error!("index search error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "index search failed".to_string(),
            )
        })?;

    // โหลด metadata ทั้งหมด (ยังคงใช้ไฟล์ jsonl เดิม)
    let p = state
        .paths
        .read()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "paths lock poisoned".into()))?;
    let reviews = load_all_reviews(&p.jsonl_path).map_err(|e| {
        error!("read metadata error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "read metadata failed".to_string(),
        )
    })?;

    // map vector_id -> review
    let by_vec: HashMap<usize, &crate::types::StoredReview> =
        reviews.iter().map(|r| (r.vector_id, r)).collect();

    // รวมผลลัพธ์
    let mut out: Vec<SearchHit> = ids
        .into_iter()
        .zip(scores.into_iter())
        .filter_map(|(vid_i64, score)| {
            let vid: usize = usize::try_from(vid_i64).ok()?;
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
