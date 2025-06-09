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
