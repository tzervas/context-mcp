//! CPU-optimized text-based context retrieval with scoring
//!
//! Provides parallel processing capabilities using rayon for efficient
//! text matching and relevance scoring of stored contexts, with optional
//! semantic similarity via a real [`crate::embeddings::Embedder`] (Wave 1).
//!
//! **Honesty:** Legitimate vector RAG is not complete (no ANN store / eval yet).
//! Semantic mode is off by default and **fail closed** without a real embedder.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::context::{Context, ContextDomain, ContextId, ContextQuery};
use crate::embeddings::{Embedder, QuantizedEmbeddingGenerator};
use crate::error::{ContextError, ContextResult};
use crate::storage::ContextStore;
use crate::temporal::{TemporalQuery, TemporalStats};

/// RAG processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Maximum results per query
    pub max_results: usize,
    /// Minimum relevance threshold (0.0 to 1.0)
    pub min_relevance: f64,
    /// Enable parallel processing
    pub parallel: bool,
    /// Number of threads (0 = auto)
    pub num_threads: usize,
    /// Apply temporal decay to scoring
    pub temporal_decay: bool,
    /// Only retrieve screened-safe contexts
    pub safe_only: bool,
    /// Chunk size for parallel processing
    pub chunk_size: usize,
    /// Embedding strategy for semantic search: "sparse", "rvq", or "hybrid"
    pub embedding_strategy: String,
    /// Weight for semantic similarity in final score
    pub semantic_weight: f64,
    /// Gate for semantic similarity (C0 honesty): false until real embedder + vector + eval gates.
    /// When false, retrieve uses only metadata/temporal/keyword scores.
    /// When true, requires a real (`is_semantic`) [`Embedder`] — fail closed, no hash pseudo.
    pub enable_semantic: bool,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            min_relevance: 0.1,
            parallel: true,
            num_threads: 0, // Auto-detect
            temporal_decay: true,
            safe_only: true,
            chunk_size: 1000,
            embedding_strategy: "sparse".to_string(),
            semantic_weight: 0.2,
            enable_semantic: false, // C0: off by default (fail closed for semantic claims)
        }
    }
}

/// Result from RAG retrieval with scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredContext {
    /// The context
    pub context: Context,
    /// Relevance score (0.0 to 1.0)
    pub score: f64,
    /// Contributing score components
    pub score_breakdown: ScoreBreakdown,
}

/// Breakdown of score components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Temporal relevance
    pub temporal: f64,
    /// Importance score
    pub importance: f64,
    /// Domain match score
    pub domain_match: f64,
    /// Tag match score
    pub tag_match: f64,
    /// Content similarity (if embedding available)
    pub similarity: Option<f64>,
}

/// RAG retrieval results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// Scored contexts
    pub contexts: Vec<ScoredContext>,
    /// Query used
    pub query_summary: String,
    /// Processing time in ms
    pub processing_time_ms: u64,
    /// Total candidates considered
    pub candidates_considered: usize,
    /// Temporal statistics
    pub temporal_stats: TemporalStats,
}

/// CPU-optimized retrieval processor (metadata/temporal + optional real-embedder similarity)
pub struct RagProcessor {
    config: RagConfig,
    store: Arc<ContextStore>,
    /// Wave 1 dense embedder (required + `is_semantic` when `enable_semantic`)
    embedder: Option<Arc<dyn Embedder>>,
    /// Legacy quantized generator (ternary pipeline); not used for fail-closed semantic gate
    #[allow(dead_code)]
    embedding_generator: Option<Arc<dyn QuantizedEmbeddingGenerator>>,
}

impl RagProcessor {
    /// Create a new processor without an embedder (semantic mode will fail closed if enabled)
    pub fn new(store: Arc<ContextStore>, config: RagConfig) -> Self {
        Self::configure_threads(&config);
        Self {
            config,
            store,
            embedder: None,
            embedding_generator: None,
        }
    }

    /// Create with a Wave 1 [`Embedder`]. Semantic mode requires `embedder.is_semantic()`.
    pub fn with_embedder(
        store: Arc<ContextStore>,
        config: RagConfig,
        embedder: Arc<dyn Embedder>,
    ) -> Self {
        Self::configure_threads(&config);
        Self {
            config,
            store,
            embedder: Some(embedder),
            embedding_generator: None,
        }
    }

