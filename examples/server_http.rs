//! Example of running context-mcp as an HTTP-based MCP server
//! 
//! This example demonstrates how to run the server in HTTP mode,
//! which exposes the MCP protocol over HTTP/WebSocket endpoints.

use context_mcp::server::{McpServer, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configure server (uses default host:port of 127.0.0.1:3000)
    let server_config = ServerConfig::default();

    // Create server
    let server = McpServer::new(server_config)?;
    
    println!("Starting MCP server...");
    println!("Server will listen on: {}", server.address());
    
    // Run server
    server.run().await?;

    Ok(())
}
