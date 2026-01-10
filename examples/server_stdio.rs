//! Example of running context-mcp with stdio transport
//! 
//! This example demonstrates the StdioTransport for running
//! the server in stdio mode, useful for editor integrations.

use context_mcp::{
    server::{ServerConfig, StdioTransport},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configure server
    let server_config = ServerConfig::default();

    // Create stdio transport
    let stdio = StdioTransport::new(server_config)?;
    
    println!("Starting MCP server in stdio mode...");
    
    // Run stdio transport
    stdio.run().await?;

    Ok(())
}
