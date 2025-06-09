#!/bin/bash

# Don't exit on error - we want to collect all errors
# set -e

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Error tracking
declare -a ERRORS
declare -a WARNINGS
SETUP_SUCCESS=true

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
    WARNINGS+=("$1")
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
    ERRORS+=("$1")
    SETUP_SUCCESS=false
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

# Detect OS for sed compatibility
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    SED_INPLACE=(-i '')
    OS_TYPE="macOS"
else
    # Linux
    SED_INPLACE=(-i)
    OS_TYPE="Linux"
fi

print_info "Detected OS: $OS_TYPE"

# Check if we're in the right directory
if [ ! -d "lakefs-rust" ]; then
    print_error "lakefs-rust directory not found. Please run this script from the parent directory."
    echo "Current directory: $(pwd)"
    echo "Expected to find: lakefs-rust/"
    exit 1
fi

cd lakefs-rust

print_status "Starting test setup for lakeFS Rust project..."

# Create the test utilities crate
print_info "Creating test utilities crate..."
if ! mkdir -p crates/lakefs-test-utils/src; then
    print_error "Failed to create directory: crates/lakefs-test-utils/src"
else
    print_status "Created test utilities directory"
fi

# Create Cargo.toml for test utils
print_info "Creating test utilities Cargo.toml..."
if ! cat > crates/lakefs-test-utils/Cargo.toml << 'EOF'
[package]
name = "lakefs-test-utils"
version.workspace = true
edition.workspace = true

[dependencies]
lakefs-api = { path = "../lakefs-api" }
chrono = "0.4"
wiremock = "0.5"
serde_json = "1.0"
tempfile = "3.8"
EOF
then
    print_error "Failed to create crates/lakefs-test-utils/Cargo.toml"
else
    print_status "Created test utilities Cargo.toml"
fi

# Create test utilities lib.rs
print_info "Creating test utilities lib.rs..."
if ! cat > crates/lakefs-test-utils/src/lib.rs << 'EOF'
use lakefs_api::models::{Repository, Branch, Commit, ObjectStats, PathType};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub mod fixtures {
    use super::*;
    
    pub fn test_repository() -> Repository {
        Repository {
            id: "test-repo".to_string(),
            storage_namespace: "s3://test-bucket".to_string(),
            default_branch: "main".to_string(),
            creation_date: Utc::now(),
        }
    }
    
    pub fn test_branch() -> Branch {
        Branch {
            id: "test-branch".to_string(),
            commit_id: "abc123".to_string(),
        }
    }
    
    pub fn test_commit() -> Commit {
        Commit {
            id: "abc123".to_string(),
            parents: vec!["parent1".to_string()],
            committer: "test@example.com".to_string(),
            message: "Test commit".to_string(),
            creation_date: Utc::now(),
            meta_range_id: "meta123".to_string(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn test_object_stats() -> ObjectStats {
        ObjectStats {
            path: "test/file.txt".to_string(),
            path_type: PathType::Object,
            physical_address: "s3://bucket/object".to_string(),
            checksum: "sha256:123abc".to_string(),
            size_bytes: 1024,
            mtime: Utc::now(),
            metadata: None,
        }
    }
}

pub mod mock_server {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use serde_json::json;
    
    pub async fn setup_mock_lakefs() -> MockServer {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/repositories"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "results": [],
                    "pagination": {
                        "has_more": false,
                        "max_per_page": 100,
                        "results": 0
                    }
                })))
            .mount(&mock_server)
            .await;
            
        mock_server
    }
}

pub mod test_helpers {
    use tempfile::TempDir;
    use std::fs;
    use std::path::Path;
    
    pub fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> std::io::Result<()> {
        let file_path = dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)
    }
    
    pub fn setup_test_directory() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        create_test_file(temp_dir.path(), "file1.txt", b"content1").unwrap();
        create_test_file(temp_dir.path(), "dir/file2.txt", b"content2").unwrap();
        create_test_file(temp_dir.path(), ".hidden", b"hidden").unwrap();
        
        temp_dir
    }
}
EOF
then
    print_error "Failed to create crates/lakefs-test-utils/src/lib.rs"
else
    print_status "Created test utilities lib.rs"
fi

# Add test-utils to workspace members
print_info "Adding test-utils to workspace..."
if grep -q "lakefs-test-utils" Cargo.toml; then
    print_warning "test-utils already in workspace"
