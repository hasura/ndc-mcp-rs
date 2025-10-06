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

# Generate configuration file
generate-config:
    cargo run --bin mcp-connector-cli -- --configuration configuration update --outfile configuration/configuration.json
