# NDC MCP Connector Justfile

# Run clippy on all targets without dependencies
clippy:
    cargo clippy --all-targets --no-deps

# Build the project
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Serve the connector with configuration
serve:
    cargo run --bin mcp-connector -- serve --configuration configuration

# Format the code
format:
    cargo fmt

# Build Docker image with specified tag
docker-build TAG:
    docker build -t ghcr.io/hasura/mcp-connector:{{TAG}} .

# Push Docker image to ghcr.io repository
docker-push TAG:
    docker push ghcr.io/hasura/mcp-connector:{{TAG}}

# Package connector definition into a tarball
pack-connector:
    tar -czf connector-definition.tgz -C connector-definition .