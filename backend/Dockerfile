# syntax=docker/dockerfile:1.7

############################
# Builder
############################
FROM rust:1.88-bookworm AS builder
WORKDIR /app

# For openssl-sys and friends
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# Use offline mode for sqlx to avoid database connection during build
ENV SQLX_OFFLINE=true

# Copy everything (including .sqlx directory for offline mode)
COPY . .

# Speed up dependency resolution
RUN cargo fetch

# Build all bins (cache mounts optional; see comment below)
RUN cargo build --release --bins

############################
# Runtime
############################
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# OpenSSL runtime + certs (Bookworm ships OpenSSL 3.x -> libssl3)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binaries
COPY --from=builder /app/target/release/server /app/bin/server
COPY --from=builder /app/target/release/attestation_watcher /app/bin/attestation_watcher
COPY --from=builder /app/target/release/migrate /app/bin/migrate

RUN chmod +x /app/bin/*

# Default binary; override per-service in Railway
ENV APP_BIN=server

# Your HTTP server should bind to $PORT; background workers can ignore it
EXPOSE 8080
CMD ["/bin/sh", "-c", "/app/bin/${APP_BIN}"]
