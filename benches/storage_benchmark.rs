use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use context_mcp::{Context, ContextDomain, ContextStore, StorageConfig};
use tokio::runtime::Runtime;

fn storage_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Benchmark: Store single context
    c.bench_function("store_single_context", |b| {
        b.to_async(&rt).iter(|| async {
            let config = StorageConfig {
                memory_cache_size: 1000,
                enable_persistence: false,
            };
            let store = ContextStore::new(config).unwrap();
            let ctx = Context::new("Test content", ContextDomain::Code);
            
            store.store(black_box(ctx)).await.unwrap();
        });
    });
    
    // Benchmark: Store and retrieve
    c.bench_function("store_and_retrieve", |b| {
        b.to_async(&rt).iter(|| async {
            let config = StorageConfig {
                memory_cache_size: 1000,
                enable_persistence: false,
            };
            let store = ContextStore::new(config).unwrap();
            let ctx = Context::new("Test content", ContextDomain::Code);
            
            let id = store.store(ctx).await.unwrap();
            let _retrieved = store.get(&id).await.unwrap();
        });
    });
    
    // Benchmark: Query with different cache sizes
    let mut group = c.benchmark_group("query_performance");
    for cache_size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(cache_size),
            cache_size,
            |b, &cache_size| {
                b.to_async(&rt).iter(|| async move {
                    let config = StorageConfig {
                        memory_cache_size: cache_size,
                        enable_persistence: false,
                    };
                    let store = ContextStore::new(config).unwrap();
                    
                    // Pre-populate with some contexts
                    for i in 0..100 {
                        let ctx = Context::new(
                            &format!("Test content {}", i),
                            ContextDomain::Code
                        );
                        store.store(ctx).await.unwrap();
                    }
                    
                    // Query
                    let _results = store.query_text("Test", 10, None).await.unwrap();
                });
            },
        );
    }
    group.finish();
    
    // Benchmark: Bulk storage
    c.bench_function("store_100_contexts", |b| {
        b.to_async(&rt).iter(|| async {
            let config = StorageConfig {
                memory_cache_size: 1000,
                enable_persistence: false,
            };
            let store = ContextStore::new(config).unwrap();
            
            for i in 0..100 {
                let ctx = Context::new(
                    &format!("Test content {}", i),
                    ContextDomain::Code
                );
                store.store(ctx).await.unwrap();
            }
        });
    });
}

criterion_group!(benches, storage_benchmarks);
criterion_main!(benches);
