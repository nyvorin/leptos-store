# =============================================================================
# leptos-store Showcase — Multi-stage Docker Build
# =============================================================================

# ---------------------------------------------------------------------------
# Stage 1: Builder
# ---------------------------------------------------------------------------
FROM rust:1.92-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --version 0.3.2 \
    && cargo install wasm-bindgen-cli --version 0.2.108

WORKDIR /app
COPY . .

RUN cd examples/showcase && cargo leptos build --release

# ---------------------------------------------------------------------------
# Stage 2: Runtime
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/showcase /app/showcase
COPY --from=builder /app/target/site /app/site

ENV LEPTOS_OUTPUT_NAME="showcase"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_RELOAD_PORT="0"

EXPOSE 8080

CMD ["/app/showcase"]
