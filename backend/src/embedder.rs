use anyhow::Result;

/// Placeholder embedding function.
/// Replace this with fastembed-rs call in production.
pub fn embed_text(text: &str) -> Result<Vec<f32>> {
    let dim = 256usize;
    let mut v = vec![0.0f32; dim];
    let mut seed: u64 = 1469598103934665603;
    for b in text.bytes() {
        seed = seed.wrapping_mul(1099511628211).wrapping_add(b as u64);
    }
    for i in 0..dim {
        let val = ((seed.wrapping_add(i as u64) % 1000) as f32) / 1000.0;
        v[i] = val;
    }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() { *x /= norm; }
    }
    Ok(v)
}