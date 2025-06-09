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
