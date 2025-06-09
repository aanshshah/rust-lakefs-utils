#!/bin/bash

set -e

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

cd lakefs-rust

print_info "Fixing missing dependencies in lakectl-cli..."

# Update lakectl-cli Cargo.toml with all required dependencies
cat > crates/lakectl-cli/Cargo.toml << 'EOF'
[package]
name = "lakectl-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "lakectl"
path = "src/main.rs"

[dependencies]
# Local dependencies
lakefs-api = { path = "../lakefs-api" }
lakefs-auth = { path = "../lakefs-auth" }
lakefs-local = { path = "../lakefs-local" }

# Shared workspace dependencies
clap.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
config.workspace = true
home.workspace = true
directories.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
indicatif.workspace = true

# CLI-specific dependencies
colored = "2.0"
tabled = "0.15"
human_bytes = "0.4"
bytes = "1.5"  # Added missing dependency
dirs = "5.0"   # Added missing dependency

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"
serde_yaml = "0.9"
wiremock = "0.5"
lakefs-test-utils = { path = "../lakefs-test-utils" }
EOF

print_status "Updated lakectl-cli Cargo.toml with missing dependencies"

# Also ensure bytes is in the workspace dependencies since it's used in multiple places
print_info "Updating workspace Cargo.toml to include bytes..."

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
bytes = "1.5"  # Added to workspace since it's used in multiple crates

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

print_status "Updated workspace Cargo.toml"

# Update lakefs-local to use workspace bytes dependency
print_info "Updating lakefs-local to use workspace bytes..."

if [ -f "crates/lakefs-local/Cargo.toml" ]; then
    # Backup
    cp crates/lakefs-local/Cargo.toml crates/lakefs-local/Cargo.toml.bak
    
    # Replace "bytes = " with "bytes.workspace = true" if it exists
    sed -i.bak 's/^bytes = .*/bytes.workspace = true/' crates/lakefs-local/Cargo.toml
    
    print_status "Updated lakefs-local to use workspace bytes"
fi

# Update lakefs-api to use workspace bytes dependency
print_info "Updating lakefs-api to use workspace bytes..."

if [ -f "crates/lakefs-api/Cargo.toml" ]; then
    # Backup
    cp crates/lakefs-api/Cargo.toml crates/lakefs-api/Cargo.toml.bak
    
    # Replace "bytes = " with "bytes.workspace = true" if it exists
    sed -i.bak 's/^bytes = .*/bytes.workspace = true/' crates/lakefs-api/Cargo.toml
    
    print_status "Updated lakefs-api to use workspace bytes"
fi

# Clean and rebuild
print_info "Cleaning and rebuilding..."
cargo clean
rm -f Cargo.lock

print_info "Building all crates..."
cargo build --all

print_status "Dependencies fixed!"
echo ""
echo "Now you can run tests with:"
echo "  cargo test --all"
