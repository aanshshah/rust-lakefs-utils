use crate::cli::BranchCommands;
use crate::utils::parse_uri;
use anyhow::Result;
use lakefs_api::LakeFSClient;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct BranchRow {
    name: String,
    commit_id: String,
}

pub async fn execute(command: BranchCommands, client: LakeFSClient) -> Result<()> {
    match command {
        BranchCommands::Create { uri, source } => {
            let parsed = parse_uri(&uri)?;
            let branch_name = parsed.path.ok_or_else(|| {
                anyhow::anyhow!("Invalid branch URI: must include branch name")
            })?;
            
            let branch = client.create_branch(
                &parsed.repository,
                &branch_name,
                &source,
            ).await?;
            
            println!("Created branch: {}", branch.id);
            println!("Commit ID: {}", branch.commit_id);
        }
        
        BranchCommands::List { repository, amount: _ } => {
            let parsed = parse_uri(&repository)?;
            let response = client.list_branches(&parsed.repository).await?;
            
            let rows: Vec<BranchRow> = response
                .results
                .into_iter()
                .map(|b| BranchRow {
                    name: b.id,
                    commit_id: b.commit_id,
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
        
        BranchCommands::Delete { uri, yes } => {
            let parsed = parse_uri(&uri)?;
            
            if !yes {
                print!("Are you sure you want to delete branch '{}'? [y/N] ", 
                      parsed.reference);
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if input.trim().to_lowercase() != "y" {
                    println!("Deletion cancelled");
                    return Ok(());
                }
            }
            
            client.delete_branch(&parsed.repository, &parsed.reference).await?;
            println!("Deleted branch: {}", parsed.reference);
        }
        
        BranchCommands::Show { uri } => {
            let parsed = parse_uri(&uri)?;
            let branch = client.get_branch(&parsed.repository, &parsed.reference).await?;
            
            println!("Branch: {}", branch.id);
            println!("Commit ID: {}", branch.commit_id);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{MockServer, Mock, ResponseTemplate};

    #[tokio::test]
    async fn test_create_branch_command()  {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/repositories/test-repo/branches"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "id": "feature-branch",
                    "commit_id": "abc123"
                })))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        
        let command = BranchCommands::Create {
            uri: "lakefs://test-repo/feature-branch".to_string(),
            source: "main".to_string(),
        };
        
        let result = execute(command, client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_branches_command()  {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/repositories/test-repo/branches"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "results": [{
                        "id": "main",
                        "commit_id": "abc123"
                    }],
                    "pagination": {
                        "has_more": false,
                        "max_per_page": 100,
                        "results": 1,
                        "next_offset": null
                    }
                })))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        
        let command = BranchCommands::List {
            repository: "lakefs://test-repo".to_string(),
            amount: None,
        };
        
        let result = execute(command, client).await;
        assert!(result.is_ok());
    }
}