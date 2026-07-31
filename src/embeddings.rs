//! Embedder trait (Wave 1) and legacy embedding generators
//!
//! ## Wave 1 — real embedder interface ([`Embedder`])
//!
//! Pluggable batch embedding for future legitimate RAG. **Not RAG complete** —
//! vector store, hybrid rank, and eval remain Wave 2–3 (see `docs/ROADMAP.md`).
//!
//! - [`NullEmbedder`]: fail-closed stub (default semantic backend)
//! - [`HashingEmbedder`]: deterministic local vectors for tests only (`is_semantic() == false`)
//! - `HttpEmbedder` (feature `http-embedder`): OpenAI-compatible HTTP embeddings (`is_semantic() == true`)
//!   (intra-doc link omitted so `cargo doc` without that feature stays clean under `-D warnings`)
//!
//! C0 honesty: `enable_semantic` stays false by default. Semantic mode hard-errors
//! unless a real (`is_semantic()`) embedder is configured — never silently falls
//! back to hash pseudo-vectors.
//!
//! ## Legacy (quantization / tests)
//!
//! [`EmbeddingGenerator`] / [`MockEmbeddingGenerator`] / ternary wrappers remain for
//! quantization pipelines. Prefer [`Embedder`] for new code.

use crate::error::{ContextError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Dense embedding vector (f32 components).
pub type Vector = Vec<f32>;

/// Metadata recorded when content is embedded (ROADMAP C1.4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmbeddingInfo {
    /// Model identifier (e.g. `"text-embedding-3-small"`, `"hashing-v1"`)
    pub model: String,
    /// Vector dimensionality
    pub dims: usize,
    /// SHA-256 hex of the embedded content (staleness / cache key)
    pub content_hash: String,
}

/// SHA-256 hex digest of UTF-8 content (embedding cache / staleness key).
pub fn content_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Wave 1 pluggable embedder: batch of strings → dense vectors.
///
/// Target shape from `docs/ROADMAP.md` Library API.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Stable model identifier stored with each embedding.
    fn model_id(&self) -> &str;

    /// Output dimensionality.
    fn dims(&self) -> usize;

    /// `true` only for backends that produce real semantic vectors.
    ///
    /// Hash/mock backends must return `false`. Semantic retrieval requires `true`
    /// (fail closed otherwise).
    fn is_semantic(&self) -> bool;

    /// Embed a batch of texts. Output length must equal `texts.len()`;
    /// each vector length must equal `dims()`.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vector>>;

    /// Embed a single string.
    async fn embed_one(&self, text: &str) -> Result<Vector> {
        let mut batch = self.embed_batch(&[text]).await?;
        batch.pop().ok_or_else(|| {
            ContextError::Internal("embed_batch returned empty for single input".into())
        })
    }

    /// Build [`EmbeddingInfo`] for content (hash + model + dims). Does not embed.
    fn info_for(&self, text: &str) -> EmbeddingInfo {
        EmbeddingInfo {
            model: self.model_id().to_string(),
            dims: self.dims(),
            content_hash: content_hash(text),
        }
    }
}

/// Fail-closed stub: always errors. Use when no embedder is configured.
///
/// Semantic mode with this backend (or with no backend) must refuse, not
/// invent scores.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullEmbedder;

#[async_trait]
impl Embedder for NullEmbedder {
    fn model_id(&self) -> &str {
        "null"
    }

    fn dims(&self) -> usize {
        0
    }

    fn is_semantic(&self) -> bool {
        false
    }

    async fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Vector>> {
        Err(ContextError::EmbedderUnavailable(
            "NullEmbedder: no embedding backend configured (fail closed)".into(),
        ))
    }
}

/// Deterministic local hashing embedder for **tests and demos only**.
///
/// `is_semantic()` is always `false` — must not satisfy production semantic mode.
#[derive(Debug, Clone)]
pub struct HashingEmbedder {
    dimension: usize,
    model_id: String,
}

impl HashingEmbedder {
    /// Create with given output dimension (default model id `hashing-v1`).
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            model_id: "hashing-v1".into(),
        }
    }

    /// Override model id label (still non-semantic).
    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = model_id.into();
        self
    }

    fn hash_embed(&self, text: &str) -> Vector {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let value = ((hash.wrapping_mul(i as u64 + 1)) as f32) / (u64::MAX as f32);
            embedding.push(value);
        }

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }
        embedding
    }
}

