use anyhow::Result;

const EMBEDDING_DIM: usize = 256;
const MULTIPLIER: u64 = 1099511628211;
const INITIAL_SEED: u64 = 1469598103934665603;

pub fn embed_text(text: &str) -> Result<Vec<f32>> {
    let seed = calculate_seed(text);
    let vector = generate_vector(seed);
    let normalized = normalize_vector(vector);
    Ok(normalized)
}

fn calculate_seed(text: &str) -> u64 {
    text.bytes().fold(INITIAL_SEED, |seed, byte| {
        seed.wrapping_mul(MULTIPLIER).wrapping_add(byte as u64)
    })
}

fn generate_vector(seed: u64) -> Vec<f32> {
    (0..EMBEDDING_DIM)
        .map(|i| ((seed.wrapping_add(i as u64) % 1000) as f32) / 1000.0)
        .collect()
}

fn normalize_vector(mut vector: Vec<f32>) -> Vec<f32> {
    let norm = (vector.iter().map(|x| x * x).sum::<f32>()).sqrt();
    if norm > 0.0 {
        vector.iter_mut().for_each(|x| *x /= norm);
    }
    vector
}