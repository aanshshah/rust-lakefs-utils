mod repo;
mod branch;
mod commit;
mod fs;
mod diff;
mod merge;
mod local;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use anyhow::Result;
use lakefs_api::LakeFSClient;
use lakefs_auth::create_auth_provider;

pub async fn execute(cli: Cli, config: AppConfig) -> Result<()> {
    // Create auth provider
    let auth_provider = create_auth_provider(
        config.credentials.clone(),
        &config.server.endpoint_url,
    ).await?;
    
    // Create client
    let auth_header = auth_provider.get_auth_header().await?;
    let client = LakeFSClient::new(&config.server.endpoint_url, auth_header);
    
    // Execute command
    match cli.command {
        Commands::Repo { command } => repo::execute(command, client).await,
        Commands::Branch { command } => branch::execute(command, client).await,
        Commands::Commit { branch, message, allow_empty } => {
            commit::execute(branch, message, allow_empty, client).await
        }
        Commands::Log { branch, amount } => commit::log(branch, amount, client).await,
        Commands::Fs { command } => fs::execute(command, client, &config.options).await,
        Commands::Diff { left, right } => diff::execute(left, right, client).await,
        Commands::Merge { source, destination, strategy } => {
            merge::execute(source, destination, strategy, client).await
        }
        Commands::Local { command } => local::execute(command, client, &config.options).await,
    }
}
