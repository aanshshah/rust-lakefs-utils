#!/bin/bash

set -e

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Navigate to the correct directory
if [ -d "lakefs_rust_cli/lakefs-rust" ]; then
    cd lakefs_rust_cli/lakefs-rust
elif [ -d "lakefs-rust" ]; then
    cd lakefs-rust
else
    echo "Error: Cannot find lakefs-rust directory"
    exit 1
fi

print_info "Comprehensive fix for all issues..."

# 1. Fix the bytes dependency issue in workspace
print_info "Fixing workspace dependencies..."
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "crates/lakefs-api",
    "crates/lakefs-auth", 
    "crates/lakefs-local",
    "crates/lakectl-cli",
    "crates/lakefs-test-utils",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["lakeFS Rust Implementation"]

[workspace.dependencies]
# Common dependencies
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
bytes = "1.5"  # Added missing dependency

# HTTP client
reqwest = { version = "0.11", features = ["json", "stream", "multipart"] }

# AWS SDK
aws-config = "1.1"
aws-sdk-sts = "1.1"
aws-sigv4 = { version = "1.3", features = ["sign-http"] }
aws-types = "1.1"
aws-credential-types = "1.1"

# CLI dependencies
clap = { version = "4.4", features = ["derive", "env"] }
indicatif = "0.17"
config = "0.13"
home = "0.5"
directories = "5.0"

# Async trait support
async-trait = "0.1"
EOF

# 2. Fix the extra brace in client.rs
print_info "Fixing brace mismatch in client.rs..."
if [ -f "crates/lakefs-api/src/client.rs" ]; then
    # Count braces and remove extra closing brace if needed
    cat > fix_braces.py << 'EOF'
with open('crates/lakefs-api/src/client.rs', 'r') as f:
    content = f.read()

# Count braces
open_braces = content.count('{')
close_braces = content.count('}')

print(f"Open braces: {open_braces}, Close braces: {close_braces}")

if close_braces > open_braces:
    # Remove the last extra closing brace
    last_brace = content.rfind('}')
    content = content[:last_brace] + content[last_brace+1:]
    
    with open('crates/lakefs-api/src/client.rs', 'w') as f:
        f.write(content)
    print("Removed extra closing brace")
else:
    print("Braces are balanced")
EOF
    python3 fix_braces.py
    rm fix_braces.py
fi

# 3. Restore test modules properly
print_info "Restoring test modules..."

# For lakefs-api/src/client.rs
if [ -f "crates/lakefs-api/src/client.rs" ]; then
    # Check if test module exists
    if ! grep -q "#\[cfg(test)\]" crates/lakefs-api/src/client.rs; then
        # Add a basic test module at the end
        cat >> crates/lakefs-api/src/client.rs << 'EOF'

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

    #[tokio::test]
    async fn test_client_creation() {
        let client = LakeFSClient::new("http://localhost:8000", "Bearer test-token");
        assert_eq!(client.base_url, "http://localhost:8000");
        assert_eq!(client.auth_header, "Bearer test-token");
    }
}
EOF
        print_status "Added test module to client.rs"
    fi
fi

# 4. Fix missing imports in other files
print_info "Fixing imports in test modules..."

# Fix lakefs-local/src/changes.rs
if [ -f "crates/lakefs-local/src/changes.rs" ]; then
    if grep -q "#\[cfg(test)\]" crates/lakefs-local/src/changes.rs; then
        # Check if PathType import is missing
        if ! grep -q "use lakefs_api::models::PathType;" crates/lakefs-local/src/changes.rs; then
            # Add import after "use super::*;"
            sed -i.bak '/mod tests {/,/use super::\*;/ s/use super::\*;/use super::*;\n    use lakefs_api::models::PathType;/' crates/lakefs-local/src/changes.rs
            rm crates/lakefs-local/src/changes.rs.bak
        fi
    fi
fi

# Fix lakectl-cli/src/config.rs
if [ -f "crates/lakectl-cli/src/config.rs" ]; then
    if grep -q "#\[cfg(test)\]" crates/lakectl-cli/src/config.rs; then
        # Check if Commands import is missing
        if ! grep -q "use crate::cli::{Commands, RepoCommands};" crates/lakectl-cli/src/config.rs; then
            # Add import after "use super::*;"
            sed -i.bak '/mod tests {/,/use super::\*;/ s/use super::\*;/use super::*;\n    use crate::cli::{Commands, RepoCommands};/' crates/lakectl-cli/src/config.rs
            rm crates/lakectl-cli/src/config.rs.bak
        fi
    fi
fi

# 5. Update Cargo.toml files to use workspace dependencies
print_info "Updating crate dependencies..."

# Update lakefs-api
cat > crates/lakefs-api/Cargo.toml << 'EOF'
[package]
name = "lakefs-api"
version.workspace = true
edition.workspace = true

[dependencies]
# Shared workspace dependencies
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
bytes.workspace = true

# API-specific dependencies
url = "2.5"
futures = "0.3"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.21"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
wiremock = "0.5"
serde_yaml = "0.9"
lakefs-test-utils = { path = "../lakefs-test-utils" }
EOF

# Update lakefs-local
cat > crates/lakefs-local/Cargo.toml << 'EOF'
[package]
name = "lakefs-local"
version.workspace = true
edition.workspace = true

[dependencies]
# API client
lakefs-api = { path = "../lakefs-api" }

# Shared workspace dependencies
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
anyhow.workspace = true
indicatif.workspace = true
bytes.workspace = true

# Local sync dependencies
notify = "6.1"
walkdir = "2.4"
sha2 = "0.10"
chrono = "0.4"
futures = "0.3"
async-trait = "0.1"
ignore = "0.4"
relative-path = "1.9"
path-slash = "0.2"

[dev-dependencies]
tempfile = "3.8"
mockall = "0.12"
lakefs-test-utils = { path = "../lakefs-test-utils" }
EOF

print_status "All fixes applied!"
print_info "Running cargo check to verify..."

# Check if the fixes worked
if cargo check --all-targets; then
    print_status "All compilation errors fixed!"
    echo ""
    echo "Now you can run tests with:"
    echo "  cargo test --all"
else
    print_error "Some errors remain. Please check the output above."
    echo ""
    echo "Try running individual checks:"
    echo "  cargo check -p lakefs-api"
    echo "  cargo check -p lakefs-auth"
    echo "  cargo check -p lakefs-local"
    echo "  cargo check -p lakectl-cli"
fi