    /// Legacy: quantized embedding generator only (does not satisfy semantic fail-closed gate)
    pub fn with_embeddings(
        store: Arc<ContextStore>,
        config: RagConfig,
        embedding_generator: Arc<dyn QuantizedEmbeddingGenerator>,
    ) -> Self {
        Self::configure_threads(&config);
        Self {
            config,
            store,
            embedder: None,
            embedding_generator: Some(embedding_generator),
        }
    }

    /// Embedder + legacy quantized generator
    pub fn with_embedder_and_quantized(
        store: Arc<ContextStore>,
        config: RagConfig,
        embedder: Arc<dyn Embedder>,
        embedding_generator: Arc<dyn QuantizedEmbeddingGenerator>,
    ) -> Self {
        Self::configure_threads(&config);
        Self {
            config,
            store,
            embedder: Some(embedder),
            embedding_generator: Some(embedding_generator),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(store: Arc<ContextStore>) -> Self {
        Self::new(store, RagConfig::default())
    }

    fn configure_threads(config: &RagConfig) {
        if config.num_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.num_threads)
                .build_global()
                .ok();
        }
    }

    /// Active embedder, if any
    pub fn embedder(&self) -> Option<&Arc<dyn Embedder>> {
        self.embedder.as_ref()
    }

    /// Ensure semantic mode is allowed (C0/C1 fail closed).
    fn require_semantic_embedder(&self) -> ContextResult<&Arc<dyn Embedder>> {
        match &self.embedder {
            Some(e) if e.is_semantic() => Ok(e),
            Some(e) => Err(ContextError::SemanticUnavailable(format!(
                "embedder '{}' reports is_semantic=false (hash/mock/null cannot satisfy semantic mode; fail closed)",
                e.model_id()
            ))),
            None => Err(ContextError::SemanticUnavailable(
                "enable_semantic=true but no Embedder configured (fail closed; set a real embedder)".into(),
            )),
        }
    }

