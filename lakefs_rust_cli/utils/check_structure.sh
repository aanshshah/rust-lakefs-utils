#!/bin/bash

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

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

echo "Checking test module structure..."
echo ""

for file in crates/lakefs-api/src/client.rs \
           crates/lakefs-auth/src/auth_provider.rs \
           crates/lakefs-local/src/changes.rs \
           crates/lakectl-cli/src/config.rs \
           crates/lakectl-cli/src/commands/branch.rs; do
    
    if [ -f "$file" ]; then
        print_info "Checking $file:"
        echo -n "  Number of #[cfg(test)] occurrences: "
        grep -c "#\[cfg(test)\]" "$file" || echo "0"
        
        echo -n "  Open braces: "
        grep -o "{" "$file" | wc -l
        
        echo -n "  Close braces: "
        grep -o "}" "$file" | wc -l
        
        echo ""
    fi
done

# Check compilation
print_info "Checking compilation..."
echo ""

for package in lakefs-api lakefs-auth lakefs-local lakectl-cli; do
    echo -n "  $package: "
    if cargo check -p "$package" 2>/dev/null; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
    fi
done
