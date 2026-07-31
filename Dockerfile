# context-mcp 1.0 publish image (runtime from prebuilt host binary for gate)
# Built after `cargo build --release` on host (leverages 14700K for compile)
# Local podman build + GHCR at gate
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Prebuilt by host cargo (with RAYON_NUM_THREADS=28 for parallel)
COPY target/release/context-mcp /usr/local/bin/context-mcp

EXPOSE 3000

ENTRYPOINT ["context-mcp"]
CMD ["--help"]
