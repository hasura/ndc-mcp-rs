# NDC-MCP Connector

A Native Data Connector (NDC) that bridges Hasura DDN with Model Context Protocol (MCP) servers, exposing MCP resources as collections and tools as functions/procedures.

## Features

- **Dynamic Schema Generation**: Automatically generates NDC schema from MCP server introspection
- **Multiple Transports**: Supports stdio (local processes) and HTTP (remote servers)
- **Multiple Servers**: Connect to multiple MCP servers simultaneously
- **Resource Mapping**: MCP resources → NDC collections
- **Tool Execution**: MCP tools → NDC functions/procedures
- **Naming Convention**: `{server_name}__{resource_or_tool}` pattern

## Quick Start

1. **Clone and build**:

   ```bash
   git clone https://github.com/hasura/ndc-mcp-rs.git
   cd ndc-mcp-rs
   cargo build
   ```

2. **Configure servers** in `configuration/servers.yaml`:

   ```yaml
   servers:
     filesystem:
       type: stdio
       command: npx
       args:
         ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/directory"]

     remote:
       type: http
       url: "http://localhost:8080/mcp"
       headers:
         Authorization: "Bearer your-token"
       timeout_seconds: 30
   ```

3. **Generate configuration**:

   ```bash
   just generate-config
   # or: cargo run --bin mcp-connector-cli -- --configuration configuration update --outfile configuration/configuration.json
   ```

4. **Start the connector**:

   ```bash
   just serve
   # or: cargo run --bin mcp-connector -- serve --configuration configuration
   ```

5. **Test the schema**:
   ```bash
   curl http://localhost:8080/schema | jq
   ```

## Configuration

The connector uses a two-step configuration process:

1. **servers.yaml**: Define your MCP servers (this is what you edit)
2. **configuration.json**: Generated automatically by introspecting the MCP servers

### Transport Types

- **stdio**: For local MCP servers (Node.js packages, Python scripts, etc.)
- **http**: For remote MCP servers using streamable HTTP transport

## Development

```bash
# Build
just build

# Format code
just format

# Run clippy
just clippy

# Generate config and serve
just generate-config && just serve
```

## Prerequisites

- Rust 1.85.0+ (edition 2021)
- MCP servers to connect to

## License

MIT License - see LICENSE file for details.