else
    # Try to modify the Cargo.toml
    if [ -f "Cargo.toml" ]; then
        # Create a backup
        cp Cargo.toml Cargo.toml.backup
        
        # Read the file content
        content=$(cat Cargo.toml)
        
        # Try to add the test-utils to members
        if echo "$content" | grep -q 'members = \['; then
            # Replace the members array - more robust approach
            if python3 -c "
import re
content = '''$content'''
# Add lakefs-test-utils to the members list
pattern = r'(members\s*=\s*\[)'
replacement = r'\1\n    \"crates/lakefs-test-utils\",'
new_content = re.sub(pattern, replacement, content)
print(new_content)
" > Cargo.toml.new; then
                mv Cargo.toml.new Cargo.toml
                print_status "Added test-utils to workspace"
            else
                print_error "Failed to add test-utils to workspace - Python approach failed"
                mv Cargo.toml.backup Cargo.toml
            fi
        else
            print_error "Could not find 'members = [' in Cargo.toml"
        fi
    else
        print_error "Cargo.toml not found"
    fi
fi

# Function to add dev-dependencies to Cargo.toml
add_dev_dependencies() {
    local crate_path=$1
    local deps=$2
    local crate_name=$(basename "$crate_path")
    
    if [ ! -f "$crate_path/Cargo.toml" ]; then
        print_error "Cargo.toml not found for $crate_name at $crate_path/Cargo.toml"
        return 1
    fi
    
    if grep -q "\[dev-dependencies\]" "$crate_path/Cargo.toml"; then
        print_warning "Dev dependencies section already exists in $crate_name"
    else
        if echo -e "\n[dev-dependencies]\n$deps" >> "$crate_path/Cargo.toml"; then
            print_status "Added dev dependencies to $crate_name"
        else
            print_error "Failed to add dev dependencies to $crate_name"
        fi
    fi
}

# Add dev dependencies to each crate
print_info "Adding dev dependencies to crates..."

add_dev_dependencies "crates/lakefs-api" 'tokio-test = "0.4"
mockall = "0.12"
wiremock = "0.5"
serde_yaml = "0.9"
lakefs-test-utils = { path = "../lakefs-test-utils" }'

add_dev_dependencies "crates/lakefs-auth" 'tokio-test = "0.4"
wiremock = "0.5"
lakefs-test-utils = { path = "../lakefs-test-utils" }'

add_dev_dependencies "crates/lakefs-local" 'tempfile = "3.8"
mockall = "0.12"
lakefs-test-utils = { path = "../lakefs-test-utils" }'

add_dev_dependencies "crates/lakectl-cli" 'tokio-test = "0.4"
tempfile = "3.8"
serde_yaml = "0.9"
wiremock = "0.5"
human-bytes = "0.4"
colored = "2.0"
tabled = "0.12"
lakefs-test-utils = { path = "../lakefs-test-utils" }'

# Add workspace dev-dependencies
if ! grep -q "\[workspace.dev-dependencies\]" "Cargo.toml"; then
    print_info "Adding workspace dev dependencies..."
    if cat >> Cargo.toml << 'EOF'

