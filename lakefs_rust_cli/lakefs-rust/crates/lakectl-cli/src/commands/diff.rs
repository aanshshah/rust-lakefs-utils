use crate::utils::{parse_uri, format_diff_type};
use anyhow::Result;
use lakefs_api::LakeFSClient;

pub async fn execute(
    left: String,
    right: Option<String>,
    client: LakeFSClient,
) -> Result<()> {
    let left_uri = parse_uri(&left)?;
    
    let (right_repo, right_ref) = match &right {
        Some(r) => {
            let right_uri = parse_uri(r)?;
            (right_uri.repository, right_uri.reference)
        }
        None => {
            // If no right reference provided, assume working tree
            (left_uri.repository.clone(), "~".to_string())
        }
    };
    
    if left_uri.repository != right_repo {
        anyhow::bail!("Cannot diff across different repositories");
    }
    
    let diff_result = client.diff(
        &left_uri.repository,
        &left_uri.reference,
        &right_ref,
    ).await?;
    
    if diff_result.results.is_empty() {
        println!("No differences found");
        return Ok(());
    }
    
    let right_str = right.as_deref().unwrap_or("working tree");
    println!("Differences between {} and {}:", left, right_str);
    println!();
    
    for diff in diff_result.results {
        let diff_type = format_diff_type(&diff.diff_type.to_string());
        println!("{} {}", diff_type, diff.path);
    }
    
    Ok(())
}