FROM ghcr.io/hasura/mcp-connector:v0.1.0

# Base image provides a non-root 'connector' user (UID 1000) with pre-configured
# writable directories for package managers and a /etc/connector config directory.
# Switch to 'root' for system installations, then back to 'connector' for user operations.

# Copy configuration file
COPY --chown=connector:connector configuration.json /etc/connector

# =============================================================================
# USER EXTENSION SECTION - UNCOMMENT AND MODIFY AS NEEDED
# =============================================================================

# -----------------------------------------------------------------------------
# Option 1: Node.js Runtime (for npm-based MCP servers)
# Uncomment the following lines to install Node.js and npm
# -----------------------------------------------------------------------------
# USER root
# RUN apt-get update && apt-get install -y \
#     nodejs \
#     npm \
#     && rm -rf /var/lib/apt/lists/*
# USER connector

# Install Node.js MCP servers (uncomment and modify as needed)
# RUN npm install -g @modelcontextprotocol/server-filesystem
# RUN npm install -g @modelcontextprotocol/server-sequential-thinking
# RUN npm install -g @modelcontextprotocol/server-everything

# -----------------------------------------------------------------------------
# Option 2: Python Runtime (for pip-based MCP servers)
# Uncomment the following lines to install Python and pip
# -----------------------------------------------------------------------------
# USER root
# RUN apt-get update && apt-get install -y \
#     python3 \
#     python3-pip \
#     python3-venv \
#     git \
#     && rm -rf /var/lib/apt/lists/*
# USER connector

# Create Python virtual environment (recommended)
# RUN python3 -m venv /home/connector/.venv
# ENV PATH="/home/connector/.venv/bin:$PATH"

# Install Python MCP servers (uncomment and modify as needed)
# RUN pip install mcp-server-git
# RUN pip install mcp-server-sqlite

# -----------------------------------------------------------------------------
# Configuration Setup
# Uncomment to copy your configuration files
# -----------------------------------------------------------------------------
# COPY --chown=connector:connector configuration/ /etc/connector/

# Optional: Pre-generate configuration if all servers are pre-installed
# This requires that all MCP servers are installed and available in PATH
# RUN /bin/mcp-connector-cli update --outfile /etc/connector/configuration.json

# =============================================================================
# END USER EXTENSION SECTION
# =============================================================================
