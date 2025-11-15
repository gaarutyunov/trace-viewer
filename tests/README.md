# Tests

This directory contains unit tests and E2E tests for the Playwright Trace Viewer.

## Unit Tests

Unit tests can be run with:

```bash
cargo test --lib
```

These tests cover:
- **trace_loader_tests.rs**: ZIP parsing, trace loading, event parsing (11 tests)
- **models_tests.rs**: Data model serialization/deserialization (14 tests)

## End-to-End Tests

E2E tests using playwright-rust are now enabled and run automatically in CI.

### Running E2E Tests Locally

1. Install playwright browsers:
   ```bash
   npx playwright install chromium
   ```

2. Build and serve the application:
   ```bash
   trunk serve
   ```

3. In another terminal, run E2E tests:
   ```bash
   cargo test --test e2e_tests -- --ignored
   ```

### E2E Test Coverage

The E2E tests verify:
- Application loads in browser
- File upload UI interactions
- Trace file parsing and display
- Action selection and details view
- Complete user workflow from file drop to viewing trace details

## Test Fixtures

- `fixtures/sample-trace.zip`: Real Playwright trace file extracted from the test report
- Contains actual trace events, network logs, screenshots, and WASM resources
- 177KB authentic test data from real Playwright test execution

## Running Tests in CI

**Automated CI Testing:**
- **Unit Tests**: Run on every push via `cargo test --lib`
- **E2E Tests**: Run on every push via separate E2E job
  - Playwright chromium is installed automatically
  - App is built with Trunk and served on port 8080
  - Tests run with `--ignored` flag
  - All cleanup handled automatically

Both test suites must pass for CI to succeed.
