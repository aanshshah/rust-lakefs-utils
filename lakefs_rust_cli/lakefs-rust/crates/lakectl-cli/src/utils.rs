use colored::Colorize;
use human_bytes::human_bytes;
use lakefs_api::LakeFSUri;
use std::str::FromStr;

pub fn parse_uri(uri: &str) -> anyhow::Result<LakeFSUri> {
    LakeFSUri::from_str(uri)
        .map_err(|e| anyhow::anyhow!("Invalid URI '{}': {}", uri, e))
}

pub fn format_size(bytes: i64) -> String {
    human_bytes(bytes as f64)
}

pub fn format_diff_type(diff_type: &str) -> String {
    match diff_type {
        "added" => "+".green().to_string(),
        "removed" => "-".red().to_string(),
        "changed" => "~".yellow().to_string(),
        _ => diff_type.to_string(),
    }
}

pub fn confirm(prompt: &str) -> anyhow::Result<bool> {
    use std::io::{self, Write};
    
    print!("{} [y/N] ", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_lowercase() == "y")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uri_valid() {
        let uri = parse_uri("lakefs://repo/branch/path").unwrap();
        assert_eq!(uri.repository, "repo");
        assert_eq!(uri.reference, "branch");
        assert_eq!(uri.path, Some("path".to_string()));
    }

    #[test]
    fn test_parse_uri_invalid() {
        assert!(parse_uri("invalid://uri").is_err());
        assert!(parse_uri("lakefs://").is_err());
    }

    #[test]
    fn test_format_size() {
        // The human_bytes function returns strings with specific formatting
        let kb = format_size(1024);
        assert!(kb.contains("1"));
        
        let mb = format_size(1024 * 1024);
        assert!(mb.contains("1"));
        
        let b = format_size(100);
        assert!(b.contains("100"));
    }
    #[test]
    fn test_format_diff_type() {
        // Basic test for format_diff_type
        let result = format_diff_type("added");
        assert!(!result.is_empty());
    }
}
