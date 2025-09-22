# NDC-MCP Connector (Rust)

A Native Data Connector (NDC) that bridges Hasura's Data Delivery Network (DDN) with the Model Context Protocol (MCP), enabling seamless integration between DDN Engine and MCP. This connector is written in Rust.

## Overview

This connector allows you to:

- Access MCP resources through NDC collections
- Execute read-only MCP tools through NDC functions
- Execute mutable MCP tools through NDC procedures
- Generate a dynamic NDC schema from MCP resources and tools

## Features

- **Dynamic Schema Generation**: Automatically generates an NDC schema from MCP resources and tools
- **Resource Mapping**: MCP resources are exposed as queryable collections in NDC
- **Tool Execution**: MCP tools can be executed through NDC functions and procedures
- **Multiple Server Support**: Connect to multiple MCP servers with different transport types
- **Naming Convention**: Uses a `{server_name}__{resource_or_tool}` pattern to uniquely identify resources and tools

## Prerequisites

- Rust 1.85.0 or later (with edition2024 support)
- An MCP server to connect to (e.g., a filesystem MCP server)

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/hasura/ndc-mcp-rs.git
   cd ndc-mcp-rs
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Configuration

Create a `config.json` file in the `configuration` directory with the following structure:

```json
{
  "servers": {
    "filesystem": {
      "type": "stdio",
      "command": "secure-filesystem-server",
      "args": ["--allowed-paths", "/path/to/allowed/directory"]
    },
    "remote_server": {
      "type": "http",
      "url": "http://localhost:8080/mcp",
      "headers": {
        "Authorization": "Bearer your-token-here"
      },
      "timeout_seconds": 30
    }
  }
}
```

You can configure multiple MCP servers with different transport types:

- **stdio**: For local MCP servers that communicate over standard input/output
- **http**: For remote MCP servers that communicate over streamable HTTP transport (recommended for HTTP-based servers)

## Usage

1. Start the NDC-MCP connector:
   ```bash
   cargo run -- serve --configuration configuration
   ```

2. The connector will start on port 8080 by default. You can now use it with Hasura or any other NDC client.

3. Access the schema:
   ```bash
   curl http://localhost:8080/schema | jq
   ```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgements

- [Hasura NDC Specification](https://github.com/hasura/ndc-spec)
- [Model Context Protocol](https://github.com/hasura/rmcp)
