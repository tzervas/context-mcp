//! CLI-level contract for embedder selection (docs/ROADMAP.md "Config (CLI / env)").
//!
//! These drive the real binary, so they hold regardless of how the wiring is
//! refactored internally. Every assertion here fails against the pre-change binary,
//! which had no `--embedder` flag at all and started successfully with
//! `--enable-semantic` even though no embedder could ever exist behind it.
//!
//! Each invocation gets `--storage-path` under a tempdir: the default store path is
//! `./data/context_store` and sled takes an exclusive lock, so tests sharing it would
//! collide with each other and with any locally running server.

#![cfg(feature = "server")]

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("store");

    let output = Command::new(env!("CARGO_BIN_EXE_context-mcp"))
        .args(args)
        .arg("--storage-path")
        .arg(&store)
        // stdin is an immediately-closed pipe: a server that starts successfully reads
        // EOF and exits 0, so a healthy start is distinguishable from a refused one.
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to run context-mcp");

    output
}

/// Captured stderr with ANSI colour codes removed — the tracing formatter wraps even the
/// `=` between a field and its value, so raw output would not match plain substrings.
fn stderr(output: &Output) -> String {
    strip_ansi(&String::from_utf8_lossy(&output.stderr))
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        // CSI: ESC '[' , parameter bytes (0x30–0x3F), then a final byte in '@'..='~'.
        // The '[' must be consumed before scanning, since '[' is itself in that range.
        if chars.peek() == Some(&'[') {
            chars.next();
        }
        for c in chars.by_ref() {
            if ('@'..='~').contains(&c) {
                break;
            }
        }
    }
    out
}

#[test]
fn semantic_without_embedder_aborts_at_startup() {
    let out = run(&["--stdio", "--enable-semantic"]);

    assert!(
        !out.status.success(),
        "--enable-semantic with no embedder must not start; got success. stderr:\n{}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(
        err.contains("--embedder"),
        "startup error must point at the flag that fixes it:\n{err}"
    );
}

#[test]
fn semantic_with_local_embedder_aborts_rather_than_faking_it() {
    let out = run(&["--stdio", "--embedder", "local", "--enable-semantic"]);

    assert!(
        !out.status.success(),
        "the local hashing stub must not be accepted as a semantic backend. stderr:\n{}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(
        err.contains("is_semantic=false"),
        "error must say why the backend was rejected:\n{err}"
    );
}

#[test]
fn unknown_embedder_lists_the_valid_values() {
    let out = run(&["--stdio", "--embedder", "fastembed"]);

    assert!(!out.status.success(), "unknown backend must not start");
    let err = stderr(&out);
    assert!(
        err.contains("none, local, http"),
        "error must enumerate valid backends:\n{err}"
    );
}

#[test]
fn local_embedder_starts_and_reports_itself() {
    let out = run(&["--stdio", "--embedder", "local", "--embed-dims", "32"]);

    assert!(
        out.status.success(),
        "non-semantic local embedder is a valid configuration. stderr:\n{}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(
        err.contains("embedder active") && err.contains("dims=32"),
        "startup must report the active embedder so operators can verify it:\n{err}"
    );
    assert!(
        err.contains("semantic=false"),
        "startup must state plainly that this backend is not semantic:\n{err}"
    );
}

#[test]
fn default_invocation_still_starts_with_no_embedder() {
    // The shipped launch command is just `--stdio`; it must keep working untouched.
    let out = run(&["--stdio"]);
    assert!(
        out.status.success(),
        "plain --stdio must still start. stderr:\n{}",
        stderr(&out)
    );
}

/// In the default build (no `http-embedder`), selecting http must name the missing
/// cargo feature instead of quietly using something else.
#[cfg(not(feature = "http-embedder"))]
#[test]
fn http_embedder_without_cargo_feature_names_the_feature() {
    let out = run(&["--stdio", "--embedder", "http", "--enable-semantic"]);

    assert!(!out.status.success(), "http backend is not in this build");
    let err = stderr(&out);
    assert!(
        err.contains("http-embedder"),
        "error must name the missing cargo feature:\n{err}"
    );
    assert!(
        err.contains("cargo build"),
        "error must tell the operator how to get it:\n{err}"
    );
}

/// With the feature compiled in, the semantic path is genuinely reachable: the server
/// starts with `--enable-semantic`. No request is made, so no network is required.
#[cfg(feature = "http-embedder")]
#[test]
fn http_embedder_with_cargo_feature_enables_semantic_mode() {
    let out = run(&[
        "--stdio",
        "--embedder",
        "http",
        "--embed-base-url",
        "https://example.invalid/v1",
        "--embed-model",
        "text-embedding-3-small",
        "--embed-dims",
        "1536",
        "--enable-semantic",
    ]);

    assert!(
        out.status.success(),
        "http embedder must satisfy semantic mode. stderr:\n{}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(
        err.contains("semantic=true"),
        "startup must report a semantic backend:\n{err}"
    );
}

/// The API key is env-only on purpose: a CLI flag would expose it in `ps`/argv.
#[test]
fn api_key_is_not_a_command_line_flag() {
    let out = Command::new(env!("CARGO_BIN_EXE_context-mcp"))
        .arg("--help")
        .output()
        .expect("failed to run context-mcp --help");
    let help = String::from_utf8_lossy(&out.stdout);

    assert!(help.contains("--embedder"), "help must document --embedder");
    assert!(
        !help.contains("--embed-api-key"),
        "the API key must not be a flag (argv leaks to `ps`):\n{help}"
    );
    assert!(
        help.contains("CONTEXT_MCP_EMBED_API_KEY"),
        "help must say where the key comes from:\n{help}"
    );
}
