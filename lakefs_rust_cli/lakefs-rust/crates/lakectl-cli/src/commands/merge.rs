use crate::utils::parse_uri;
use anyhow::Result;
use lakefs_api::LakeFSClient;

pub async fn execute(
    source: String,
    destination: String,
    _strategy: Option<String>,  // Currently unused
    client: LakeFSClient,
) -> Result<()> {
    let source_uri = parse_uri(&source)?;
    let dest_uri = parse_uri(&destination)?;
    
    if source_uri.repository != dest_uri.repository {
        anyhow::bail!("Cannot merge across different repositories");
    }
    
    let merge_result = client.merge(
        &source_uri.repository,
        &source_uri.reference,
        &dest_uri.reference,
    ).await?;
    
    println!("Merged {} into {}", source, destination);
    println!("Merge commit: {}", merge_result.id);
    
    Ok(())
}