mod cli;
mod config;
mod commands;
mod utils;

use anyhow::Result;
use clap::Parser;
use crate::cli::Cli;  // Changed from lakectl_cli::cli::Cli

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Load configuration
    let config = config::load_config(&cli)?;
    
    // Execute command
    commands::execute(cli, config).await?;
    
    Ok(())
}