#[async_trait]
impl Embedder for HashingEmbedder {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn dims(&self) -> usize {
        self.dimension
    }

    fn is_semantic(&self) -> bool {
        false
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vector>> {
        Ok(texts.iter().map(|t| self.hash_embed(t)).collect())
    }
}

/// OpenAI-compatible HTTP embeddings client (`POST {base}/embeddings`).
///
/// Feature-gated to avoid default network deps. `is_semantic() == true` —
/// suitable for semantic mode when a real remote model is configured.
///
/// **Honesty:** quality depends entirely on the remote model; this crate does
/// not claim MTEB or eval results until Wave 3 gates pass.
#[cfg(feature = "http-embedder")]
#[derive(Debug, Clone)]
pub struct HttpEmbedder {
    base_url: String,
    api_key: Option<String>,
    model: String,
    dims: usize,
    client: reqwest::Client,
}

#[cfg(feature = "http-embedder")]
impl HttpEmbedder {
    /// Create an HTTP embedder.
    ///
    /// `base_url` should be the API root (e.g. `https://api.openai.com/v1`).
    /// Path `/embeddings` is appended.
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        dims: usize,
        api_key: Option<String>,
    ) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| ContextError::Config(format!("http embedder client: {e}")))?;
        Ok(Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            model: model.into(),
            dims,
            client,
        })
    }
}

#[cfg(feature = "http-embedder")]
#[async_trait]
impl Embedder for HttpEmbedder {
    fn model_id(&self) -> &str {
        &self.model
    }

    fn dims(&self) -> usize {
        self.dims
    }

    fn is_semantic(&self) -> bool {
        true
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vector>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/embeddings", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "input": texts,
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| ContextError::EmbedderUnavailable(format!("HTTP embed request: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ContextError::EmbedderUnavailable(format!(
                "HTTP embed status {status}: {text}"
            )));
        }

        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ContextError::EmbedderUnavailable(format!("HTTP embed JSON: {e}")))?;

        let data = value
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| {
                ContextError::EmbedderUnavailable("HTTP embed response missing data[]".into())
            })?;

        // OpenAI returns objects with index; sort by index for stable order.
        let mut indexed: Vec<(usize, Vector)> = Vec::with_capacity(data.len());
        for item in data {
            let idx = item.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
            let emb = item
                .get("embedding")
                .and_then(|e| e.as_array())
                .ok_or_else(|| {
                    ContextError::EmbedderUnavailable("HTTP embed item missing embedding".into())
                })?;
            let vec: Vector = emb
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            if vec.len() != self.dims {
                return Err(ContextError::EmbedderUnavailable(format!(
                    "HTTP embed dims mismatch: got {} expected {}",
                    vec.len(),
                    self.dims
                )));
            }
            indexed.push((idx, vec));
        }
        indexed.sort_by_key(|(i, _)| *i);
        if indexed.len() != texts.len() {
            return Err(ContextError::EmbedderUnavailable(format!(
                "HTTP embed count mismatch: got {} expected {}",
                indexed.len(),
                texts.len()
            )));
        }
        Ok(indexed.into_iter().map(|(_, v)| v).collect())
    }
}

// ---------------------------------------------------------------------------
// Embedder selection (ROADMAP "Config (CLI / env)": --embedder / --embed-model)
// ---------------------------------------------------------------------------

/// Which embedder backend to construct (`--embedder none|local|http`).
///
/// Before this existed, [`crate::rag::RagProcessor::with_embedder`] had no
/// non-test caller: the server always took the `None` path, so no CLI or config
/// input could put a real embedder behind the MCP tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbedderKind {
    /// No embedder. Retrieval is metadata/temporal/keyword only. Semantic mode refuses.
    #[default]
    None,
    /// In-process local backend. **Currently non-semantic** ([`HashingEmbedder`]):
    /// ROADMAP C1.2 (local GGUF/ONNX/candle) is still open, so this cannot satisfy
    /// semantic mode and startup refuses if combined with `enable_semantic`.
    Local,
    /// OpenAI-compatible HTTP backend (`HttpEmbedder`, `is_semantic() == true`).
    /// Requires the `http-embedder` cargo feature, which is **not** in `default`.
    ///
    /// Plain code span rather than an intra-doc link: the type is `cfg`-gated, so a link
    /// would warn in the default build.
    Http,
}

