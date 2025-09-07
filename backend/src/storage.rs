use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde_json::Deserializer;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use crate::types::StoredReview;

pub struct SpFreshIndex {
    path: PathBuf,
    vectors: RwLock<Vec<Vec<f32>>>,
}

unsafe impl Send for SpFreshIndex {}
unsafe impl Sync for SpFreshIndex {}

impl SpFreshIndex {
    pub fn open<P: AsRef<Path>>(p: P) -> Result<Self> {
        let path = p.as_ref().to_path_buf();
        if !path.exists() {
            File::create(&path)?;
        }
        let mut vectors = Vec::new();
        {
            use std::io::Read;
            let mut f = File::open(&path)?;
            loop {
                let mut len_buf = [0u8; 4];
                if f.read_exact(&mut len_buf).is_err() {
                    break;
                }
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut fb = vec![0u8; len * 4];
                f.read_exact(&mut fb)?;
                let mut vecf = Vec::with_capacity(len);
                for chunk in fb.chunks_exact(4) {
                    vecf.push(f32::from_le_bytes(chunk.try_into().unwrap()));
                }
                vectors.push(vecf);
            }
        }
        Ok(Self {
            path,
            vectors: RwLock::new(vectors),
        })
    }

    pub fn append_vector(&self, vector: &[f32]) -> Result<usize> {
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        let len = vector.len() as u32;
        file.write_all(&len.to_le_bytes())?;
        for v in vector {
            file.write_all(&v.to_le_bytes())?;
        }
        let mut guard = self.vectors.write();
        guard.push(vector.to_vec());
        Ok(guard.len() - 1)
    }

    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<(usize, f32)> {
        let guard = self.vectors.read();
        let mut scored: Vec<(usize, f32)> = guard
            .iter()
            .enumerate()
            .map(|(i, v)| (i, cosine_similarity(query, v)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn len(&self) -> usize {
        self.vectors.read().len()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0f32;
    let mut na = 0f32;
    let mut nb = 0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

pub fn append_review_line(path: &str, review: &StoredReview) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(review)? + "\n";
    file.write_all(line.as_bytes())?;
    Ok(())
}

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

pub fn append_vector_map_line(path: &str, vector_id: usize, review_id: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line =
        serde_json::json!({"vector_id": vector_id, "review_id": review_id }).to_string() + "\n";
    file.write_all(line.as_bytes())?;
    Ok(())
}

pub fn verify_alignment(index: &SpFreshIndex, jsonl_path: &str) -> Result<()> {
    let reviews = load_all_reviews(jsonl_path)?;
    if reviews.len() != index.len() {
        return Err(anyhow!(
            "Mismatch: vectors={} metadata_lines={}",
            index.len(),
            reviews.len()
        ));
    }
    Ok(())
}