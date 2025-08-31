use crate::types::Review;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

/// SpFreshIndex: wrapper (replace append/search with real binding)
pub struct SpFreshIndex {
    index_path: String,
    writer_lock: Mutex<()>,
}

impl SpFreshIndex {
    pub fn open(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            File::create(path).context("create spfresh index file")?;
        }
        Ok(Self {
            index_path: path.to_string(),
            writer_lock: Mutex::new(()),
        })
    }

    /// Append vector and return a deterministic vector id (u64).
    /// Replace body with real spfresh append that returns actual id.
    pub fn append_vector(&self, vector: &[f32]) -> Result<u64> {
        let _g = self.writer_lock.lock();
        let meta = std::fs::metadata(&self.index_path)?;
        let id = meta.len() / 16 + 1; // simplistic pseudo-id
        let mut f = OpenOptions::new().append(true).open(&self.index_path)?;
        writeln!(f, "VEC_ID:{} LEN:{}", id, vector.len()).context("write index placeholder")?;
        Ok(id as u64)
    }

    /// Search returning (vector_id, score).
    /// Replace with real knn query.
    pub fn search(&self, _vector: &[f32], top_k: usize) -> Result<Vec<(u64, f32)>> {
        let mut res = Vec::new();
        for i in 1..=top_k {
            res.push((i as u64, 1.0 / (i as f32)));
        }
        Ok(res)
    }
}

/// Append review metadata (JSONL) append-only
pub fn append_metadata(jsonl_path: &str, review: &Review) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(jsonl_path)
        .with_context(|| format!("open metadata file: {}", jsonl_path))?;
    let mut writer = BufWriter::new(file);
    let line = serde_json::to_string(&review)?;
    writer.write_all(line.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

/// Append vector_id -> review_id mapping (JSONL) append-only
#[derive(serde::Serialize)]
struct VectorMapRecord<'a> {
    vector_id: u64,
    review_id: &'a str,
}
pub fn append_vector_map(map_path: &str, vector_id: u64, review_id: &str) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(map_path)
        .with_context(|| format!("open vector_map file: {}", map_path))?;
    let mut writer = BufWriter::new(file);
    let rec = VectorMapRecord { vector_id, review_id };
    let line = serde_json::to_string(&rec)?;
    writer.write_all(line.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

/// Read mapping file into Vec<(vector_id, review_id)> keeping order
pub fn read_vector_map(map_path: &str) -> Result<Vec<(u64, String)>> {
    if !Path::new(map_path).exists() {
        return Ok(Vec::new());
    }
    let file = File::open(map_path)?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let l = line?;
        if l.trim().is_empty() { continue; }
        let v: serde_json::Value = serde_json::from_str(&l)?;
        let vid = v["vector_id"].as_u64().ok_or_else(|| anyhow::anyhow!("invalid map"))?;
        let rid = v["review_id"].as_str().unwrap_or_default().to_string();
        out.push((vid, rid));
    }
    Ok(out)
}

/// Read metadata by review_id quickly by streaming (naive)
pub fn read_metadata_by_review_ids(jsonl_path: &str, review_ids: &[String]) -> Result<Vec<Review>> {
    let mut map = std::collections::HashMap::new();
    // early exit if file missing
    if !Path::new(jsonl_path).exists() {
        return Ok(Vec::new());
    }
    let file = File::open(jsonl_path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line?;
        if l.trim().is_empty() { continue; }
        let r: Review = serde_json::from_str(&l)?;
        map.insert(r.id.clone(), r);
    }
    let mut out = Vec::new();
    for rid in review_ids {
        if let Some(r) = map.get(rid) {
            out.push(r.clone());
        }
    }
    Ok(out)
}