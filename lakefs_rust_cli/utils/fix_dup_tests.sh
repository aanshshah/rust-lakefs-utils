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

print_info "Fixing duplicate test modules and imports..."

# Function to remove duplicate test modules from a file
fix_duplicate_tests() {
    local file=$1
    local temp_file="${file}.tmp"
    
    if [ ! -f "$file" ]; then
        print_error "File not found: $file"
        return
    fi
    
    print_info "Processing $file..."
    
    # Create a Python script to fix the file
    cat > fix_tests.py << 'EOF'
import sys
import re

def extract_balanced_content(text, start_pos):
    """Extract content between balanced braces"""
    if start_pos >= len(text):
        return ""
    
    # Skip until we find the opening brace
    while start_pos < len(text) and text[start_pos] != '{':
        start_pos += 1
    
    if start_pos >= len(text):
        return ""
    
    count = 0
    end_pos = start_pos
    
    while end_pos < len(text):
        if text[end_pos] == '{':
            count += 1
        elif text[end_pos] == '}':
            count -= 1
            if count == 0:
                return text[start_pos:end_pos + 1]
        end_pos += 1
    
    return text[start_pos:]

def fix_file(filename):
    with open(filename, 'r') as f:
        content = f.read()
    
    # Find all test modules
    test_pattern = r'#\[cfg\(test\)\]\s*\nmod tests'
    test_starts = []
    for match in re.finditer(test_pattern, content):
        test_starts.append(match.start())
    
    if len(test_starts) <= 1:
        print(f"No duplicate test modules in {filename}")
        return False
    
    print(f"Found {len(test_starts)} test modules in {filename}")
    
    # Extract all test modules
    test_modules = []
    for start in test_starts:
        module_content = extract_balanced_content(content, content.find('{', start))
        if module_content:
            test_modules.append((start, module_content))
    
    # Extract all test functions and imports from all modules
    all_tests = []
    imports = set()
    
    for _, module_content in test_modules:
        # Extract imports
        import_pattern = r'use\s+[^;]+;'
        for imp in re.findall(import_pattern, module_content):
            imports.add(imp.strip())
        
        # Extract individual test functions
        # Find all test attributes and their following functions
        test_attr_pattern = r'(#\[(?:tokio::)?test\])'
        test_fn_pattern = r'((?:async\s+)?fn\s+\w+\s*\([^)]*\)\s*(?:->\s*[^{]+)?)'
        
        # First, find all test attributes
        attr_positions = [(m.group(1), m.end()) for m in re.finditer(test_attr_pattern, module_content)]
        
        for i, (attr, attr_end) in enumerate(attr_positions):
            # Find the function definition after this attribute
            remaining = module_content[attr_end:]
            fn_match = re.search(test_fn_pattern, remaining)
            
            if fn_match:
                fn_start = attr_end + fn_match.start()
                fn_def = fn_match.group(1)
                
                # Extract the function body
                body_start = fn_start + len(fn_def)
                body = extract_balanced_content(module_content, body_start)
                
                if body:
                    full_test = f"    {attr}\n    {fn_def} {body}"
                    if full_test not in all_tests:
                        all_tests.append(full_test)
    
    # Remove all test modules
    # Sort in reverse order to avoid index issues
    test_modules.sort(key=lambda x: x[0], reverse=True)
    
    new_content = content
    for start, module in test_modules:
        # Find the full module including #[cfg(test)]
        module_start = content.rfind('#[cfg(test)]', 0, start)
        module_end = start + len(extract_balanced_content(content, content.find('{', start)))
        
        # Also include any trailing newlines
        while module_end < len(content) and content[module_end] in '\n\r':
            module_end += 1
        
        new_content = new_content[:module_start] + new_content[module_end:]
    
    # Add a single test module with all tests
    imports_str = '\n    '.join(sorted(imports))
    tests_str = '\n\n'.join(all_tests)
    
    test_module = f'''
#[cfg(test)]
mod tests {{
    use super::*;
    {imports_str}

{tests_str}
}}'''
    
    new_content = new_content.rstrip() + '\n' + test_module + '\n'
    
    # Save the fixed file
    with open(filename, 'w') as f:
        f.write(new_content)
    
    return True

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python fix_tests.py <filename>")
        sys.exit(1)
    
    try:
        if fix_file(sys.argv[1]):
            print(f"Fixed {sys.argv[1]}")
        else:
            print(f"No changes needed for {sys.argv[1]}")
    except Exception as e:
        print(f"Error processing {sys.argv[1]}: {e}")
        sys.exit(1)
EOF
    
    # Run the Python script
    python3 fix_tests.py "$file"
    rm fix_tests.py
}

# Fix all files with duplicate test modules
fix_duplicate_tests "crates/lakefs-api/src/client.rs"
fix_duplicate_tests "crates/lakefs-auth/src/auth_provider.rs"
fix_duplicate_tests "crates/lakefs-local/src/changes.rs"
fix_duplicate_tests "crates/lakectl-cli/src/config.rs"
fix_duplicate_tests "crates/lakectl-cli/src/commands/branch.rs"

# Fix missing imports in lakefs-local/changes.rs
print_info "Fixing missing imports in lakefs-local/changes.rs..."
if [ -f "crates/lakefs-local/src/changes.rs" ]; then
    # Check if PathType import is already there
    if ! grep -q "use lakefs_api::models::PathType;" crates/lakefs-local/src/changes.rs; then
        # Add the PathType import at the top of the test module
        sed -i.bak '/mod tests {/,/use super::\*;/ s/use super::\*;/use super::*;\n    use lakefs_api::models::PathType;/' crates/lakefs-local/src/changes.rs
    fi
fi

# Fix missing imports in lakectl-cli/config.rs
print_info "Fixing missing imports in lakectl-cli/config.rs..."
if [ -f "crates/lakectl-cli/src/config.rs" ]; then
    # Check if imports are already there
    if ! grep -q "use crate::cli::{Commands, RepoCommands};" crates/lakectl-cli/src/config.rs; then
        # Add the missing imports at the top of the test module
        sed -i.bak '/mod tests {/,/use super::\*;/ s/use super::\*;/use super::*;\n    use crate::cli::{Commands, RepoCommands};/' crates/lakectl-cli/src/config.rs
    fi
fi

# Remove unused imports
print_info "Removing unused imports..."
if [ -f "crates/lakectl-cli/src/commands/branch.rs" ]; then
    sed -i.bak '/use lakefs_test_utils::mock_server::setup_mock_lakefs;/d' crates/lakectl-cli/src/commands/branch.rs
fi

# Clean up backup files
find . -name "*.bak" -delete

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
fi
