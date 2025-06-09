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

# Navigate to the correct directory
if [ -d "lakefs_rust_cli/lakefs-rust" ]; then
    cd lakefs_rust_cli/lakefs-rust
elif [ -d "lakefs-rust" ]; then
    cd lakefs-rust
else
    echo "Error: Cannot find lakefs-rust directory"
    exit 1
fi

print_info "Manual fix for duplicate test modules..."

# Backup files first
for file in crates/lakefs-api/src/client.rs \
           crates/lakefs-auth/src/auth_provider.rs \
           crates/lakefs-local/src/changes.rs \
           crates/lakectl-cli/src/config.rs \
           crates/lakectl-cli/src/commands/branch.rs; do
    if [ -f "$file" ]; then
        cp "$file" "${file}.backup"
        print_status "Backed up $file"
    fi
done

# Fix lakefs-api/src/client.rs
print_info "Fixing crates/lakefs-api/src/client.rs..."
if [ -f "crates/lakefs-api/src/client.rs" ]; then
    # Remove the duplicate test module (keeping the first one)
    # This is a simple approach - just remove from line 424 to the end
    head -n 423 crates/lakefs-api/src/client.rs > crates/lakefs-api/src/client.rs.tmp
    mv crates/lakefs-api/src/client.rs.tmp crates/lakefs-api/src/client.rs
fi

# Fix lakefs-auth/src/auth_provider.rs  
print_info "Fixing crates/lakefs-auth/src/auth_provider.rs..."
if [ -f "crates/lakefs-auth/src/auth_provider.rs" ]; then
    head -n 75 crates/lakefs-auth/src/auth_provider.rs > crates/lakefs-auth/src/auth_provider.rs.tmp
    mv crates/lakefs-auth/src/auth_provider.rs.tmp crates/lakefs-auth/src/auth_provider.rs
fi

# Fix lakefs-local/src/changes.rs
print_info "Fixing crates/lakefs-local/src/changes.rs..."
if [ -f "crates/lakefs-local/src/changes.rs" ]; then
    # First, add the missing import to the existing test module
    sed -i.bak '/#\[cfg(test)\]/,/mod tests {/ {
        /mod tests {/a\
    use super::*;\
    use lakefs_api::models::PathType;
    }' crates/lakefs-local/src/changes.rs
    
    # Remove the duplicate test module
    head -n 354 crates/lakefs-local/src/changes.rs > crates/lakefs-local/src/changes.rs.tmp
    mv crates/lakefs-local/src/changes.rs.tmp crates/lakefs-local/src/changes.rs
fi

# Fix lakectl-cli/src/config.rs
print_info "Fixing crates/lakectl-cli/src/config.rs..."
if [ -f "crates/lakectl-cli/src/config.rs" ]; then
    # First, add the missing imports to the existing test module
    sed -i.bak '/#\[cfg(test)\]/,/mod tests {/ {
        /mod tests {/a\
    use super::*;\
    use crate::cli::{Commands, RepoCommands};
    }' crates/lakectl-cli/src/config.rs
    
    # Remove the duplicate test module
    head -n 154 crates/lakectl-cli/src/config.rs > crates/lakectl-cli/src/config.rs.tmp
    mv crates/lakectl-cli/src/config.rs.tmp crates/lakectl-cli/src/config.rs
fi

# Fix lakectl-cli/src/commands/branch.rs
print_info "Fixing crates/lakectl-cli/src/commands/branch.rs..."
if [ -f "crates/lakectl-cli/src/commands/branch.rs" ]; then
    # Remove unused import first
    sed -i.bak '/use lakefs_test_utils::mock_server::setup_mock_lakefs;/d' crates/lakectl-cli/src/commands/branch.rs
    
    # Remove the duplicate test module
    head -n 154 crates/lakectl-cli/src/commands/branch.rs > crates/lakectl-cli/src/commands/branch.rs.tmp
    mv crates/lakectl-cli/src/commands/branch.rs.tmp crates/lakectl-cli/src/commands/branch.rs
fi

# Clean up backup files
find . -name "*.bak" -delete

print_status "Manual fixes applied!"
print_info "Running cargo check to verify..."

# Check if the fixes worked
if cargo check --all-targets; then
    print_status "All compilation errors fixed!"
    echo ""
    echo "Now you can run tests with:"
    echo "  cargo test --all"
else
    print_error "Some errors remain. Please check the output above."
fi

print_info "If you need to restore backups, use:"
echo "  for file in crates/**/*.backup; do mv \"\$file\" \"\${file%.backup}\"; done"
