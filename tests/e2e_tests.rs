// E2E tests using playwright-rust
// These tests run in CI with Playwright installed
// They require the application to be running on http://localhost:8080
// Run with: cargo test --test e2e_tests --features e2e-tests -- --ignored

#![cfg(feature = "e2e-tests")]

use ::playwright::api::*;
use ::playwright::imp::utils::File;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
#[ignore]
async fn test_app_loads_in_browser() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::initialize().await?;
    playwright.install_chromium()?;

    let chromium = playwright.chromium();
    let browser = chromium.launcher().headless(true).launch().await?;
    let context = browser.context_builder().build().await?;
    let page = context.new_page().await?;

    page.goto_builder("http://localhost:8080").goto().await?;
    page.wait_for_selector_builder(".app")
        .wait_for_selector()
        .await?;

    let header = page.query_selector(".header").await?;
    assert!(header.is_some(), "Header not found");

    let drop_zone = page.query_selector(".drop-zone").await?;
    assert!(drop_zone.is_some(), "Drop zone not found");

    let title = page.title().await?;
    assert!(title.contains("Playwright Trace Viewer"));

    browser.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_trace_file_upload_and_display() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::initialize().await?;
    playwright.install_chromium()?;

    let chromium = playwright.chromium();
    let browser = chromium.launcher().headless(true).launch().await?;
    let context = browser.context_builder().build().await?;
    let page = context.new_page().await?;

    page.goto_builder("http://localhost:8080").goto().await?;
    page.wait_for_selector_builder(".drop-zone")
        .wait_for_selector()
        .await?;

    let mut trace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    trace_path.push("tests/fixtures/sample-trace.zip");

    page.click_builder("button.select-file-button")
        .click()
        .await?;
    let file_input = page
        .wait_for_selector_builder("input[type='file']")
        .wait_for_selector()
        .await?
        .expect("File input not found");
    let file_contents = fs::read(&trace_path)?;
    let file = File::new(
        trace_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        "application/zip".to_string(),
        &file_contents,
    );
    file_input
        .set_input_files_builder(file)
        .set_input_files()
        .await?;

    page.wait_for_selector_builder(".trace-viewer")
        .wait_for_selector()
        .await?;

    let viewer = page.query_selector(".trace-viewer").await?;
    assert!(viewer.is_some(), "Trace viewer not displayed");

    let action_list = page.query_selector(".action-list").await?;
    assert!(action_list.is_some(), "Action list not found");

    let action_items = page.query_selector_all(".action-item").await?;
    assert!(!action_items.is_empty(), "No actions displayed");

    browser.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_action_selection() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::initialize().await?;
    playwright.install_chromium()?;

    let chromium = playwright.chromium();
    let browser = chromium.launcher().headless(true).launch().await?;
    let context = browser.context_builder().build().await?;
    let page = context.new_page().await?;

    page.goto_builder("http://localhost:8080").goto().await?;
    page.wait_for_selector_builder(".drop-zone")
        .wait_for_selector()
        .await?;

    // Upload trace file
    let mut trace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    trace_path.push("tests/fixtures/sample-trace.zip");

    page.click_builder("button.select-file-button")
        .click()
        .await?;
    let file_input = page
        .wait_for_selector_builder("input[type='file']")
        .wait_for_selector()
        .await?
        .expect("File input not found");
    let file_contents = fs::read(&trace_path)?;
    let file = File::new(
        trace_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        "application/zip".to_string(),
        &file_contents,
    );
    file_input
        .set_input_files_builder(file)
        .set_input_files()
        .await?;

    // Wait for actions to load
    let first_action = page
        .wait_for_selector_builder(".action-item")
        .wait_for_selector()
        .await?
        .expect("First action not found");

    // Click the first action
    first_action.click_builder().click().await?;

    // Wait for action details to appear
    page.wait_for_selector_builder(".action-details")
        .wait_for_selector()
        .await?;

    // Verify details are shown
    let details = page.query_selector(".action-details").await?;
    assert!(details.is_some(), "Action details not shown");

    // Check that the action is marked as selected
    let selected_action = page.query_selector(".action-item.selected").await?;
    assert!(selected_action.is_some(), "Action not marked as selected");

    browser.close().await?;
    Ok(())
}
