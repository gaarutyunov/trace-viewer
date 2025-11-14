# Tests

This directory contains unit tests and E2E tests for the Playwright Trace Viewer.

## Unit Tests

Unit tests can be run with:

```bash
cargo test --lib
```

These tests cover:
- **trace_loader_tests.rs**: ZIP parsing, trace loading, event parsing
- **models_tests.rs**: Data model serialization/deserialization

## End-to-End Tests

E2E tests using playwright-rust are available in `e2e_tests.rs` but are currently disabled by default due to playwright-rust driver installation complexity.

To enable E2E tests:

1. Uncomment the playwright dependencies in `Cargo.toml`:
   ```toml
   [dev-dependencies]
   playwright = "0.0.20"
   tokio = { version = "1", features = ["full"] }
   ```

2. Install playwright browsers:
   ```bash
   npx playwright install chromium
   ```

3. Build and serve the application:
   ```bash
   trunk serve
   ```

4. Run E2E tests:
   ```bash
   cargo test --test e2e_tests -- --ignored
   ```

### E2E Test Coverage

The E2E tests verify:
- Application loads in browser
- File upload UI interactions
- Drag-and-drop functionality
- Trace file parsing and display
- Action selection and details view
- Error handling for invalid files

## Test Fixtures

- `fixtures/sample-trace.zip`: Real Playwright trace file extracted from the test report
- Contains actual trace events, network logs, and resources

## Running Tests in CI

Unit tests run automatically in GitHub Actions CI on every push. E2E tests are optional and can be added to CI when playwright driver support is configured.