[workspace.dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
wiremock = "0.5"
tempfile = "3.8"
serde_yaml = "0.9"
EOF
    then
        print_status "Added workspace dev dependencies"
    else
        print_error "Failed to add workspace dev dependencies"
    fi
else
    print_warning "Workspace dev dependencies already exist"
fi

# Function to add tests to a source file
add_tests_to_file() {
    local file_path=$1
    local test_content=$2
    local file_desc=$3
    
    if [ -f "$file_path" ]; then
        if grep -q "#\[cfg(test)\]" "$file_path"; then
            print_warning "Tests already exist in $file_desc ($file_path)"
        else
            # Create a backup
            cp "$file_path" "$file_path.backup"
            
            if echo -e "\n$test_content" >> "$file_path"; then
                print_status "Added tests to $file_desc"
            else
                print_error "Failed to add tests to $file_desc"
                mv "$file_path.backup" "$file_path"
            fi
        fi
    else
        print_error "File not found: $file_desc ($file_path)"
    fi
}

# Add tests to lakefs-api
print_info "Adding tests to lakefs-api..."

add_tests_to_file "crates/lakefs-api/src/uri.rs" '#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_from_str_with_path() {
        let uri = LakeFSUri::from_str("lakefs://my-repo/my-branch/path/to/file").unwrap();
        assert_eq!(uri.repository, "my-repo");
        assert_eq!(uri.reference, "my-branch");
        assert_eq!(uri.path, Some("path/to/file".to_string()));
    }

    #[test]
    fn test_uri_from_str_without_path() {
        let uri = LakeFSUri::from_str("lakefs://my-repo/my-branch").unwrap();
        assert_eq!(uri.repository, "my-repo");
        assert_eq!(uri.reference, "my-branch");
        assert_eq!(uri.path, None);
    }

    #[test]
    fn test_uri_from_str_invalid() {
        assert!(LakeFSUri::from_str("invalid://uri").is_err());
        assert!(LakeFSUri::from_str("lakefs://").is_err());
        assert!(LakeFSUri::from_str("lakefs://repo").is_err());
    }

    #[test]
    fn test_uri_to_string() {
        let uri = LakeFSUri::new("repo", "branch").with_path("path/to/file");
        assert_eq!(uri.to_string(), "lakefs://repo/branch/path/to/file");
        
        let uri_no_path = LakeFSUri::new("repo", "branch");
        assert_eq!(uri_no_path.to_string(), "lakefs://repo/branch");
    }
}' "lakefs-api/uri.rs"

add_tests_to_file "crates/lakefs-api/src/models.rs" '#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            id: "test-repo".to_string(),
            storage_namespace: "s3://bucket/path".to_string(),
            default_branch: "main".to_string(),
            creation_date: Utc::now(),
        };

        let json = serde_json::to_string(&repo).unwrap();
        let deserialized: Repository = serde_json::from_str(&json).unwrap();

        assert_eq!(repo.id, deserialized.id);
        assert_eq!(repo.storage_namespace, deserialized.storage_namespace);
        assert_eq!(repo.default_branch, deserialized.default_branch);
    }

    #[test]
    fn test_diff_type_display() {
        assert_eq!(DiffType::Added.to_string(), "added");
        assert_eq!(DiffType::Removed.to_string(), "removed");
        assert_eq!(DiffType::Changed.to_string(), "changed");
    }

    #[test]
    fn test_path_type_serialization() {
        let obj = PathType::Object;
        let dir = PathType::Directory;

        assert_eq!(serde_json::to_string(&obj).unwrap(), "\"object\"");
        assert_eq!(serde_json::to_string(&dir).unwrap(), "\"directory\"");
    }
}' "lakefs-api/models.rs"

# Add tests to lakefs-auth
print_info "Adding tests to lakefs-auth..."

add_tests_to_file "crates/lakefs-auth/src/basic.rs" '#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_auth_header() {
        let auth = BasicAuth::new("user".to_string(), "pass".to_string());
        let header = auth.get_auth_header().await.unwrap();
        
        // "user:pass" base64 encoded is "dXNlcjpwYXNz"
        assert_eq!(header, "Basic dXNlcjpwYXNz");
    }

    #[tokio::test]
    async fn test_basic_auth_special_chars() {
        let auth = BasicAuth::new("user@example.com".to_string(), "p@$$w0rd!".to_string());
        let header = auth.get_auth_header().await.unwrap();
        
        assert!(header.starts_with("Basic "));
        
        // Decode and verify
        let encoded = header.strip_prefix("Basic ").unwrap();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        
        assert_eq!(decoded_str, "user@example.com:p@$$w0rd!");
    }
}' "lakefs-auth/basic.rs"

# Add tests to lakefs-local
print_info "Adding tests to lakefs-local..."

