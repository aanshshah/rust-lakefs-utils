#!/bin/bash

set -e

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

cd lakefs-rust

print_info "Fixing test setup issues..."

# 1. Fix unused import in lakefs-test-utils
print_info "Fixing unused import warning..."
if [ -f "crates/lakefs-test-utils/src/lib.rs" ]; then
    sed -i.bak 's/use chrono::{DateTime, Utc};/use chrono::Utc;/' crates/lakefs-test-utils/src/lib.rs
    print_status "Fixed unused import in lakefs-test-utils"
else
    print_error "Could not find lakefs-test-utils/src/lib.rs"
fi

# 2. Fix workspace dev-dependencies issue
print_info "Fixing workspace dev-dependencies..."
# Remove the workspace.dev-dependencies section as it may not be supported
if grep -q "\[workspace.dev-dependencies\]" Cargo.toml; then
    # Create backup
    cp Cargo.toml Cargo.toml.bak
    
    # Remove the workspace.dev-dependencies section
    sed -i.bak '/\[workspace\.dev-dependencies\]/,/^$/d' Cargo.toml
    print_status "Removed workspace.dev-dependencies section"
fi

# 3. Fix tabled dependency conflict and crate name issues
print_info "Fixing dependency issues..."

# Update lakectl-cli to use correct crate names and consistent versions
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
human_bytes = "0.4"  # Fixed: underscore not hyphen

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"
serde_yaml = "0.9"
wiremock = "0.5"
lakefs-test-utils = { path = "../lakefs-test-utils" }
EOF

# Update the workspace Cargo.toml to include all dependencies
print_info "Updating workspace dependencies..."
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

# Also need to fix the utils.rs file to use human_bytes correctly
print_info "Fixing utils.rs to use human_bytes..."
if [ -f "crates/lakectl-cli/src/utils.rs" ]; then
    # Check current content to preserve existing code
    if grep -q "human_bytes" "crates/lakectl-cli/src/utils.rs"; then
        print_status "utils.rs already uses human_bytes"
    else
        # Update the import
        sed -i.bak 's/use human-bytes/use human_bytes/' "crates/lakectl-cli/src/utils.rs" || true
    fi
fi

# 4. Clean and rebuild
print_info "Cleaning build artifacts..."
cargo clean

# Remove Cargo.lock to ensure fresh dependency resolution
if [ -f "Cargo.lock" ]; then
    rm Cargo.lock
    print_status "Removed Cargo.lock for fresh dependency resolution"
fi

print_info "Rebuilding project..."
cargo build --all

# 5. Fix test command syntax
print_info "Creating corrected test runner..."
cat > run_all_tests.sh << 'EOF'
#!/bin/bash

set -e

echo "Running all tests..."

# Run all tests (correct syntax)
echo "Running unit and integration tests..."
cargo test --all

# Run only unit tests
echo "Running unit tests only..."
cargo test --all --lib

# Run only integration tests
echo "Running integration tests only..."
cargo test --all --test integration_test

echo "All tests completed!"
EOF

chmod +x run_all_tests.sh

print_status "All fixes applied!"
echo ""
echo "Now you can run tests with:"
echo "  ./run_all_tests.sh"
echo "Or:"
echo "  cargo test --all"
echo ""
echo "Note: Fixed the following issues:"
echo "  - Removed unused import in lakefs-test-utils"
echo "  - Fixed human_bytes crate name (underscore not hyphen)"
echo "  - Standardized tabled version to 0.15"
echo "  - Removed Cargo.lock for fresh dependency resolution"
echo "  - Created corrected test runner script"
