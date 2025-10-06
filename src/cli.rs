use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use ndc_mcp_rs::config::{generate_config, Servers, SERVERS_FILE_NAME};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mcp-connector-cli")]
#[command(about = "CLI to generate configuration for MCP connector")]
struct CliArgs {
    #[arg(
        short = 'c',
        long = "configuration",
        value_name = "DIRECTORY",
        help = "configuration directory"
    )]
    configuration: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Update configuration")]
    Update(UpdateCommand),
}

#[derive(Parser)]
struct UpdateCommand {
    #[arg(
        short = 'o',
        long = "outfile",
        value_name = "PATH",
        help = "output file path"
    )]
    outfile: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let config_dir = args
        .configuration
        .or_else(|| {
            std::env::var("HASURA_CONFIGURATION_DIRECTORY")
                .ok()
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from("/etc/connector"));

    match args.command {
        Command::Update(update) => {
            match generate_and_write_config(config_dir, &update.outfile).await {
                Ok(()) => {
                    println!(
                        "Configuration generated successfully at {}",
                        update.outfile.display()
                    );
                }
                Err(error) => {
                    eprintln!("Error generating configuration: {}", error);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

async fn generate_and_write_config(
    config_dir: PathBuf,
    output_file: &PathBuf,
) -> anyhow::Result<()> {
    let servers_file = config_dir.join(SERVERS_FILE_NAME);
    // Check if servers file exists
    if !servers_file.exists() {
        Err(anyhow!(
            "Servers file not found at {}",
            servers_file.display()
        ))?
    }

    // Read servers file as yaml
    let servers: Servers = serde_yaml::from_reader(std::fs::File::open(servers_file)?)?;

    let config = generate_config(servers).await?;
    let config_str = serde_json::to_string_pretty(&config)?;

    // Write config to output file. If it exists, overwrite it.
    std::fs::write(output_file, config_str)?;
    Ok(())
}