add_tests_to_file "crates/lakefs-local/src/index.rs" '#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_new_index() {
        let index = LocalIndex::new("test-repo", "main", "commit123");
        
        assert_eq!(index.version, LocalIndex::VERSION);
        assert_eq!(index.repository, "test-repo");
        assert_eq!(index.reference, "main");
        assert_eq!(index.head_commit, "commit123");
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_index_entry_operations() {
        let mut index = LocalIndex::new("test-repo", "main", "commit123");
        
        let entry = IndexEntry {
            path: "test.txt".to_string(),
            checksum: "abc123".to_string(),
            size: 1024,
            mtime: Utc::now(),
            permissions: Some(0o644),
        };
        
        // Add entry
        index.add_entry("test.txt".to_string(), entry.clone());
        assert_eq!(index.entries.len(), 1);
        
        // Get entry
        let retrieved = index.get_entry("test.txt").unwrap();
        assert_eq!(retrieved.checksum, "abc123");
        assert_eq!(retrieved.size, 1024);
        
        // Remove entry
        let removed = index.remove_entry("test.txt").unwrap();
        assert_eq!(removed.checksum, "abc123");
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_index_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        // Create and save index
        let mut index = LocalIndex::new("test-repo", "main", "commit123");
        index.add_entry("file1.txt".to_string(), IndexEntry {
            path: "file1.txt".to_string(),
            checksum: "checksum1".to_string(),
            size: 100,
            mtime: Utc::now(),
            permissions: None,
        });
        
        index.save(path).unwrap();
        
        // Load index
        let loaded = LocalIndex::load(path).unwrap();
        
        assert_eq!(loaded.repository, "test-repo");
        assert_eq!(loaded.reference, "main");
        assert_eq!(loaded.head_commit, "commit123");
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.get_entry("file1.txt").unwrap().checksum, "checksum1");
    }
}' "lakefs-local/index.rs"

# Add tests to lakectl-cli
print_info "Adding tests to lakectl-cli..."

add_tests_to_file "crates/lakectl-cli/src/utils.rs" '#[cfg(test)]
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
        assert_eq!(format_size(1024), "1 KiB");
        assert_eq!(format_size(1024 * 1024), "1 MiB");
        assert_eq!(format_size(100), "100 B");
    }

    #[test]
    fn test_format_diff_type() {
        // Basic test for format_diff_type
        let result = format_diff_type("added");
        assert!(!result.is_empty());
    }
}' "lakectl-cli/utils.rs"

# Create integration tests directory
print_info "Creating integration tests..."
if mkdir -p tests; then
    print_status "Created tests directory"
else
    print_error "Failed to create tests directory"
fi

if cat > tests/integration_test.rs << 'EOF'
use lakefs_test_utils::mock_server::setup_mock_lakefs;
use lakefs_api::LakeFSClient;

#[tokio::test]
async fn test_client_integration() {
    let mock_server = setup_mock_lakefs().await;
    let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
    
    // Test listing repositories
    let repos = client.list_repositories().await.unwrap();
    assert_eq!(repos.results.len(), 0);
}
EOF
then
    print_status "Created integration test"
else
    print_error "Failed to create integration test"
fi

# Create GitHub Actions workflow
print_info "Creating GitHub Actions workflow..."
if mkdir -p .github/workflows; then
    if cat > .github/workflows/test.yml << 'EOF'
name: Test

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt --all -- --check
      
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
      
    - name: Run tests
      run: cargo test --all --verbose
      
    - name: Run integration tests
      run: cargo test --all --test '*' --verbose

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      
    - name: Install cargo-tarpaulin
      run: cargo install cargo-tarpaulin
      
    - name: Generate code coverage
      run: cargo tarpaulin --out Xml --all-features --workspace
      
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./cobertura.xml
        flags: unittests
        name: codecov-umbrella
        fail_ci_if_error: true
EOF
    then
        print_status "Created GitHub Actions workflow"
    else
        print_error "Failed to create GitHub Actions workflow"
    fi
else
    print_error "Failed to create .github/workflows directory"
fi

# Create a test runner script
print_info "Creating test runner script..."
if cat > run_tests.sh << 'EOF'
#!/bin/bash

set -e

echo "Running all tests..."

# Format check
echo "Checking code formatting..."
cargo fmt --all -- --check

# Clippy check
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Run unit tests
echo "Running unit tests..."
cargo test --all --lib

# Run integration tests
echo "Running integration tests..."
cargo test --all --test '*'

# Run doc tests
echo "Running doc tests..."
cargo test --all --doc

# Generate coverage report (optional, requires cargo-tarpaulin)
if command -v cargo-tarpaulin &> /dev/null; then
    echo "Generating coverage report..."
    cargo tarpaulin --out Html --all-features --workspace
    echo "Coverage report generated at target/tarpaulin/tarpaulin-report.html"
else
    echo "Skipping coverage report (cargo-tarpaulin not installed)"
fi

echo "All tests passed!"
EOF
then
    chmod +x run_tests.sh
    print_status "Created test runner script"