impl EmbedderKind {
    /// Accepted CLI/config spellings.
    pub const VARIANTS: [&'static str; 3] = ["none", "local", "http"];

    /// Canonical lowercase name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Local => "local",
            Self::Http => "http",
        }
    }
}

impl std::fmt::Display for EmbedderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EmbedderKind {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" | "off" | "" => Ok(Self::None),
            "local" => Ok(Self::Local),
            "http" => Ok(Self::Http),
            other => Err(ContextError::Config(format!(
                "unknown embedder '{other}' (expected one of: {})",
                Self::VARIANTS.join(", ")
            ))),
        }
    }
}

/// Default dimensionality for the local (hashing) backend when `--embed-dims` is absent.
pub const DEFAULT_LOCAL_DIMS: usize = 384;

/// Declarative embedder selection, resolved into an [`Embedder`] by [`build_embedder`].
///
/// Plain data (no `Arc<dyn Embedder>`) so it can live in
/// [`crate::server::ServerConfig`] while keeping that type `Debug + Clone`.
#[derive(Clone, Default)]
pub struct EmbedderConfig {
    /// Backend to construct.
    pub kind: EmbedderKind,
    /// Model id / path (`--embed-model`). Required for `http`.
    pub model: Option<String>,
    /// Output dimensionality (`--embed-dims`). Required for `http`; local defaults
    /// to [`DEFAULT_LOCAL_DIMS`].
    pub dims: Option<usize>,
    /// API root for `http`, e.g. `https://api.openai.com/v1` (`--embed-base-url`).
    pub base_url: Option<String>,
    /// Bearer token for `http`. Sourced from the environment, never a CLI flag,
    /// so it does not land in `ps`/argv.
    pub api_key: Option<String>,
}

/// Redacts `api_key`: [`crate::server::ServerConfig`] is `Debug` and gets logged.
impl std::fmt::Debug for EmbedderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbedderConfig")
            .field("kind", &self.kind)
            .field("model", &self.model)
            .field("dims", &self.dims)
            .field("base_url", &self.base_url)
            .field(
                "api_key",
                &self
                    .api_key
                    .as_ref()
                    .map(|_| "<redacted>")
                    .unwrap_or("None"),
            )
            .finish()
    }
}

impl EmbedderConfig {
    /// Config that constructs no embedder.
    pub fn none() -> Self {
        Self::default()
    }

    /// Environment variable holding the HTTP embedder bearer token.
    pub const API_KEY_ENV: &'static str = "CONTEXT_MCP_EMBED_API_KEY";
}

/// Construct the selected embedder, or fail loudly.
///
/// Never substitutes a different backend for the one that was asked for: an
/// unavailable backend is an error naming exactly what is missing (cargo feature
/// or config value). A silent downgrade to a non-semantic embedder is the same
/// class of bug as an embedder that cannot be reached at all.
pub fn build_embedder(config: &EmbedderConfig) -> Result<Option<Arc<dyn Embedder>>> {
    match config.kind {
        EmbedderKind::None => Ok(None),

        EmbedderKind::Local => {
            let dims = config.dims.unwrap_or(DEFAULT_LOCAL_DIMS);
            if dims == 0 {
                return Err(ContextError::Config(
                    "--embed-dims must be greater than 0 for --embedder local".into(),
                ));
            }
            let mut embedder = HashingEmbedder::new(dims);
            if let Some(model) = &config.model {
                embedder = embedder.with_model_id(model.clone());
            }
            Ok(Some(Arc::new(embedder)))
        }

        EmbedderKind::Http => build_http_embedder(config),
    }
}

#[cfg(feature = "http-embedder")]
fn build_http_embedder(config: &EmbedderConfig) -> Result<Option<Arc<dyn Embedder>>> {
    let base_url = config.base_url.as_deref().ok_or_else(|| {
        ContextError::Config("--embedder http requires --embed-base-url (API root)".into())
    })?;
    let model = config.model.as_deref().ok_or_else(|| {
        ContextError::Config("--embedder http requires --embed-model (remote model id)".into())
    })?;
    let dims = config.dims.ok_or_else(|| {
        ContextError::Config(
            "--embedder http requires --embed-dims (vectors of the wrong length are rejected, \
             not truncated)"
                .into(),
        )
    })?;
    if dims == 0 {
        return Err(ContextError::Config(
            "--embed-dims must be greater than 0 for --embedder http".into(),
        ));
    }
    let embedder = HttpEmbedder::new(base_url, model, dims, config.api_key.clone())?;
    Ok(Some(Arc::new(embedder)))
}

