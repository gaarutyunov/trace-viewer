# Claude Development Guidelines

## Before Committing

Always run these commands before committing changes:

### 1. Format Code
```bash
cargo fmt --all
```

Ensures consistent code formatting across the project.

### 2. Run Linter
```bash
cargo clippy -- -D warnings
```

Catches common mistakes and enforces Rust best practices. All clippy warnings are treated as errors.

**Note:** We don't use `--all-features` locally because the `e2e-tests` feature requires playwright driver installation. CI runs clippy with all features.

### 3. Run Tests
```bash
# Run unit tests only
cargo test --lib

# Run integration tests (models and trace loader)
cargo test --test models_tests --test trace_loader_tests
```

Verifies that all tests pass before committing.

### 4. Full Pre-Commit Check
```bash
# Run all checks in sequence
cargo fmt --all && \
cargo clippy -- -D warnings && \
cargo test --test models_tests --test trace_loader_tests
```

## E2E Tests

E2E tests require playwright driver installation and may fail locally if not set up. They run automatically in CI.

To run E2E tests locally:
```bash
# Install playwright via npm
npm install -D @playwright/test
npx playwright install --with-deps chromium

# Build and serve the app
trunk build --release
cd dist && python3 -m http.server 8080 &

# Wait for app to start, then run tests with the e2e-tests feature
cargo test --test e2e_tests --features e2e-tests -- --ignored
```

**Note:** E2E tests use the `e2e-tests` feature flag to conditionally compile playwright dependencies. This prevents playwright from being built during normal development.

## CI/CD

The CI pipeline automatically runs:
- `cargo fmt --all -- --check` (formatting validation)
- `cargo clippy --all-features -- -D warnings` (linting with all features, including e2e-tests)
- `cargo test --lib --all-features` (unit tests)
- E2E tests (in separate job with playwright and the `--features e2e-tests` flag)

All checks must pass before merging.

## Project Structure

- `src/` - Main application code
  - `lib.rs` - Application entry point
  - `models.rs` - Data structures for traces
  - `trace_loader.rs` - ZIP parsing and trace loading
  - `components/` - Yew UI components
- `tests/` - Test suite
  - `models_tests.rs` - Unit tests for data models
  - `trace_loader_tests.rs` - Unit tests for trace loading
  - `e2e_tests.rs` - End-to-end browser tests
  - `fixtures/` - Test data files
- `.github/workflows/` - CI/CD configuration

## Common Tasks

### Building for Production
```bash
trunk build --release
```

### Development Server
```bash
trunk serve
```

### Running Specific Tests
```bash
# Run only model tests
cargo test --test models_tests

# Run only trace loader tests
cargo test --test trace_loader_tests

# Run a specific test
cargo test test_load_trace_from_zip_success
```

### Checking Without Building
```bash
cargo check --all-features
```
