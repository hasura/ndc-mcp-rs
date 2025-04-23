mod config;
mod connector;
mod schema;
mod state;
mod transport;

use std::process::ExitCode;
use ndc_sdk::default_main::default_main_with;

/// Run the NDC MCP connector using the default_main_with function from ndc-sdk
#[tokio::main]
async fn main() -> ExitCode {
    match default_main_with(connector::McpConnectorSetup).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
