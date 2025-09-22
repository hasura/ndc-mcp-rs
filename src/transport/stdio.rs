use anyhow::{Result, anyhow};
use rmcp::{service::RunningService, RoleClient, ServiceExt, transport::TokioChildProcess};
use tokio::process::Command;
use std::collections::HashMap;
use std::path::Path;

use crate::config::StdioConfig;

/// Create an MCP client using stdio transport
pub async fn create_stdio_client(config: &StdioConfig) -> Result<RunningService<RoleClient, ()>> {
    // Extract fields from the config
    // Build command
    let mut cmd = Command::new(&config.command);
    cmd.args(&config.args);

    // Add environment variables
    for (key, value) in &config.env {
        cmd.env(key, value);
    }

    // Load environment variables from file if specified
    if let Some(env_file) = &config.env_file {
        load_env_file(env_file, &mut cmd)?;
    }

    // Create the child process
    let child_process = TokioChildProcess::new(cmd)
        .map_err(|e| anyhow!("Failed to start MCP server: {}", e))?;

    // Create and initialize the client with timeout
    let service = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        ().serve(child_process)
    ).await
    .map_err(|_| anyhow!("Timeout during MCP service initialization"))?
    .map_err(|e| anyhow!("Failed to initialize MCP service: {}", e))?;

    Ok(service)
}

/// Load environment variables from a .env file
fn load_env_file(env_file: &String, cmd: &mut Command) -> Result<()> {
    let path = Path::new(env_file);
    if !path.exists() {
        return Err(anyhow!("Environment file not found: {}", env_file));
    }

    // Set the path for dotenv
    dotenv::from_path(path)?;

    // Get all environment variables
    let vars: HashMap<String, String> = dotenv::vars().collect();

    // Add them to the command
    for (key, value) in vars {
        cmd.env(key, value);
    }

    Ok(())
}
