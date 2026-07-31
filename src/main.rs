//! context-mcp
//!
//! MCP server for context management with temporal reasoning (no real RAG yet).
//!
//! This crate provides a Model Context Protocol (MCP) server for storing,
//! retrieving, and querying context with:
//! - Multi-tier storage (LRU memory cache + sled disk persistence)
//! - Temporal reasoning with time-based filtering and decay scoring
//! - CPU-optimized text/metadata retrieval (semantic off by default; fail closed without real Embedder)
//! - Security screening status integration
//!
//! Wave 1 Embedder interface landed; vector store + eval still open (see docs/ROADMAP.md).
//!
//! # Usage
//!
//! Run as HTTP server:
//! ```bash
//! context-mcp --host 127.0.0.1 --port 3000
//! ```
//!
//! Run as stdio transport:
//! ```bash
//! context-mcp --stdio
//! ```

use clap::Parser;
use std::path::PathBuf;

use context_mcp::{
    rag::RagConfig,
    server::{McpServer, ServerConfig, StdioTransport},
    storage::StorageConfig,
};

/// MCP Context Management Server
#[derive(Parser, Debug)]
#[command(name = "context-mcp")]
#[command(about = "Context management MCP server with temporal reasoning")]
#[command(version)]
struct Args {
    /// Use stdio transport instead of HTTP
    #[arg(long)]
    stdio: bool,

    /// Server host (HTTP mode only)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port (HTTP mode only)
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Path for persistent storage
    #[arg(long)]
    storage_path: Option<PathBuf>,

    /// Memory cache size
    #[arg(long, default_value = "1000")]
    cache_size: usize,

    /// Disable disk persistence (default: enabled).
    ///
    /// Was `--persist` (opt-in, default false), which silently overrode
    /// StorageConfig::default()'s `enable_persistence: true`. The shipped launch
    /// command passes only `--stdio`, so durable memory was off in practice while
    /// the config claimed it was on. Inverting the flag makes the default the
    /// documented one and leaves opting OUT explicit.
    #[arg(long)]
    no_persist: bool,

    /// Number of RAG threads (0 = auto)
    #[arg(long, default_value = "0")]
    threads: usize,

    /// Disable temporal decay scoring
    #[arg(long)]
    no_decay: bool,

    /// Enable semantic similarity in retrieve (C0/C1: off-by-default; fail closed without a real
    /// Embedder — default server has none, so this flag errors until an embedder is wired).
    /// Per docs/ROADMAP.md honesty gate. Not legitimate RAG (no vector store/eval yet).
    #[arg(long)]
    enable_semantic: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    //
    // Logs MUST go to stderr, never stdout: in `--stdio` mode stdout is the JSON-RPC
    // transport, and a single log line interleaved into it makes the stream unparseable —
    // an MCP client sees `Starting MCP Context Server in stdio mode` where a JSON-RPC frame
    // should be and drops the connection. (The default `fmt()` writer is stdout.) stderr is
    // also correct for the HTTP transport, so this is unconditional rather than mode-gated.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();

    // Build configuration
    let storage_config = StorageConfig {
        memory_cache_size: args.cache_size,
        persist_path: args.storage_path,
        enable_persistence: !args.no_persist,
        auto_cleanup: true,
        cleanup_interval_secs: 300,
    };

    let rag_config = RagConfig {
        num_threads: args.threads,
        temporal_decay: !args.no_decay,
        enable_semantic: args.enable_semantic,
        ..Default::default()
    };

    let server_config = ServerConfig {
        host: args.host,
        port: args.port,
        storage: storage_config,
        rag: rag_config,
    };

    if args.stdio {
        tracing::info!("Starting MCP Context Server in stdio mode");
        let transport = StdioTransport::new(server_config)?;
        transport.run().await?;
    } else {
        tracing::info!(
            "Starting MCP Context Server on {}:{}",
            server_config.host,
            server_config.port
        );
        let server = McpServer::new(server_config)?;
        server.run().await?;
    }

    Ok(())
}
