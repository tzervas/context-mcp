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
    embeddings::{EmbedderConfig, EmbedderKind},
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
    /// Embedder). Requires --embedder to select a backend whose is_semantic() is true;
    /// startup ABORTS otherwise rather than degrading to hash pseudo-vectors.
    /// Per docs/ROADMAP.md honesty gate. Not legitimate RAG (no vector store/eval yet).
    #[arg(long)]
    enable_semantic: bool,

    /// Embedder backend: none | local | http (docs/ROADMAP.md "Config (CLI / env)").
    ///
    /// `none` (default): no embedder; retrieval is metadata/temporal/keyword only.
    /// `local`: deterministic hashing stub — NOT semantic (ROADMAP C1.2 open), so it
    /// cannot be combined with --enable-semantic.
    /// `http`: OpenAI-compatible remote embeddings; requires the `http-embedder`
    /// cargo feature, which is not in the default build.
    #[arg(long, default_value = "none", value_name = "none|local|http")]
    embedder: String,

    /// Model id/path for the selected embedder (required for --embedder http)
    #[arg(long, value_name = "MODEL")]
    embed_model: Option<String>,

    /// Embedding dimensionality. Required for --embedder http; local defaults to 384.
    #[arg(long, value_name = "N")]
    embed_dims: Option<usize>,

    /// API root for --embedder http, e.g. https://api.openai.com/v1
    ///
    /// The bearer token is read from $CONTEXT_MCP_EMBED_API_KEY — deliberately not a
    /// flag, so it does not appear in `ps`/argv.
    #[arg(long, value_name = "URL")]
    embed_base_url: Option<String>,
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

    // Parsed here (not by clap's value_enum) so the crate's own FromStr is the single
    // definition of the accepted values, shared by CLI and any library caller.
    let embedder_kind: EmbedderKind = args.embedder.parse()?;

    let embedder_config = EmbedderConfig {
        kind: embedder_kind,
        model: args.embed_model,
        dims: args.embed_dims,
        base_url: args.embed_base_url,
        // Secret from the environment only; a CLI flag would leak it into argv.
        api_key: std::env::var(EmbedderConfig::API_KEY_ENV).ok(),
    };

    let server_config = ServerConfig {
        host: args.host,
        port: args.port,
        storage: storage_config,
        rag: rag_config,
        embedder: embedder_config,
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