    /// Retrieve contexts using a query
    pub async fn retrieve(&self, query: &RetrievalQuery) -> ContextResult<RetrievalResult> {
        let start = std::time::Instant::now();

        // C1 fail closed: semantic mode never uses hash pseudo-vectors
        if self.config.enable_semantic {
            self.require_semantic_embedder()?;
        }

        // Build context query
        let mut ctx_query = ContextQuery::new();

        if let Some(domain) = &query.domain {
            ctx_query = ctx_query.with_domain(domain.clone());
        }

        for tag in &query.tags {
            ctx_query = ctx_query.with_tag(tag.clone());
        }

        if let Some(min_importance) = query.min_importance {
            ctx_query = ctx_query.with_min_importance(min_importance);
        }

        // Get candidates from storage
        let candidates: Vec<Context> = self.store.query(&ctx_query).await?;
        let candidates_count = candidates.len();

        // Apply temporal filtering
        let temporal_query = query.temporal.clone().unwrap_or_default();
        let filtered: Vec<Context> = candidates
            .into_iter()
            .filter(|c| temporal_query.matches(c))
            .filter(|c| !self.config.safe_only || c.is_safe())
            .collect();

        // Precompute semantic similarities (async embed) before parallel score
        let similarities = if self.config.enable_semantic {
            self.compute_semantic_similarities(&filtered, query).await?
        } else {
            HashMap::new()
        };

        // Score contexts (parallel or sequential)
        let scored = if self.config.parallel && filtered.len() > self.config.chunk_size {
            self.score_parallel(&filtered, query, &temporal_query, &similarities)
        } else {
            self.score_sequential(&filtered, query, &temporal_query, &similarities)
        };

        // Filter by minimum relevance and sort
        let mut results: Vec<ScoredContext> = scored
            .into_iter()
            .filter(|s| s.score >= self.config.min_relevance)
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(self.config.max_results);

        let temporal_stats = TemporalStats::from_contexts(
            &results
                .iter()
                .map(|s| s.context.clone())
                .collect::<Vec<_>>(),
        );

        Ok(RetrievalResult {
            contexts: results,
            query_summary: query.to_string(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            candidates_considered: candidates_count,
            temporal_stats,
        })
    }

    /// Embed query + contexts (batch where needed) and return cosine similarities in [0, 1].
    async fn compute_semantic_similarities(
        &self,
        contexts: &[Context],
        query: &RetrievalQuery,
    ) -> ContextResult<HashMap<ContextId, f64>> {
        let embedder = self.require_semantic_embedder()?;
        let text = match &query.text {
            Some(t) if !t.is_empty() => t.as_str(),
            _ => return Ok(HashMap::new()),
        };

        let query_vec = embedder.embed_one(text).await?;
        let model = embedder.model_id();
        let dims = embedder.dims();

        // Reuse stored vectors when model + dims match; otherwise batch-embed content
        let mut need_ids: Vec<ContextId> = Vec::new();
        let mut need_texts: Vec<&str> = Vec::new();
        let mut cached: HashMap<ContextId, Vec<f32>> = HashMap::new();

        for ctx in contexts {
            // Reuse stored vector when dims match query and model is compatible
            let reusable = match (&ctx.embedding, &ctx.embedding_model) {
                (Some(v), Some(m)) if *m == model && v.len() == query_vec.len() => Some(v),
                // Legacy items without model label: accept only if dims match declared embedder dims
                (Some(v), None) if (dims == 0 || v.len() == dims) && v.len() == query_vec.len() => {
                    Some(v)
                }
                _ => None,
            };

            if let Some(v) = reusable {
                cached.insert(ctx.id.clone(), v.clone());
                continue;
            }
            need_ids.push(ctx.id.clone());
            need_texts.push(ctx.content.as_str());
        }

        if !need_texts.is_empty() {
            let batch = embedder.embed_batch(&need_texts).await?;
            if batch.len() != need_ids.len() {
                return Err(ContextError::Internal(
                    "embed_batch length mismatch vs contexts".into(),
                ));
            }
            for (id, vec) in need_ids.into_iter().zip(batch) {
                cached.insert(id, vec);
            }
        }

        let mut out = HashMap::with_capacity(contexts.len());
        for ctx in contexts {
            if let Some(ctx_vec) = cached.get(&ctx.id) {
                let sim = cosine_similarity(&query_vec, ctx_vec).unwrap_or(0.0);
                // Map cosine [-1,1] to [0,1] for score mixing
                let unit = ((sim as f64) + 1.0) * 0.5;
                out.insert(ctx.id.clone(), unit.clamp(0.0, 1.0));
            }
        }
        Ok(out)
    }

    /// Score contexts in parallel using rayon
    fn score_parallel(
        &self,
        contexts: &[Context],
        query: &RetrievalQuery,
        temporal: &TemporalQuery,
        similarities: &HashMap<ContextId, f64>,
    ) -> Vec<ScoredContext> {
        contexts
            .par_iter()
            .map(|ctx| self.score_context(ctx, query, temporal, similarities))
            .collect()
    }

    /// Score contexts sequentially
    fn score_sequential(
        &self,
        contexts: &[Context],
        query: &RetrievalQuery,
        temporal: &TemporalQuery,
        similarities: &HashMap<ContextId, f64>,
    ) -> Vec<ScoredContext> {
        contexts
            .iter()
            .map(|ctx| self.score_context(ctx, query, temporal, similarities))
            .collect()
    }

    /// Score a single context
    fn score_context(
        &self,
        ctx: &Context,
        query: &RetrievalQuery,
        temporal: &TemporalQuery,
        similarities: &HashMap<ContextId, f64>,
    ) -> ScoredContext {
        let temporal_score = if self.config.temporal_decay {
            temporal.relevance_score(ctx)
        } else {
            1.0
        };

        let importance_score = ctx.metadata.importance as f64;

        let domain_match_score = if query.domain.as_ref() == Some(&ctx.domain) {
            1.0
        } else if query.domain.is_none() {
            0.5 // Neutral if no domain specified
        } else {
            0.2 // Partial credit for different domains
        };

        let tag_match_score = if !query.tags.is_empty() {
            let matching_tags = query
                .tags
                .iter()
                .filter(|t| ctx.metadata.tags.contains(*t))
                .count();
            matching_tags as f64 / query.tags.len() as f64
        } else {
            0.5 // Neutral
        };

        // Semantic similarity only from precomputed real-embedder map (never hash pseudo)
        let similarity_score: Option<f64> = if self.config.enable_semantic {
            similarities.get(&ctx.id).copied()
        } else {
            None
        };

        let breakdown = ScoreBreakdown {
            temporal: temporal_score,
            importance: importance_score,
            domain_match: domain_match_score,
            tag_match: tag_match_score,
            similarity: similarity_score,
        };

        // Weighted final score: incorporate semantic weight if available
        let base_weight = 1.0 - self.config.semantic_weight;
        let mut score = base_weight
            * (0.25 * breakdown.temporal
                + 0.25 * breakdown.importance
                + 0.25 * breakdown.domain_match
                + 0.25 * breakdown.tag_match);

        if let Some(sim) = similarity_score {
            score += self.config.semantic_weight * sim;
        }

        ScoredContext {
            context: ctx.clone(),
            score,
            score_breakdown: breakdown,
        }
    }

    /// Cosine similarity between two equal-length dense vectors
    pub fn compute_similarity(a: &[f32], b: &[f32]) -> Result<f32, String> {
        cosine_similarity(a, b)
    }

    /// Retrieve by text query with simple keyword matching
    pub async fn retrieve_by_text(&self, text: &str) -> ContextResult<RetrievalResult> {
        let query = RetrievalQuery::from_text(text);
        self.retrieve(&query).await
    }

    /// Get configuration
    pub fn config(&self) -> &RagConfig {
        &self.config
    }
}

/// Query for RAG retrieval
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetrievalQuery {
    /// Text query (for keyword/semantic matching)
    pub text: Option<String>,
    /// Domain filter
    pub domain: Option<ContextDomain>,
    /// Tag filters
    pub tags: Vec<String>,
    /// Minimum importance
    pub min_importance: Option<f32>,
    /// Temporal query parameters
    pub temporal: Option<TemporalQuery>,
    /// Maximum results
    pub max_results: Option<usize>,
}

impl RetrievalQuery {
    /// Create a new retrieval query
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from text
    pub fn from_text(text: &str) -> Self {
        Self {
            text: Some(text.to_string()),
            ..Default::default()
        }
    }

