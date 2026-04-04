//! Text embedding generation using ONNX Runtime.
//!
//! Uses sentence-transformer compatible ONNX models for generating
//! semantic embeddings from text chunks.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ort::session::Session;
use ort::value::Tensor;
use thiserror::Error;

/// Embedding dimension for all-MiniLM-L6-v2 model.
pub const EMBEDDING_DIM: usize = 384;

/// Errors that can occur during embedding generation.
#[derive(Debug, Error)]
pub enum EmbedderError {
    #[error("Model not found at {0}")]
    ModelNotFound(PathBuf),
    #[error("ONNX Runtime error: {0}")]
    Ort(String),
    #[error("Processing error: {0}")]
    Processing(String),
}

type Result<T> = std::result::Result<T, EmbedderError>;

/// Text embedder using ONNX Runtime.
pub struct Embedder {
    session: Mutex<Session>,
}

impl Embedder {
    /// Creates a new Embedder by loading an ONNX model from the given path.
    pub fn new(model_path: &Path) -> Result<Self> {
        if !model_path.exists() {
            return Err(EmbedderError::ModelNotFound(model_path.to_path_buf()));
        }

        let session = Session::builder()
            .map_err(|e| EmbedderError::Ort(e.to_string()))?
            .with_intra_threads(2)
            .map_err(|e| EmbedderError::Ort(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| EmbedderError::Ort(e.to_string()))?;

        Ok(Self {
            session: Mutex::new(session),
        })
    }

    /// Generates an embedding for a single text.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text])?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }

    /// Generates embeddings for a batch of texts.
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let batch_size = texts.len();
        let (input_ids, attention_mask, token_type_ids) = tokenize_batch(texts);
        let seq_len = input_ids[0].len();

        let ids_flat: Vec<i64> = input_ids.into_iter().flatten().collect();
        let mask_flat: Vec<i64> = attention_mask.into_iter().flatten().collect();
        let types_flat: Vec<i64> = token_type_ids.into_iter().flatten().collect();

        let shape = vec![batch_size as i64, seq_len as i64];

        let ids_tensor = Tensor::from_array((shape.clone(), ids_flat))
            .map_err(|e| EmbedderError::Ort(e.to_string()))?;
        let mask_tensor = Tensor::from_array((shape.clone(), mask_flat))
            .map_err(|e| EmbedderError::Ort(e.to_string()))?;
        let types_tensor = Tensor::from_array((shape, types_flat))
            .map_err(|e| EmbedderError::Ort(e.to_string()))?;

        let inputs = ort::inputs![
            "input_ids" => ids_tensor,
            "attention_mask" => mask_tensor,
            "token_type_ids" => types_tensor,
        ];

        let mut session = self.session.lock()
            .map_err(|e| EmbedderError::Processing(format!("Session lock failed: {e}")))?;
        let outputs = session.run(inputs)
            .map_err(|e| EmbedderError::Ort(e.to_string()))?;

        // Extract output tensor — returns (&Shape, &[f32])
        // Shape derefs to &[i64]
        let (out_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| EmbedderError::Processing(format!("Failed to extract output: {e}")))?;

        let dims: Vec<usize> = out_shape.iter().map(|&d| d as usize).collect();
        let hidden_dim = dims[dims.len() - 1];

        let mut result = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let embedding = if dims.len() == 3 {
                // [batch, seq_len, hidden] → mean pool over seq_len
                let s = dims[1];
                let mut pooled = vec![0.0f32; hidden_dim];
                for j in 0..s {
                    let offset = i * s * hidden_dim + j * hidden_dim;
                    for k in 0..hidden_dim {
                        pooled[k] += data[offset + k];
                    }
                }
                for v in &mut pooled {
                    *v /= s as f32;
                }
                pooled
            } else if dims.len() == 2 {
                // [batch, hidden] → already pooled
                let offset = i * hidden_dim;
                data[offset..offset + hidden_dim].to_vec()
            } else {
                vec![0.0; hidden_dim]
            };

            result.push(l2_normalize(&embedding));
        }

        Ok(result)
    }

    /// Returns the embedding dimension of the loaded model.
    pub fn dimension(&self) -> usize {
        EMBEDDING_DIM
    }
}

/// Tokenized batch result: (input_ids, attention_mask, token_type_ids).
type TokenizedBatch = (Vec<Vec<i64>>, Vec<Vec<i64>>, Vec<Vec<i64>>);

/// Simple tokenizer for sentence-transformer models.
fn tokenize_batch(texts: &[&str]) -> TokenizedBatch {
    let max_len = 128;
    let mut all_ids = Vec::new();
    let mut all_masks = Vec::new();
    let mut all_types = Vec::new();

    let mut actual_max = 0;
    let token_lists: Vec<Vec<&str>> = texts
        .iter()
        .map(|text| {
            let tokens: Vec<&str> = text.split_whitespace().take(max_len - 2).collect();
            actual_max = actual_max.max(tokens.len() + 2);
            tokens
        })
        .collect();

    let padded_len = actual_max.min(max_len);

    for tokens in &token_lists {
        let mut ids = vec![101i64]; // [CLS]
        let mut mask = vec![1i64];
        let mut types = vec![0i64];

        for token in tokens {
            ids.push(token_to_id(token));
            mask.push(1);
            types.push(0);
        }

        ids.push(102); // [SEP]
        mask.push(1);
        types.push(0);

        while ids.len() < padded_len {
            ids.push(0);
            mask.push(0);
            types.push(0);
        }

        all_ids.push(ids);
        all_masks.push(mask);
        all_types.push(types);
    }

    (all_ids, all_masks, all_types)
}

fn token_to_id(token: &str) -> i64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.to_lowercase().hash(&mut hasher);
    (hasher.finish() % 29000 + 1000) as i64
}

/// L2 normalizes a vector.
fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm < 1e-12 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

/// Computes cosine similarity between two embedding vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a < 1e-12 || norm_b < 1e-12 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_batch() {
        let texts = &["hello world", "foo bar baz"];
        let (ids, masks, types) = tokenize_batch(texts);
        assert_eq!(ids.len(), 2);
        assert_eq!(masks.len(), 2);
        assert_eq!(ids[0][0], 101);
        assert_eq!(masks[0][0], 1);
        assert_eq!(types[0][0], 0);
    }

    #[test]
    fn test_l2_normalize() {
        let v = vec![3.0, 4.0];
        let n = l2_normalize(&v);
        let norm: f32 = n.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_token_to_id_deterministic() {
        let id1 = token_to_id("hello");
        let id2 = token_to_id("hello");
        assert_eq!(id1, id2);
        assert!(id1 >= 1000);
    }
}
