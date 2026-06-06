# Multi-stage build for the Rust Axum API.
FROM rust:1.77-bookworm AS builder
WORKDIR /app

# System dependencies for OpenSSL (used by jsonwebtoken, sea-orm).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Pre-warm dependency cache.
COPY Cargo.toml Cargo.lock ./
COPY apps/api/Cargo.toml apps/api/
RUN mkdir -p apps/api/src && echo "fn main(){}" > apps/api/src/main.rs \
 && cargo build --release -p acmind-api || true

# Real build.
COPY apps/api apps/api
RUN cargo build --release -p acmind-api

# Runtime image.
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/acmind-api /usr/local/bin/acmind-api

EXPOSE 8080
CMD ["acmind-api"]
