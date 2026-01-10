use context_mcp::context::ContextDomain;
use context_mcp::{Context, ContextStore, StorageConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage configuration
    let config = StorageConfig::default();

    // Create context store
    let store = ContextStore::new(config)?;

    // Store some context
    let ctx = Context::new("This is some important information", ContextDomain::Code);
    let id = store.store(ctx).await?;
    println!("Stored context with ID: {}", id);

    // Retrieve it
    let retrieved = store.get(&id).await?.expect("Context should exist");
    println!("Retrieved: {}", retrieved.content);

    // Query contexts
    let query = context_mcp::context::ContextQuery::new()
        .with_domain(ContextDomain::Code)
        .with_limit(10);
    let results = store.query(&query).await?;
    println!("Found {} matching contexts", results.len());

    Ok(())
}
