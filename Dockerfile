# =============================================================================
# leptos-store Showcase — Multi-stage Docker Build
# =============================================================================
# Stage 1: Build the Rust binary + WASM client bundle
# Stage 2: Minimal runtime image with binary + static assets
# =============================================================================

# ---------------------------------------------------------------------------
# Stage 1: Builder
# ---------------------------------------------------------------------------
FROM rust:1.92-bookworm AS builder

# Install cargo-leptos and wasm target
RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos

# Install binaryen for wasm-opt (smaller WASM output)
RUN apt-get update && apt-get install -y --no-install-recommends binaryen \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# -- Dependency caching layer --
# Copy only Cargo manifests first so Docker can cache dependency builds
COPY Cargo.toml Cargo.lock ./
COPY src/lib.rs src/lib.rs
COPY examples/counter-example/Cargo.toml examples/counter-example/Cargo.toml
COPY examples/auth-store-example/Cargo.toml examples/auth-store-example/Cargo.toml
COPY examples/token-explorer-example/Cargo.toml examples/token-explorer-example/Cargo.toml
COPY examples/middleware-example/Cargo.toml examples/middleware-example/Cargo.toml
COPY examples/persistence-example/Cargo.toml examples/persistence-example/Cargo.toml
COPY examples/composition-example/Cargo.toml examples/composition-example/Cargo.toml
COPY examples/feature-flags-example/Cargo.toml examples/feature-flags-example/Cargo.toml
COPY examples/devtools-example/Cargo.toml examples/devtools-example/Cargo.toml
COPY examples/csr-example/Cargo.toml examples/csr-example/Cargo.toml
COPY examples/selectors-example/Cargo.toml examples/selectors-example/Cargo.toml
COPY examples/showcase/Cargo.toml examples/showcase/Cargo.toml

# Create stub lib.rs / main.rs for each example so cargo can resolve the workspace
RUN mkdir -p examples/counter-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/counter-example/src/lib.rs \
    && mkdir -p examples/auth-store-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/auth-store-example/src/lib.rs \
    && mkdir -p examples/token-explorer-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/token-explorer-example/src/lib.rs \
    && mkdir -p examples/middleware-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/middleware-example/src/lib.rs \
    && mkdir -p examples/persistence-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/persistence-example/src/lib.rs \
    && mkdir -p examples/composition-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/composition-example/src/lib.rs \
    && mkdir -p examples/feature-flags-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/feature-flags-example/src/lib.rs \
    && mkdir -p examples/devtools-example/src && echo "pub mod components { pub fn Demo() {} }" > examples/devtools-example/src/lib.rs \
    && mkdir -p examples/csr-example/src && echo "pub fn Demo() {}" > examples/csr-example/src/lib.rs \
    && mkdir -p examples/selectors-example/src && echo "pub fn Demo() {}" > examples/selectors-example/src/lib.rs \
    && mkdir -p examples/showcase/src && echo "fn main() {}" > examples/showcase/src/main.rs && touch examples/showcase/src/lib.rs

# -- Copy full source --
COPY . .

# Build the showcase (SSR binary + WASM client)
RUN cd examples/showcase && cargo leptos build --release

# ---------------------------------------------------------------------------
# Stage 2: Runtime
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the server binary
COPY --from=builder /app/target/release/showcase /app/showcase

# Copy the site assets (WASM, CSS, JS, etc.)
COPY --from=builder /app/examples/showcase/target/site /app/site

# Leptos expects these env vars
ENV LEPTOS_OUTPUT_NAME="showcase"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_RELOAD_PORT="0"

EXPOSE 8080

CMD ["/app/showcase"]
