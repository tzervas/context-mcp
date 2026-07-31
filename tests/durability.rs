//! Durable memory: context stored before a restart must be FINDABLE after one.
//!
//! The bug: opening sled restored the BYTES but not the ability to find them.
//! ContextStore::new initialised domain_index and tag_index as empty maps on every
//! construction, and query() -> get_candidate_ids() reads exactly those indices
//! (storage.rs:412, 420). So a restart left prior context on disk and invisible to
//! every discovery path — persisted, and unreachable.
#![cfg(feature = "persistence")]

use context_mcp::context::{Context, ContextDomain, ContextQuery};
use context_mcp::storage::{ContextStore, StorageConfig};
use std::path::PathBuf;

fn tmp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ctxmcp-dur-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn cfg(path: &PathBuf) -> StorageConfig {
    StorageConfig {
        enable_persistence: true,
        persist_path: Some(path.clone()),
        ..Default::default()
    }
}

/// THE BUG. Store, drop the store (a restart), reopen the same path, and query by
/// domain. Fails against the pre-fix code, which is what makes this a regression
/// test rather than a description of current behaviour.
#[tokio::test]
async fn context_survives_restart_and_is_findable_by_domain() {
    let path = tmp_path("domain");
    let id;
    {
        let store = ContextStore::new(cfg(&path)).expect("open");
        let c = Context::new("a durable fact about the fleet", ContextDomain::Code);
        id = c.id.clone();
        store.store(c).await.expect("store");
    } // dropped — this is the restart

    let reopened = ContextStore::new(cfg(&path)).expect("reopen");
    let found = reopened
        .query(&ContextQuery {
            domain_filter: Some(ContextDomain::Code),
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");

    assert!(
        found.iter().any(|c| c.id == id),
        "context stored before a restart must be findable by domain after it; \
         got {} result(s)",
        found.len()
    );
}

/// Tags are a SEPARATE index, so a fix that rebuilt only the domain index would
/// pass the test above and still lose tag discovery.
#[tokio::test]
async fn context_survives_restart_and_is_findable_by_tag() {
    let path = tmp_path("tag");
    let id;
    {
        let store = ContextStore::new(cfg(&path)).expect("open");
        let mut c = Context::new("tagged durable fact", ContextDomain::Research);
        c.metadata.tags = vec!["beta".into(), "gamma".into()];
        id = c.id.clone();
        store.store(c).await.expect("store");
    }

    let reopened = ContextStore::new(cfg(&path)).expect("reopen");
    let found = reopened
        .query(&ContextQuery {
            tag_filter: Some(vec!["beta".into()]),
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");

    assert!(
        found.iter().any(|c| c.id == id),
        "context stored before a restart must be findable by tag after it; \
         got {} result(s)",
        found.len()
    );
}

/// disk_count reading 0 with data present is the symptom that made this look like a
/// config problem rather than an architectural one.
#[tokio::test]
async fn disk_count_reflects_persisted_records_after_restart() {
    let path = tmp_path("stats");
    {
        let store = ContextStore::new(cfg(&path)).expect("open");
        for i in 0..3 {
            store
                .store(Context::new(format!("record {i}"), ContextDomain::Code))
                .await
                .expect("store");
        }
    }
    let reopened = ContextStore::new(cfg(&path)).expect("reopen");
    let stats = reopened.stats().await;
    assert!(
        stats.disk_count >= 3,
        "disk_count must reflect persisted records, got {}",
        stats.disk_count
    );
}