else
    print_error "Failed to create test runner script"
fi

# Create a README for tests
print_info "Creating test README..."
if cat > tests/README.md << 'EOF'
# LakeFS Rust Project Tests

This directory contains tests for the LakeFS Rust implementation.

## Running Tests

### All Tests
```bash
./run_tests.sh
```

### Unit Tests Only
```bash
cargo test --all --lib
```

### Integration Tests Only
```bash
cargo test --all --test '*'
```

### Specific Crate Tests
```bash
cargo test -p lakefs-api
cargo test -p lakefs-auth
cargo test -p lakefs-local
cargo test -p lakectl-cli
```

### With Output
```bash
cargo test --all -- --nocapture
```

### Coverage Report
First install cargo-tarpaulin:
```bash
cargo install cargo-tarpaulin
```

Then generate the report:
```bash
cargo tarpaulin --out Html --all-features --workspace
```

## Test Structure

- Unit tests are located within each module's source file
- Integration tests are in the `tests/` directory
- Test utilities are in the `lakefs-test-utils` crate
- GitHub Actions workflow is in `.github/workflows/test.yml'

## Continuous Integration

Tests are automatically run on every push and pull request via GitHub Actions.
The CI pipeline includes:
- Code formatting check
- Clippy linting
- All unit and integration tests
- Code coverage reporting
EOF
then
    print_status "Created test README"
else
    print_error "Failed to create test README"
fi

# Final summary
echo ""
echo "============================================="
echo "           Test Setup Summary"
echo "============================================="

if [ ${#ERRORS[@]} -eq 0 ] && [ ${#WARNINGS[@]} -eq 0 ]; then
    print_status "Test setup completed successfully with no issues!"
else
    if [ ${#WARNINGS[@]} -gt 0 ]; then
        echo ""
        echo -e "${YELLOW}Warnings:${NC}"
        for warning in "${WARNINGS[@]}"; do
            echo "  - $warning"
        done
    fi
    
    if [ ${#ERRORS[@]} -gt 0 ]; then
        echo ""
        echo -e "${RED}Errors encountered:${NC}"
        for error in "${ERRORS[@]}"; do
            echo "  - $error"
        done
        
        echo ""
        echo -e "${BLUE}Suggested fixes:${NC}"
        
        # Provide specific fixes for common errors
        for error in "${ERRORS[@]}"; do
            case "$error" in
                *"Failed to add test-utils to workspace"*)
                    echo "  - Manually edit Cargo.toml and add \"crates/lakefs-test-utils\" to the members array"
                    ;;
                *"File not found:"*)
                    echo "  - Check if the source files exist in the expected locations"
                    echo "  - Verify the project structure matches what the script expects"
                    ;;
                *"Failed to create"*)
                    echo "  - Check file permissions in the project directory"
                    echo "  - Ensure you have write access to all directories"
                    ;;
                *"Python approach failed"*)
                    echo "  - Install Python 3 if not available"
                    echo "  - Or manually edit Cargo.toml to add the test-utils crate"
                    ;;
            esac
        done
    fi
fi

echo ""
echo "Next steps:"
if [ "$SETUP_SUCCESS" = true ]; then
    echo "  cd lakefs-rust"
    echo "  cargo build --all            # Build all crates"
    echo "  ./run_tests.sh              # Run all tests"
else
    echo "  1. Fix the errors listed above"
    echo "  2. Re-run this setup script"
    echo "  3. Or manually complete the setup following the error messages"
fi
echo ""

# Create a setup report file
cat > test_setup_report.txt << EOF
Test Setup Report
Generated: $(date)

OS: $OS_TYPE
Directory: $(pwd)

Summary:
- Warnings: ${#WARNINGS[@]}
- Errors: ${#ERRORS[@]}

Warnings:
$(printf '%s\n' "${WARNINGS[@]}")

Errors:
$(printf '%s\n' "${ERRORS[@]}")

Files created/modified:
- crates/lakefs-test-utils/
- tests/
- .github/workflows/test.yml
- run_tests.sh
- Various source files with test modules

Next steps:
1. Review this report
2. Fix any errors
3. Run: cd lakefs-rust && cargo build --all
4. Run: ./run_tests.sh
EOF

print_info "Setup report saved to: test_setup_report.txt"
