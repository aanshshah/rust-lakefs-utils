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

print_info "Carefully fixing test modules..."

# First, let's restore from backups if they exist
for file in crates/**/*.backup; do
    if [ -f "$file" ]; then
        mv "$file" "${file%.backup}"
        print_status "Restored ${file%.backup}"
    fi
done

# Fix lakefs-api/src/client.rs
print_info "Fixing crates/lakefs-api/src/client.rs..."
if [ -f "crates/lakefs-api/src/client.rs" ]; then
    cat > fix_client.py << 'EOF'
import re

# Read the file
with open('crates/lakefs-api/src/client.rs', 'r') as f:
    content = f.read()

# Find the first test module
first_test_start = content.find('#[cfg(test)]')
if first_test_start == -1:
    print("No test module found")
    exit(1)

# Find where the first test module ends
brace_count = 0
in_module = False
first_test_end = first_test_start

for i in range(first_test_start, len(content)):
    if content[i] == '{':
        brace_count += 1
        in_module = True
    elif content[i] == '}':
        brace_count -= 1
        if in_module and brace_count == 0:
            first_test_end = i + 1
            break

# Find the second test module
second_test_start = content.find('#[cfg(test)]', first_test_end)
if second_test_start == -1:
    print("No duplicate test module found")
    exit(0)

# Extract content up to the second test module
new_content = content[:second_test_start].rstrip()

# Make sure we don't have trailing braces
# Count braces in the remaining content
open_braces = new_content.count('{')
close_braces = new_content.count('}')

# If we have more closing braces, remove the extras from the end
while close_braces > open_braces:
    last_brace = new_content.rfind('}')
    new_content = new_content[:last_brace] + new_content[last_brace+1:]
    close_braces -= 1

# Write the fixed content
with open('crates/lakefs-api/src/client.rs', 'w') as f:
    f.write(new_content + '\n')

print("Fixed crates/lakefs-api/src/client.rs")
EOF
    python3 fix_client.py
    rm fix_client.py
fi

# Fix other files with a general approach
fix_file_tests() {
    local file=$1
    local missing_imports=$2
    
    print_info "Fixing $file..."
    
    cat > fix_file.py << EOF
import re

# Read the file
with open('$file', 'r') as f:
    content = f.read()

# Find all test module positions
test_modules = []
for match in re.finditer(r'#\[cfg\(test\)\]', content):
    test_modules.append(match.start())

if len(test_modules) < 2:
    print(f"No duplicate test modules in $file")
    exit(0)

# Keep only the first test module
# Find where the first test module ends
brace_count = 0
in_module = False
first_test_end = test_modules[0]

for i in range(test_modules[0], len(content)):
    if content[i] == '{':
        brace_count += 1
        in_module = True
    elif content[i] == '}':
        brace_count -= 1
        if in_module and brace_count == 0:
            first_test_end = i + 1
            break

# Extract content up to the second test module
new_content = content[:test_modules[1]].rstrip()

# Add missing imports if needed
if '$missing_imports':
    imports = '$missing_imports'
    # Find the test module content
    test_start = content.find('mod tests {', test_modules[0])
    if test_start != -1:
        # Insert imports after "use super::*;"
        super_import = content.find('use super::*;', test_start)
        if super_import != -1:
            import_end = super_import + len('use super::*;')
            new_content = content[:import_end] + '\\n    ' + imports + content[import_end:test_modules[1]].rstrip()

# Clean up extra closing braces
open_braces = new_content.count('{')
close_braces = new_content.count('}')

while close_braces > open_braces:
    last_brace = new_content.rfind('}')
    new_content = new_content[:last_brace] + new_content[last_brace+1:]
    close_braces -= 1

# Write the fixed content
with open('$file', 'w') as f:
    f.write(new_content + '\\n')

print(f"Fixed $file")
EOF
    python3 fix_file.py
    rm fix_file.py
}

# Fix each file
fix_file_tests "crates/lakefs-auth/src/auth_provider.rs" ""
fix_file_tests "crates/lakefs-local/src/changes.rs" "use lakefs_api::models::PathType;"
fix_file_tests "crates/lakectl-cli/src/config.rs" "use crate::cli::{Commands, RepoCommands};"
fix_file_tests "crates/lakectl-cli/src/commands/branch.rs" ""

# Remove unused imports from branch.rs
print_info "Removing unused imports..."
if [ -f "crates/lakectl-cli/src/commands/branch.rs" ]; then
    sed -i.bak '/use lakefs_test_utils::mock_server::setup_mock_lakefs;/d' crates/lakectl-cli/src/commands/branch.rs
    rm crates/lakectl-cli/src/commands/branch.rs.bak
fi

print_status "All files fixed!"
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
    echo "You can check individual files with:"
    echo "  cargo check -p lakefs-api"
    echo "  cargo check -p lakefs-auth"
    echo "  cargo check -p lakefs-local"
    echo "  cargo check -p lakectl-cli"
fi