#[cfg(not(feature = "http-embedder"))]
fn build_http_embedder(_config: &EmbedderConfig) -> Result<Option<Arc<dyn Embedder>>> {
    Err(ContextError::Config(
        "--embedder http requires the `http-embedder` cargo feature, which is NOT in the \
         default feature set (default = server, persistence, ternary-embeddings). This binary \
         was built without it, so no HTTP embedder exists to construct. Rebuild with: \
         cargo build --release --features http-embedder"
            .into(),
    ))
}

/// Startup gate: refuse to serve if semantic mode was requested but cannot be honoured.
///
/// Enforced when the server is constructed, not on the first `retrieve` call, so a
/// misconfiguration is a failed launch instead of a server that answers `tools/list`
/// and then errors on every retrieval.
pub fn ensure_semantic_capable(embedder: Option<&Arc<dyn Embedder>>) -> Result<()> {
    match embedder {
        Some(e) if e.is_semantic() => Ok(()),
        Some(e) => Err(ContextError::SemanticUnavailable(format!(
            "semantic mode requested but embedder '{}' reports is_semantic=false. \
             The local backend is a deterministic hashing stub (ROADMAP C1.2: a real local \
             GGUF/ONNX/candle backend is still open); it will not be silently used as a \
             stand-in for real embeddings. Use --embedder http with the `http-embedder` \
             cargo feature, or drop --enable-semantic.",
            e.model_id()
        ))),
        None => Err(ContextError::SemanticUnavailable(
            "semantic mode requested but no embedder is configured. \
             Select one with --embedder <none|local|http>; only `http` (cargo feature \
             `http-embedder`) currently produces semantic vectors."
                .into(),
        )),
    }
}

/// Apply an embedder to a context: sets embedding vector + model/dims/content_hash.
///
/// Works with any [`Embedder`] (including non-semantic hashing for tests).
/// Production semantic store should only call this with `is_semantic()` backends.
pub async fn apply_embedding(
    ctx: &mut crate::context::Context,
    embedder: &dyn Embedder,
) -> Result<()> {
    let info = embedder.info_for(&ctx.content);
    let vector = embedder.embed_one(&ctx.content).await?;
    if vector.len() != embedder.dims() && embedder.dims() > 0 {
        return Err(ContextError::Internal(format!(
            "embedder returned len {} but dims() is {}",
            vector.len(),
            embedder.dims()
        )));
    }
    ctx.embedding = Some(vector);
    ctx.embedding_model = Some(info.model);
    ctx.embedding_dims = Some(if embedder.dims() > 0 {
        embedder.dims()
    } else {
        ctx.embedding.as_ref().map(|v| v.len()).unwrap_or(0)
    });
    ctx.content_hash = Some(info.content_hash);
    Ok(())
}

// ---------------------------------------------------------------------------
// Legacy EmbeddingGenerator (quantization / Mock) — kept for ternary pipeline
// ---------------------------------------------------------------------------

/// Trait for generating embeddings from text (legacy single-string API).
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate an embedding vector from text
    async fn generate(&self, text: &str) -> Result<Vec<f32>>;

    /// Get the dimension of embeddings produced by this generator
    fn dimension(&self) -> usize;
}

/// Trait for quantized embeddings with reconstruction capability
#[async_trait]
pub trait QuantizedEmbeddingGenerator: Send + Sync {
    /// Generate a quantized embedding from text
    async fn generate_quantized(&self, text: &str) -> Result<QuantizedEmbedding>;

    /// Get the dimension of original embeddings
    fn dimension(&self) -> usize;

    /// Get the quantization strategy (e.g., "sparse", "rvq", "hybrid")
    fn strategy(&self) -> &str;

    /// Reconstruct the original embedding from quantized form
    async fn reconstruct(&self, quantized: &QuantizedEmbedding) -> Result<Vec<f32>>;
}

/// Quantized embedding representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantizedEmbedding {
    /// Sparse ternary embedding
    SparseTernary(crate::ternary::TernaryQuantizedEmbedding),
    /// Dense embedding (baseline)
    Dense(Vec<f32>),
}

