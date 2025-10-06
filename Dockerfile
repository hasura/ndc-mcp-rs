# Multi-stage build for a Rust-based MCP connector for Hasura DDN Engine

# Chef stage - base image with cargo-chef installed
FROM rust:1.85.0-slim-bookworm AS chef

WORKDIR /app

ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libprotobuf-dev \
    protobuf-compiler \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set up Cargo environment
ENV CARGO_HOME=/app/.cargo
ENV PATH="$PATH:$CARGO_HOME/bin"

# Install Rust tools
COPY rust-toolchain.toml .
RUN rustup show
RUN cargo install cargo-chef

###
# Planner stage - prepare the recipe
FROM chef AS planner

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Prepare the recipe
RUN cargo chef prepare --recipe-path recipe.json

###
# Builder stage - build dependencies then application
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --bin mcp-connector --bin mcp-connector-cli --recipe-path recipe.json

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build the application and copy binaries to persistent location
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin mcp-connector --bin mcp-connector-cli && \
    cp /app/target/release/mcp-connector /app/mcp-connector && \
    cp /app/target/release/mcp-connector-cli /app/mcp-connector-cli

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
RUN useradd -m -u 1000 connector

# Copy the binary from builder stage
COPY --from=builder /app/mcp-connector /bin/mcp-connector
COPY --from=builder /app/mcp-connector-cli /bin/mcp-connector-cli
RUN chmod +x /bin/mcp-connector /bin/mcp-connector-cli


# Create configuration directory
RUN mkdir -p /etc/connector && \
    chown connector:connector /etc/connector

# Switch to non-root user
USER connector

ENV HASURA_CONFIGURATION_DIRECTORY=/etc/connector

# Expose the default port
EXPOSE 8080

# Set default command
CMD ["/bin/mcp-connector", "serve"]
