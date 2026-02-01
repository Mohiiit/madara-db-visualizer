# Build stage
FROM rust:1.82-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the API binary
RUN cargo build -p api --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/api /app/api

# Copy sample database
COPY sample-db /app/sample-db

# Expose the API port
EXPOSE 3000

# Default environment variables
ENV DB_PATH=/app/sample-db
ENV INDEX_PATH=/app/index.db
ENV PORT=3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

# Run the API server
CMD ["/app/api", "--db-path", "/app/sample-db", "--port", "3000"]
