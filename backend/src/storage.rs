use anyhow::Result;
use serde_json::Deserializer;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;

use crate::types::StoredReview;

/// เขียน 1 บรรทัดของรีวิวลงไฟล์ JSONL
pub fn append_review_line(path: &str, review: &StoredReview) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(review)? + "\n";
    file.write_all(line.as_bytes())?;
    Ok(())
}

/// โหลดรีวิวทั้งหมดจากไฟล์ JSONL
pub fn load_all_reviews(path: &str) -> Result<Vec<StoredReview>> {
    if !Path::new(path).exists() {
        return Ok(vec![]);
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let stream = Deserializer::from_reader(reader).into_iter::<StoredReview>();
    let mut out = Vec::new();
    for item in stream {
        out.push(item?);
    }
    Ok(out)
}

/// เขียน mapping (vector_id → review_id) 1 บรรทัด
pub fn append_vector_map_line(path: &str, vector_id: usize, review_id: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line =
        serde_json::json!({ "vector_id": vector_id, "review_id": review_id }).to_string() + "\n";
    file.write_all(line.as_bytes())?;
    Ok(())
}
