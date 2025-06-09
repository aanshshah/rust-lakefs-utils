use crate::utils::parse_uri;
use anyhow::Result;
use lakefs_api::LakeFSClient;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct CommitRow {
    id: String,
    message: String,
    committer: String,
    date: String,
}

pub async fn execute(
    branch: String,
    message: String,
    _allow_empty: bool,  // Currently unused
    client: LakeFSClient,
) -> Result<()> {
    let uri = parse_uri(&branch)?;
    
    let commit = client.commit(
        &uri.repository,
        &uri.reference,
        &message,
    ).await?;
    
    println!("Created commit: {}", commit.id);
    println!("Message: {}", commit.message);
    println!("Committer: {}", commit.committer);
    println!("Date: {}", commit.creation_date.format("%Y-%m-%d %H:%M:%S"));
    
    Ok(())
}

pub async fn log(branch: String, amount: usize, client: LakeFSClient) -> Result<()> {
    let uri = parse_uri(&branch)?;
    
    let response = client.log_commits(&uri.repository, &uri.reference).await?;
    
    let rows: Vec<CommitRow> = response
        .results
        .into_iter()
        .take(amount)
        .map(|c| CommitRow {
            id: c.id[..8].to_string(), // Show short commit ID
            message: c.message.lines().next().unwrap_or("").to_string(), // First line only
            committer: c.committer,
            date: c.creation_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();
    
    let table = Table::new(rows);
    println!("{}", table);
    
    Ok(())
}