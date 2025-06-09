#!/bin/bash

set -e

echo "Installing tarpaulin if not installed..."
if ! command -v cargo-tarpaulin &> /dev/null; then
    cargo install cargo-tarpaulin
fi

echo "Running tests with coverage..."
cargo tarpaulin \
    --all-features \
    --workspace \
    --out Html \
    --out Lcov \
    --exclude-files '*/tests/*' \
    --exclude-files '*/target/*' \
    --ignore-panics \
    --ignore-tests

echo "Coverage report generated:"
echo "  - HTML: target/tarpaulin/tarpaulin-report.html"
echo "  - LCOV: target/lcov.info"

# Print summary
echo ""
echo "Coverage Summary:"
cargo tarpaulin --print-summary
