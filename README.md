# lakeFS Rust Implementation

A Rust implementation of the lakeFS client and CLI, providing a fast and type-safe interface to lakeFS.

## Features

- **Core lakeFS Operations**: Repository, branch, commit, and object management
- **AWS IAM Authentication**: Secure authentication using AWS credentials
- **Local Sync**: Synchronize local directories with lakeFS repositories
- **High Performance**: Parallel uploads/downloads with progress tracking
- **Type Safety**: Leverages Rust's type system for reliable operations

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/lakefs-rust.git
cd lakefs-rust

# Build the project
cargo build --release

# Install the CLI
cargo install --path crates/lakectl-cli
```

## Configuration

Create a configuration file at `~/.lakectl.yaml`:

```yaml
server:
  endpoint_url: "http://localhost:8000"

credentials:
  type: Basic
  access_key_id: "your-access-key"
  secret_access_key: "your-secret-key"

options:
  parallelism: 10
  no_progress: false
```

For AWS IAM authentication:

```yaml
server:
  endpoint_url: "http://localhost:8000"

credentials:
  type: AwsIam
  region: "us-east-1"
```

## Usage

### Repository Operations

```bash
# Create a repository
lakectl repo create my-repo s3://my-bucket

# List repositories
lakectl repo list

# Delete a repository
lakectl repo delete my-repo
```

### Branch Operations

```bash
# Create a branch
lakectl branch create lakefs://my-repo/feature-branch -s main

# List branches
lakectl branch list lakefs://my-repo

# Delete a branch
lakectl branch delete lakefs://my-repo/feature-branch
```

### File System Operations

```bash
# List objects
lakectl fs ls lakefs://my-repo/main/

# Upload files
lakectl fs upload ./local-file.txt lakefs://my-repo/main/remote-file.txt

# Download files
lakectl fs download lakefs://my-repo/main/remote-file.txt ./local-file.txt

# Remove objects
lakectl fs rm lakefs://my-repo/main/file.txt
```

### Local Sync

```bash
# Clone a repository
lakectl local clone lakefs://my-repo/main ./my-local-repo

# Check status
lakectl local status ./my-local-repo

# Pull changes
lakectl local pull ./my-local-repo

# Commit and push changes
lakectl local commit ./my-local-repo -m "Updated files"
```

## Development

### Project Structure

```
lakefs-rust/
├── crates/
│   ├── lakefs-api/      # Core API client
│   ├── lakefs-auth/     # Authentication providers
│   ├── lakefs-local/    # Local sync functionality
│   └── lakectl-cli/     # CLI implementation
└── Cargo.toml           # Workspace configuration
```

### Running Tests

```bash
# Run all tests
cargo test --all

# Run tests for a specific crate
cargo test -p lakefs-api
```

### Building Documentation

```bash
# Build and open documentation
cargo doc --all --open
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.
