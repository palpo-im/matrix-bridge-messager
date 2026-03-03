# Build stage
FROM rust:1.93 AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/matrix-bridge-messager /usr/local/bin/

# Create data directory
RUN mkdir -p /data

# Set environment variables
ENV CONFIG_PATH=/data/config.yaml
ENV RUST_LOG=info

# Expose port
EXPOSE 9006

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9006/health || exit 1

# Run the binary
ENTRYPOINT ["matrix-bridge-messager"]

