//! Semantic mode must be refused at construction time, not at first retrieval.
//!
//! Written deliberately against the pre-change public API (struct-update syntax, no
//! reference to the new `ServerConfig::embedder` field) so it compiles on both sides of
//! this change and fails on the old one: previously `ServerState::new` returned `Ok` here
//! and the server ran advertising a semantic flag it could never honour.

use context_mcp::{
    error::ContextError,
    rag::RagConfig,
    server::{ServerConfig, ServerState},
    storage::StorageConfig,
};

/// In-memory storage: the default path `./data/context_store` is sled-locked and would
/// collide with a locally running server.
fn config(enable_semantic: bool) -> ServerConfig {
    ServerConfig {
        storage: StorageConfig::memory_only(64),
        rag: RagConfig {
            enable_semantic,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn enable_semantic_without_embedder_fails_at_startup() {
    let err = ServerState::new(&config(true))
        .err()
        .expect("semantic mode with no embedder must refuse to start");

    assert!(
        matches!(err, ContextError::SemanticUnavailable(_)),
        "expected a typed refusal, got: {err}"
    );
}

#[test]
fn semantic_off_still_starts_without_an_embedder() {
    // Control: the default posture (semantic off, no embedder) is unaffected.
    ServerState::new(&config(false)).expect("non-semantic startup must keep working");
}