impl QuantizedEmbedding {
    /// Get size in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::SparseTernary(sparse) => sparse.size_bytes(),
            Self::Dense(vec) => vec.len() * 4,
        }
    }
}

/// Mock embedding generator for testing and development (legacy API).
///
/// Prefer [`HashingEmbedder`] + [`Embedder`] for new code.
pub struct MockEmbeddingGenerator {
    dimension: usize,
}

impl MockEmbeddingGenerator {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingGenerator for MockEmbeddingGenerator {
    async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        // Delegate to HashingEmbedder for one implementation of the hash path.
        HashingEmbedder::new(self.dimension).embed_one(text).await
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Adapter: use any [`Embedder`] as a legacy [`EmbeddingGenerator`].
pub struct EmbedderAsGenerator {
    inner: Arc<dyn Embedder>,
}

impl EmbedderAsGenerator {
    pub fn new(inner: Arc<dyn Embedder>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl EmbeddingGenerator for EmbedderAsGenerator {
    async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        self.inner.embed_one(text).await
    }

    fn dimension(&self) -> usize {
        self.inner.dims()
    }
}

/// Ternary embedding generator with configurable quantization strategies
pub struct TernaryEmbeddingGeneratorWrapper {
    base_generator: Arc<dyn EmbeddingGenerator>,
    ternary_gen: Arc<crate::ternary::TernaryEmbeddingGenerator>,
}

impl TernaryEmbeddingGeneratorWrapper {
    /// Create with sparse ternary quantization
    pub fn with_sparse(
        base_generator: Arc<dyn EmbeddingGenerator>,
        config: crate::ternary::SparsityConfig,
    ) -> Self {
        let dimension = base_generator.dimension();
        let ternary_gen = Arc::new(crate::ternary::TernaryEmbeddingGenerator::with_sparse(
            dimension, config,
        ));

        Self {
            base_generator,
            ternary_gen,
        }
    }

    /// Create with RVQ quantization
    pub fn with_rvq(
        base_generator: Arc<dyn EmbeddingGenerator>,
        num_layers: usize,
        codebook_size: usize,
    ) -> Self {
        let dimension = base_generator.dimension();
        let ternary_gen = Arc::new(crate::ternary::TernaryEmbeddingGenerator::with_rvq(
            dimension,
            num_layers,
            codebook_size,
        ));

        Self {
            base_generator,
            ternary_gen,
        }
    }

    /// Create with hybrid quantization
    pub fn with_hybrid(
        base_generator: Arc<dyn EmbeddingGenerator>,
        sparse_config: crate::ternary::SparsityConfig,
        num_layers: usize,
        codebook_size: usize,
    ) -> Self {
        let dimension = base_generator.dimension();
        let ternary_gen = Arc::new(crate::ternary::TernaryEmbeddingGenerator::with_hybrid(
            dimension,
            sparse_config,
            num_layers,
            codebook_size,
        ));

        Self {
            base_generator,
            ternary_gen,
        }
    }
}

#[async_trait]
impl QuantizedEmbeddingGenerator for TernaryEmbeddingGeneratorWrapper {
    async fn generate_quantized(&self, text: &str) -> Result<QuantizedEmbedding> {
        let dense = self.base_generator.generate(text).await?;
        let quantized = self.ternary_gen.quantize(&dense)?;
        Ok(QuantizedEmbedding::SparseTernary(quantized))
    }

    fn dimension(&self) -> usize {
        self.base_generator.dimension()
    }

    fn strategy(&self) -> &str {
        &self.ternary_gen.strategy
    }

