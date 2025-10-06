# Multi-stage build for a Rust-based MCP connector for Hasura DDN Engine

# Build stage
FROM rust:1.85.0-slim-bookworm AS builder

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libprotobuf-dev \
    protobuf-compiler \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src/ ./src/

# Build the application in release mode
# The main binary is mcp-connector based on the Cargo.toml configuration
RUN cargo build --release --bin mcp-connector

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libprotobuf32 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create a non-root user for security
RUN groupadd -r connector && useradd -r -g connector connector

# Copy the binary from builder stage
COPY --from=builder /app/target/release/mcp-connector /bin/mcp-connector

# Set ownership and permissions
RUN chown connector:connector /bin/mcp-connector && \
    chmod +x /bin/mcp-connector

# Create configuration directory
RUN mkdir -p /etc/connector && \
    chown connector:connector /etc/connector

# Copy configuration file
COPY configuration/config.json /etc/connector/config.json

# Switch to non-root user
USER connector

ENV HASURA_CONFIGURATION_DIRECTORY=/etc/connector

# Expose the default port
EXPOSE 8080

# Set default command
CMD ["/bin/mcp-connector", "serve"]