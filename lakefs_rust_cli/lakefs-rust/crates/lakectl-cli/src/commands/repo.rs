use crate::cli::RepoCommands;
use anyhow::Result;
use lakefs_api::LakeFSClient;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct RepoRow {
    name: String,
    storage_namespace: String,
    default_branch: String,
    created: String,
}

pub async fn execute(command: RepoCommands, client: LakeFSClient) -> Result<()> {
    match command {
        RepoCommands::Create {
            name,
            storage_namespace,
            default_branch: _,  // Currently unused
        } => {
            let repo = client.create_repository(&name, &storage_namespace).await?;
            println!("Created repository: {}", repo.id);
            println!("Storage namespace: {}", repo.storage_namespace);
            println!("Default branch: {}", repo.default_branch);
        }
        
        RepoCommands::List { amount: _, after: _ } => {
            let response = client.list_repositories().await?;
            
            let rows: Vec<RepoRow> = response
                .results
                .into_iter()
                .map(|r| RepoRow {
                    name: r.id.clone(),
                    storage_namespace: r.storage_namespace,
                    default_branch: r.default_branch,
                    created: r.creation_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();
            
            let table = Table::new(rows);
            println!("{}", table);
            
            if response.pagination.has_more {
                println!(
                    "\nMore results available. Use --after {} to see next page",
                    response.pagination.next_offset.unwrap_or_default()
                );
            }
        }
        
        RepoCommands::Delete { name, yes } => {
            if !yes {
                print!("Are you sure you want to delete repository '{}'? [y/N] ", name);
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if input.trim().to_lowercase() != "y" {
                    println!("Deletion cancelled");
                    return Ok(());
                }
            }
            
            client.delete_repository(&name).await?;
            println!("Deleted repository: {}", name);
        }
    }
    
    Ok(())
}