# Multi-stage Docker build for SecureGuard
# Stage 1: Build Rust backend
FROM rust:1.75-slim AS rust-builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust source code
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build the application in release mode
RUN cargo build --release -p secureguard-api

# Stage 2: Build React frontend
FROM node:18-alpine AS frontend-builder

WORKDIR /app

# Copy frontend source
COPY frontend/package*.json ./
RUN npm ci --only=production

COPY frontend/ ./
RUN npm run build

# Stage 3: Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r secureguard && useradd -r -g secureguard secureguard

WORKDIR /app

# Copy the built Rust binary
COPY --from=rust-builder /app/target/release/secureguard-api /usr/local/bin/secureguard-api

# Copy the built frontend
COPY --from=frontend-builder /app/dist ./public

# Copy migrations
COPY migrations ./migrations

# Set permissions
RUN chown -R secureguard:secureguard /app
USER secureguard

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Set environment variables
ENV DATABASE_URL=""
ENV JWT_SECRET=""
ENV RUST_LOG=secureguard_api=info
ENV PORT=3000

# Run the application
CMD ["secureguard-api"]