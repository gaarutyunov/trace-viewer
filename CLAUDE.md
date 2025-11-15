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

E2E tests use the official JavaScript Playwright library and run automatically in CI.

To run E2E tests locally:
```bash
# Install dependencies
npm install

# Install Playwright browsers
npx playwright install --with-deps chromium

# Build the app
trunk build --release

# Run E2E tests (Playwright will start the server automatically)
npm run test:e2e
```

**Note:** The E2E tests are located in the `e2e/` directory and use JavaScript/Playwright.
The `playwright.config.js` file configures Playwright to automatically start and stop
the development server.

## CI/CD

The CI pipeline automatically runs:
- `cargo fmt --all -- --check` (formatting validation)
- `cargo clippy -- -D warnings` (linting)
- `cargo test --lib` (unit tests)
- `cargo test --test models_tests --test trace_loader_tests` (integration tests)
- E2E tests (JavaScript/Playwright tests in separate job)

All checks must pass before merging.

## Project Structure

- `src/` - Main application code
  - `lib.rs` - Application entry point
  - `models.rs` - Data structures for traces
  - `trace_loader.rs` - ZIP parsing and trace loading
  - `components/` - Yew UI components
- `tests/` - Rust test suite
  - `models_tests.rs` - Unit tests for data models
  - `trace_loader_tests.rs` - Unit tests for trace loading
  - `fixtures/` - Test data files
- `e2e/` - End-to-end tests (JavaScript/Playwright)
  - `trace-viewer.spec.js` - Browser tests
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