    /// Set domain filter
    pub fn with_domain(mut self, domain: ContextDomain) -> Self {
        self.domain = Some(domain);
        self
    }

    /// Add tag filter
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set minimum importance
    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = Some(importance);
        self
    }

    /// Set temporal parameters
    pub fn with_temporal(mut self, temporal: TemporalQuery) -> Self {
        self.temporal = Some(temporal);
        self
    }

    /// Query for recent contexts
    pub fn recent(hours: i64) -> Self {
        Self::new().with_temporal(TemporalQuery::recent(hours))
    }
}

impl std::fmt::Display for RetrievalQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if let Some(text) = &self.text {
            parts.push(format!("text: '{}'", text));
        }
        if let Some(domain) = &self.domain {
            parts.push(format!("domain: {:?}", domain));
        }
        if !self.tags.is_empty() {
            parts.push(format!("tags: {:?}", self.tags));
        }
        if let Some(importance) = self.min_importance {
            parts.push(format!("min_importance: {}", importance));
        }

        if parts.is_empty() {
            write!(f, "all contexts")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

/// Batch processing for multiple queries
pub struct BatchProcessor {
    processor: Arc<RagProcessor>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(processor: Arc<RagProcessor>) -> Self {
        Self { processor }
    }

    /// Process multiple queries (sequential for async compatibility)
    pub async fn process_batch(
        &self,
        queries: Vec<RetrievalQuery>,
    ) -> Vec<ContextResult<RetrievalResult>> {
        let mut results = Vec::with_capacity(queries.len());
        for query in queries {
            results.push(self.processor.retrieve(&query).await);
        }
        results
    }
}

/// Cosine similarity; returns error on dimension mismatch.
fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, String> {
    if a.len() != b.len() {
        return Err("dimension mismatch".to_string());
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        Ok(dot_product / (norm_a * norm_b))
    } else {
        Ok(0.0)
    }
}

/// Quarantined demo-only word-hash pseudo embedding (ROADMAP C1.5).
///
/// **Not used on production retrieve paths.** Kept under `cfg(test)` so historical
/// behavior remains documentable without shipping as a silent fallback.
#[cfg(test)]
pub(crate) fn text_to_pseudo_embedding_quarantined(text: &str) -> Vec<f32> {
    let words: Vec<&str> = text.split_whitespace().take(100).collect();
    let dim = 64;
    let mut embedding = vec![0.0f32; dim];

    for word in words.iter() {
        let mut hash = 5381u32;
        for c in word.chars() {
            hash = hash.wrapping_mul(33).wrapping_add(c as u32);
        }
        let h = hash as f32;
        for (j, elem) in embedding.iter_mut().enumerate().take(dim) {
            *elem += (h * ((j + 1) as f32)).sin();
        }
    }

    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        embedding.iter_mut().for_each(|x| *x /= norm);
    }
    embedding
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::{HashingEmbedder, NullEmbedder};
    use crate::storage::StorageConfig;
    use async_trait::async_trait;
    use tempfile::TempDir;

    fn create_test_store() -> (Arc<ContextStore>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            persist_path: Some(temp_dir.path().to_path_buf()),
            enable_persistence: true,
            ..Default::default()
        };
        let store = ContextStore::new(config).unwrap();
        (Arc::new(store), temp_dir)
    }

    /// Test double: hashing vectors labeled semantic so we can unit-test the gate path.
    /// **Not for production.**
    struct TestSemanticEmbedder {
        inner: HashingEmbedder,
    }

    impl TestSemanticEmbedder {
        fn new(dims: usize) -> Self {
            Self {
                inner: HashingEmbedder::new(dims).with_model_id("test-semantic-hash"),
            }
        }
    }

    #[async_trait]
    impl Embedder for TestSemanticEmbedder {
        fn model_id(&self) -> &str {
            self.inner.model_id()
        }
        fn dims(&self) -> usize {
            self.inner.dims()
        }
        fn is_semantic(&self) -> bool {
            true
        }
        async fn embed_batch(&self, texts: &[&str]) -> ContextResult<Vec<Vec<f32>>> {
            self.inner.embed_batch(texts).await
        }
    }

    #[test]
    fn test_retrieval_query() {
        let query = RetrievalQuery::from_text("test query")
            .with_domain(ContextDomain::Code)
            .with_tag("rust");

        assert_eq!(query.text, Some("test query".to_string()));
        assert_eq!(query.domain, Some(ContextDomain::Code));
        assert!(query.tags.contains(&"rust".to_string()));
    }

    #[tokio::test]
    async fn test_rag_processor_default_no_semantic() {
        let (store, _temp) = create_test_store();
        let processor = RagProcessor::with_defaults(store.clone());
        assert!(!processor.config().enable_semantic);

        let ctx = Context::new("Test content", ContextDomain::Code);
        store.store(ctx).await.unwrap();

        let result = processor.retrieve(&RetrievalQuery::new()).await.unwrap();
        assert_eq!(result.candidates_considered, 1);
        assert!(result.contexts[0].score_breakdown.similarity.is_none());
    }

    #[tokio::test]
    async fn test_enable_semantic_without_embedder_fail_closed() {
        let (store, _temp) = create_test_store();
        let config = RagConfig {
            enable_semantic: true,
            ..Default::default()
        };
        let processor = RagProcessor::new(store.clone(), config);

        store
            .store(Context::new("x", ContextDomain::General))
            .await
            .unwrap();

        let err = processor
            .retrieve(&RetrievalQuery::from_text("q"))
            .await
            .unwrap_err();
        assert!(matches!(err, ContextError::SemanticUnavailable(_)));
    }

    #[tokio::test]
    async fn test_enable_semantic_with_null_fail_closed() {
        let (store, _temp) = create_test_store();
        let config = RagConfig {
            enable_semantic: true,
            ..Default::default()
        };
        let processor = RagProcessor::with_embedder(store.clone(), config, Arc::new(NullEmbedder));

        let err = processor
            .retrieve(&RetrievalQuery::from_text("q"))
            .await
            .unwrap_err();
        assert!(matches!(err, ContextError::SemanticUnavailable(_)));
    }

    #[tokio::test]
    async fn test_enable_semantic_with_hashing_fail_closed() {
        let (store, _temp) = create_test_store();
        let config = RagConfig {
            enable_semantic: true,
            ..Default::default()
        };
        let processor =
            RagProcessor::with_embedder(store.clone(), config, Arc::new(HashingEmbedder::new(32)));

        let err = processor
            .retrieve(&RetrievalQuery::from_text("q"))
            .await
            .unwrap_err();
        assert!(matches!(err, ContextError::SemanticUnavailable(_)));
    }

    #[tokio::test]
    async fn test_semantic_with_real_flagged_embedder() {
        let (store, _temp) = create_test_store();
        let config = RagConfig {
            enable_semantic: true,
            safe_only: false,
            ..Default::default()
        };
        let processor = RagProcessor::with_embedder(
            store.clone(),
            config,
            Arc::new(TestSemanticEmbedder::new(32)),
        );

        store
            .store(Context::new("hello world alpha", ContextDomain::General))
            .await
            .unwrap();

        let result = processor
            .retrieve(&RetrievalQuery::from_text("hello world"))
            .await
            .unwrap();
        assert_eq!(result.candidates_considered, 1);
        assert!(result.contexts[0].score_breakdown.similarity.is_some());
    }

    #[test]
    fn test_quarantined_pseudo_not_empty() {
        let v = text_to_pseudo_embedding_quarantined("demo only");
        assert_eq!(v.len(), 64);
    }
}
