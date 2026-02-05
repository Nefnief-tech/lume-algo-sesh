# Multi-stage Docker build for Lume Algo

# Stage 1: Build
FROM rust:1-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs and copy benches for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY benches ./benches

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src
COPY config ./config
COPY migrations ./migrations

# Build the actual binary
RUN touch src/main.rs && cargo build --release

# Stage 2: Run
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 lume

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/lume-algo /usr/local/bin/lume-algo

# Copy configuration
COPY --from=builder /app/config /app/config

# Set ownership
RUN chown -R lume:lume /app

# Switch to non-root user
USER lume

# Expose port
EXPOSE 8080

# Set environment variables
ENV LOG_LEVEL=info
ENV LOG_FORMAT=json

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Run the binary
CMD ["lume-algo"]