    async fn reconstruct(&self, quantized: &QuantizedEmbedding) -> Result<Vec<f32>> {
        match quantized {
            QuantizedEmbedding::SparseTernary(sparse) => self.ternary_gen.dequantize(sparse),
            QuantizedEmbedding::Dense(vec) => Ok(vec.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Result<Option<Arc<dyn Embedder>>>` has no `Debug` for the Ok side, so
    /// `unwrap_err()` is unavailable here.
    fn expect_build_err(result: Result<Option<Arc<dyn Embedder>>>) -> ContextError {
        match result {
            Ok(_) => panic!("expected build_embedder to fail"),
            Err(e) => e,
        }
    }

    #[test]
    fn test_embedder_kind_parsing() {
        use std::str::FromStr;
        assert_eq!(EmbedderKind::from_str("none").unwrap(), EmbedderKind::None);
        assert_eq!(
            EmbedderKind::from_str("LOCAL").unwrap(),
            EmbedderKind::Local
        );
        assert_eq!(
            EmbedderKind::from_str(" http ").unwrap(),
            EmbedderKind::Http
        );

        let err = EmbedderKind::from_str("fastembed").unwrap_err();
        assert!(matches!(err, ContextError::Config(_)));
        // The error must enumerate the real options, not just reject.
        assert!(err.to_string().contains("none, local, http"), "{err}");
    }

    #[test]
    fn test_build_embedder_none_is_none() {
        assert!(build_embedder(&EmbedderConfig::none()).unwrap().is_none());
    }

    #[test]
    fn test_build_embedder_local_is_non_semantic() {
        let cfg = EmbedderConfig {
            kind: EmbedderKind::Local,
            dims: Some(24),
            ..Default::default()
        };
        let embedder = build_embedder(&cfg).unwrap().expect("local embedder");
        assert_eq!(embedder.dims(), 24);
        // Honesty gate: the local stub must never claim semantic capability.
        assert!(!embedder.is_semantic());
    }

    #[test]
    fn test_build_embedder_local_default_dims() {
        let cfg = EmbedderConfig {
            kind: EmbedderKind::Local,
            ..Default::default()
        };
        let embedder = build_embedder(&cfg).unwrap().expect("local embedder");
        assert_eq!(embedder.dims(), DEFAULT_LOCAL_DIMS);
    }

    /// The whole point of failing loudly: `http` must not degrade to `local`.
    #[cfg(not(feature = "http-embedder"))]
    #[test]
    fn test_build_embedder_http_without_feature_errors_naming_feature() {
        let cfg = EmbedderConfig {
            kind: EmbedderKind::Http,
            model: Some("text-embedding-3-small".into()),
            dims: Some(1536),
            base_url: Some("https://example.invalid/v1".into()),
            api_key: None,
        };
        let err = expect_build_err(build_embedder(&cfg));
        assert!(err.to_string().contains("http-embedder"), "{err}");
        assert!(err.to_string().contains("cargo build"), "{err}");
    }

    #[cfg(feature = "http-embedder")]
    #[test]
    fn test_build_embedder_http_requires_its_parameters() {
        // Missing base_url
        let err = expect_build_err(build_embedder(&EmbedderConfig {
            kind: EmbedderKind::Http,
            model: Some("m".into()),
            dims: Some(8),
            ..Default::default()
        }));
        assert!(err.to_string().contains("--embed-base-url"), "{err}");

        // Missing model
        let err = expect_build_err(build_embedder(&EmbedderConfig {
            kind: EmbedderKind::Http,
            dims: Some(8),
            base_url: Some("https://example.invalid/v1".into()),
            ..Default::default()
        }));
        assert!(err.to_string().contains("--embed-model"), "{err}");

        // Missing dims
        let err = expect_build_err(build_embedder(&EmbedderConfig {
            kind: EmbedderKind::Http,
            model: Some("m".into()),
            base_url: Some("https://example.invalid/v1".into()),
            ..Default::default()
        }));
        assert!(err.to_string().contains("--embed-dims"), "{err}");
    }

    #[cfg(feature = "http-embedder")]
    #[test]
    fn test_build_embedder_http_is_semantic() {
        let cfg = EmbedderConfig {
            kind: EmbedderKind::Http,
            model: Some("text-embedding-3-small".into()),
            dims: Some(1536),
            base_url: Some("https://example.invalid/v1".into()),
            api_key: Some("secret".into()),
        };
        let embedder = build_embedder(&cfg).unwrap().expect("http embedder");
        assert!(embedder.is_semantic());
        assert_eq!(embedder.dims(), 1536);
        assert!(ensure_semantic_capable(Some(&embedder)).is_ok());
    }

    #[test]
    fn test_ensure_semantic_capable_rejects_none_and_non_semantic() {
        let err = ensure_semantic_capable(None).unwrap_err();
        assert!(matches!(err, ContextError::SemanticUnavailable(_)));

        let hashing: Arc<dyn Embedder> = Arc::new(HashingEmbedder::new(8));
        let err = ensure_semantic_capable(Some(&hashing)).unwrap_err();
        assert!(matches!(err, ContextError::SemanticUnavailable(_)));

        let null: Arc<dyn Embedder> = Arc::new(NullEmbedder);
        assert!(ensure_semantic_capable(Some(&null)).is_err());
    }

    /// An API key in the config must not print when ServerConfig is logged/Debug-formatted.
    #[test]
    fn test_embedder_config_debug_redacts_api_key() {
        let cfg = EmbedderConfig {
            kind: EmbedderKind::Http,
            model: Some("m".into()),
            dims: Some(8),
            base_url: Some("https://example.invalid/v1".into()),
            api_key: Some("sk-super-secret".into()),
        };
        let rendered = format!("{cfg:?}");
        assert!(!rendered.contains("sk-super-secret"), "{rendered}");
        assert!(rendered.contains("<redacted>"), "{rendered}");
    }

    #[tokio::test]
    async fn test_null_embedder_fail_closed() {
        let e = NullEmbedder;
        assert!(!e.is_semantic());
        assert_eq!(e.model_id(), "null");
        let err = e.embed_batch(&["hi"]).await.unwrap_err();
        assert!(matches!(err, ContextError::EmbedderUnavailable(_)));
    }

    #[tokio::test]
    async fn test_hashing_embedder_batch_deterministic() {
        let e = HashingEmbedder::new(64);
        assert!(!e.is_semantic());
        assert_eq!(e.dims(), 64);

        let batch = e.embed_batch(&["alpha", "beta"]).await.unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].len(), 64);
        assert_eq!(batch[1].len(), 64);

        let again = e.embed_one("alpha").await.unwrap();
        assert_eq!(batch[0], again);

        let norm: f32 = again.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_content_hash_stable() {
        let a = content_hash("hello");
        let b = content_hash("hello");
        let c = content_hash("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64); // sha256 hex
    }

    #[tokio::test]
    async fn test_apply_embedding_stores_metadata() {
        use crate::context::{Context, ContextDomain};

        let mut ctx = Context::new("store me", ContextDomain::General);
        let emb = HashingEmbedder::new(32);
        apply_embedding(&mut ctx, &emb).await.unwrap();

        assert!(ctx.embedding.is_some());
        assert_eq!(ctx.embedding_model.as_deref(), Some("hashing-v1"));
        assert_eq!(ctx.embedding_dims, Some(32));
        assert_eq!(
            ctx.content_hash.as_deref(),
            Some(content_hash("store me").as_str())
        );
        assert_eq!(ctx.embedding.as_ref().unwrap().len(), 32);
    }

    #[tokio::test]
    async fn test_mock_embedding_deterministic() {
        let generator = MockEmbeddingGenerator::new(384);

        let emb1 = generator.generate("test text").await.unwrap();
        let emb2 = generator.generate("test text").await.unwrap();

        assert_eq!(emb1.len(), 384);
        assert_eq!(emb1, emb2);
    }

    #[tokio::test]
    async fn test_mock_embedding_normalized() {
        let generator = MockEmbeddingGenerator::new(384);
        let embedding = generator.generate("test").await.unwrap();

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_ternary_wrapper_sparse() {
        use crate::ternary::SparsityConfig;

        let base = Arc::new(MockEmbeddingGenerator::new(64));
        let config = SparsityConfig::default();
        let wrapper = TernaryEmbeddingGeneratorWrapper::with_sparse(base, config);

        let quantized = wrapper.generate_quantized("test").await.unwrap();
        let reconstructed = wrapper.reconstruct(&quantized).await.unwrap();

        assert_eq!(reconstructed.len(), 64);
    }

    #[tokio::test]
    async fn test_ternary_wrapper_rvq() {
        let base = Arc::new(MockEmbeddingGenerator::new(64));
        let wrapper = TernaryEmbeddingGeneratorWrapper::with_rvq(base, 2, 256);

        let quantized = wrapper.generate_quantized("test").await.unwrap();
        let reconstructed = wrapper.reconstruct(&quantized).await.unwrap();

        assert_eq!(reconstructed.len(), 64);
        assert_eq!(wrapper.strategy(), "rvq");
    }

    #[tokio::test]
    async fn test_embedder_as_generator() {
        let emb: Arc<dyn Embedder> = Arc::new(HashingEmbedder::new(16));
        let gen = EmbedderAsGenerator::new(emb);
        let v = gen.generate("x").await.unwrap();
        assert_eq!(v.len(), 16);
        assert_eq!(gen.dimension(), 16);
    }
}
