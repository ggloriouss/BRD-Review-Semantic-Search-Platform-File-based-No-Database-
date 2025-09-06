use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct Embedder {
    inner: TextEmbedding,
}

static EMBEDDER_SINGLETON: Lazy<Mutex<Option<Arc<Embedder>>>> =
    Lazy::new(|| Mutex::new(None));

impl Embedder {
    pub fn get() -> Result<Arc<Embedder>> {
        {
            let guard = EMBEDDER_SINGLETON.lock();
            if let Some(existing) = guard.as_ref() {
                return Ok(existing.clone());
            }
        }
        let model = TextEmbedding::try_new(InitOptions::new(
            EmbeddingModel::AllMiniLML6V2,
        ))?;
        let embedder = Arc::new(Embedder { inner: model });
        *EMBEDDER_SINGLETON.lock() = Some(embedder.clone());
        Ok(embedder)
    }

    pub fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.inner.embed(texts.to_vec(), None)?;
        Ok(embeddings)
    }

    pub fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        Ok(self.embed(&[text.to_string()])?.remove(0))
    }
}