# NDC-MCP Connector

**NOTE**: Use https://github.com/hasura/ndc-mcp-ts (typescript port) to work directly with `npx` invokable mcp servers. The TS ported connector has `npx` installed by default via nodejs. No need for installing deps in connector Dockerfile.

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

2. **Configure servers** in `configuration/configuration.json`:

   ```json
   {
     "servers": {
       "filesystem": {
         "type": "stdio",
         "command": "npx",
         "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/directory"]
       },
       "remote": {
         "type": "http",
         "url": "http://localhost:8080/mcp",
         "headers": {
           "Authorization": "Bearer your-token"
         },
         "timeout_seconds": 30
       }
     }
   }
   ```

3. **Start the connector**:

   ```bash
   just serve
   # or: cargo run --bin mcp-connector -- serve --configuration configuration
   ```

4. **Test the schema**:
   ```bash
   curl http://localhost:8080/schema | jq
   ```

## Configuration

The connector uses a single configuration file `configuration/configuration.json` where you define your MCP servers. The connector automatically introspects the servers at startup to discover available resources and tools.

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

# Serve the connector
just serve
```

## Prerequisites

- Rust 1.85.0+ (edition 2021)
- MCP servers to connect to

## License

MIT License - see LICENSE file for details.
