#!/bin/bash

# Color codes
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}Dependency Diagnostic Report${NC}"
echo "============================"
echo ""

cd lakefs-rust

# Check Rust version
echo -e "${GREEN}Rust Version:${NC}"
rustc --version
cargo --version
echo ""

# Check for duplicate dependencies
echo -e "${GREEN}Checking for duplicate dependencies:${NC}"
cargo tree --duplicates || echo "cargo-tree not installed"
echo ""

# Check tabled specifically
echo -e "${GREEN}Tabled dependency usage:${NC}"
grep -r "tabled" crates/*/Cargo.toml | grep -v "path = " || echo "No tabled dependencies found"
echo ""

# Check if workspace.dev-dependencies is supported
echo -e "${GREEN}Checking workspace features:${NC}"
if cargo --version | grep -E "1\.(6[4-9]|[7-9][0-9]|[1-9][0-9][0-9])" > /dev/null; then
    echo "Your Cargo version supports workspace.dev-dependencies"
else
    echo -e "${YELLOW}Your Cargo version may not support workspace.dev-dependencies${NC}"
fi
echo ""

# List all dependencies for lakectl-cli
echo -e "${GREEN}Dependencies for lakectl-cli:${NC}"
cd crates/lakectl-cli
cargo tree || echo "Could not generate dependency tree"
cd ../..
echo ""

# Check for conflicting versions
echo -e "${GREEN}Checking for version conflicts:${NC}"
find . -name "Cargo.lock" -exec grep -H "tabled" {} \; || echo "No Cargo.lock files found"
echo ""

# Suggest fixes
echo -e "${BLUE}Suggested fixes:${NC}"
echo "1. Run: ./fix_test_issues.sh"
echo "2. If issues persist, try:"
echo "   - cargo clean"
echo "   - rm Cargo.lock"
echo "   - cargo build --all"
echo "3. Ensure all crates use the same version of tabled"
echo "4. Consider updating Rust if using an older version"
