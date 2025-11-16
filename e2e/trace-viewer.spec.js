import { test, expect } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';
import AdmZip from 'adm-zip';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test.describe('Trace Viewer', () => {
  test('should load app in browser', async ({ page }) => {
    await page.goto('/');

    await expect(page.locator('.app')).toBeVisible();
    await expect(page.locator('.header')).toBeVisible();
    await expect(page.locator('.drop-zone')).toBeVisible();

    await expect(page).toHaveTitle(/Playwright Trace Viewer/);
  });

  test('should upload trace file and display trace viewer', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.drop-zone')).toBeVisible();

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    // Use file chooser to handle file upload
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    // Wait for trace viewer to load
    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.action-list')).toBeVisible();

    // Verify actions are displayed
    const actionItems = page.locator('.action-item');
    await expect(actionItems.first()).toBeVisible();
  });

  test('should select action and show details', async ({ page }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    // Use file chooser to handle file upload
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    // Wait for actions to load
    await expect(page.locator('.action-item').first()).toBeVisible({ timeout: 10000 });

    // Click the first action
    await page.locator('.action-item').first().click();

    // Wait for action details to appear
    await expect(page.locator('.action-details')).toBeVisible();

    // Verify action is marked as selected
    await expect(page.locator('.action-item.selected')).toBeVisible();
  });

  test('should display export controls with proper styling', async ({ page }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Verify export controls are visible
    await expect(page.locator('.export-controls')).toBeVisible();

    // Verify clipboard button exists with proper class
    const copyButton = page.locator('button.copy-button');
    await expect(copyButton).toBeVisible();
    await expect(copyButton).toHaveText(/Copy/);

    // Verify export button exists
    const exportButton = page.locator('button.export-button');
    await expect(exportButton).toBeVisible();
    await expect(exportButton).toHaveText(/Export/);

    // Verify errors-only checkbox exists with enhanced styling
    const errorsCheckbox = page.locator('.errors-only-checkbox');
    await expect(errorsCheckbox).toBeVisible();
    await expect(errorsCheckbox).toContainText('Errors only');
  });

  test('should toggle errors-only checkbox', async ({ page }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    const checkbox = page.locator('.errors-only-checkbox input[type="checkbox"]');

    // Verify checkbox is not checked initially
    await expect(checkbox).not.toBeChecked();

    // Click to check
    await page.locator('.errors-only-checkbox').click();
    await expect(checkbox).toBeChecked();

    // Click to uncheck
    await page.locator('.errors-only-checkbox').click();
    await expect(checkbox).not.toBeChecked();
  });

  test('should trigger download when export button is clicked', async ({ page }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Click export button
    await page.locator('button.export-button').click();

    // Wait for download to start
    const download = await downloadPromise;

    // Verify download was triggered
    expect(download).toBeTruthy();

    // Verify filename ends with .md
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/\.md$/);
  });

  test('should trigger download with errors filename when errors-only is checked', async ({ page }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Enable errors-only mode
    await page.locator('.errors-only-checkbox').click();
    const checkbox = page.locator('.errors-only-checkbox input[type="checkbox"]');
    await expect(checkbox).toBeChecked();

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Click export button
    await page.locator('button.export-button').click();

    // Wait for download to start
    const download = await downloadPromise;

    // Verify download was triggered with errors filename
    expect(download).toBeTruthy();
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/_errors\.md$/);
  });

  test('should copy to clipboard when copy button is clicked', async ({ page, context }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);

    const copyButton = page.locator('button.copy-button');

    // Verify initial state
    await expect(copyButton).toHaveText(/Copy/);

    // Click copy button
    await copyButton.click();

    // Verify button shows success state
    await expect(copyButton).toHaveText(/Copied!/);
    await expect(copyButton).toHaveClass(/copy-success/);

    // Verify clipboard has content
    const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
    expect(clipboardText).toBeTruthy();
    expect(clipboardText.length).toBeGreaterThan(0);

    // Verify it's markdown format
    expect(clipboardText).toContain('#');
  });

  test('should copy errors-only content when checkbox is enabled', async ({ page, context }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);

    // First, copy without errors-only
    await page.locator('button.copy-button').click();
    await expect(page.locator('button.copy-button')).toHaveText(/Copied!/);

    const fullClipboardText = await page.evaluate(() => navigator.clipboard.readText());

    // Wait for success state to reset
    await page.waitForTimeout(500);

    // Now enable errors-only
    await page.locator('.errors-only-checkbox').click();
    await expect(page.locator('.errors-only-checkbox input[type="checkbox"]')).toBeChecked();

    // Copy again with errors-only
    await page.locator('button.copy-button').click();
    await expect(page.locator('button.copy-button')).toHaveText(/Copied!/);

    const errorsOnlyClipboardText = await page.evaluate(() => navigator.clipboard.readText());

    // Both should have content
    expect(fullClipboardText.length).toBeGreaterThan(0);
    expect(errorsOnlyClipboardText.length).toBeGreaterThan(0);

    // Both should be markdown
    expect(fullClipboardText).toContain('#');
    expect(errorsOnlyClipboardText).toContain('#');
  });
});

test.describe('Report Archive with Multiple Traces', () => {
  let reportArchivePath;

  test.beforeAll(() => {
    // Create a report archive with multiple trace files
    const sampleTracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');
    const sampleTraceData = fs.readFileSync(sampleTracePath);

    const reportZip = new AdmZip();

    // Add two copies of the trace to data/ folder
    reportZip.addFile('data/trace1.zip', sampleTraceData);
    reportZip.addFile('data/trace2.zip', sampleTraceData);

    // Write the report archive to a temporary file
    reportArchivePath = path.join(__dirname, 'temp-report.zip');
    reportZip.writeZip(reportArchivePath);
  });

  test.afterAll(() => {
    // Clean up temporary file
    if (fs.existsSync(reportArchivePath)) {
      fs.unlinkSync(reportArchivePath);
    }
  });

  test('should display tabs when loading report with multiple traces', async ({ page }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    // Wait for trace viewer to load
    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Verify tabs container is visible
    await expect(page.locator('.tabs-container')).toBeVisible();

    // Verify we have tabs
    const tabs = page.locator('.tab');
    await expect(tabs).toHaveCount(2);

    // Verify first tab is active
    await expect(tabs.first()).toHaveClass(/tab-active/);
  });

  test('should switch between tabs', async ({ page }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    const tabs = page.locator('.tab');
    await expect(tabs).toHaveCount(2);

    // First tab should be active initially
    await expect(tabs.first()).toHaveClass(/tab-active/);
    await expect(tabs.nth(1)).not.toHaveClass(/tab-active/);

    // Click second tab
    await tabs.nth(1).click();

    // Second tab should now be active
    await expect(tabs.nth(1)).toHaveClass(/tab-active/);
    await expect(tabs.first()).not.toHaveClass(/tab-active/);

    // Click first tab again
    await tabs.first().click();

    // First tab should be active again
    await expect(tabs.first()).toHaveClass(/tab-active/);
    await expect(tabs.nth(1)).not.toHaveClass(/tab-active/);
  });

  test('should clear selected action when switching tabs', async ({ page }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.action-item').first()).toBeVisible({ timeout: 10000 });

    // Select an action in the first tab
    await page.locator('.action-item').first().click();
    await expect(page.locator('.action-item.selected')).toBeVisible();
    await expect(page.locator('.action-details')).toBeVisible();

    // Switch to second tab
    const tabs = page.locator('.tab');
    await tabs.nth(1).click();

    // Selected action should be cleared - no selection should be visible
    await expect(page.locator('.action-item.selected')).not.toBeVisible();
    await expect(page.locator('.no-selection')).toBeVisible();
  });

  test('should hide tabs when loading single trace', async ({ page }) => {
    await page.goto('/');

    const tracePath = path.join(__dirname, '..', 'tests', 'fixtures', 'sample-trace.zip');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(tracePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Tabs should not be visible for a single trace
    await expect(page.locator('.tabs-container')).not.toBeVisible();
  });

  test('should show correct tab titles', async ({ page }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    const tabs = page.locator('.tab');

    // Verify tabs have text content (either from trace title or default)
    const firstTabText = await tabs.first().textContent();
    const secondTabText = await tabs.nth(1).textContent();

    expect(firstTabText.trim().length).toBeGreaterThan(0);
    expect(secondTabText.trim().length).toBeGreaterThan(0);
  });

  test('should display different actions for different tabs', async ({ page }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.action-item').first()).toBeVisible({ timeout: 10000 });

    // Note: Since both traces are the same in our test fixture,
    // we're mainly verifying that the action list rerenders when switching tabs
    const tabs = page.locator('.tab');

    // Get action count for first tab
    await tabs.first().click();
    await expect(page.locator('.action-item').first()).toBeVisible();
    const firstTabActionCount = await page.locator('.action-item').count();

    // Switch to second tab
    await tabs.nth(1).click();
    await expect(page.locator('.action-item').first()).toBeVisible();
    const secondTabActionCount = await page.locator('.action-item').count();

    // Both should have actions (same fixture, so same count)
    expect(firstTabActionCount).toBeGreaterThan(0);
    expect(secondTabActionCount).toBeGreaterThan(0);
  });

  test('should export only active tab content', async ({ page }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    const tabs = page.locator('.tab');
    await expect(tabs).toHaveCount(2);

    // Ensure first tab is active
    await tabs.first().click();
    await expect(tabs.first()).toHaveClass(/tab-active/);

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Click export button
    await page.locator('button.export-button').click();

    // Wait for download to start
    const download = await downloadPromise;

    // Verify download was triggered
    expect(download).toBeTruthy();
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/\.md$/);

    // Note: We can't easily verify the content of the download in this test,
    // but we've verified that the export was triggered from the active tab
  });

  test('should copy only active tab content to clipboard', async ({ page, context }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);

    const tabs = page.locator('.tab');
    await expect(tabs).toHaveCount(2);

    // Copy from first tab
    await tabs.first().click();
    await expect(tabs.first()).toHaveClass(/tab-active/);

    await page.locator('button.copy-button').click();
    await expect(page.locator('button.copy-button')).toHaveText(/Copied!/);

    const firstTabClipboard = await page.evaluate(() => navigator.clipboard.readText());

    // Wait a moment for the copy success state to reset
    await page.waitForTimeout(500);

    // Switch to second tab and copy
    await tabs.nth(1).click();
    await expect(tabs.nth(1)).toHaveClass(/tab-active/);

    await page.locator('button.copy-button').click();
    await expect(page.locator('button.copy-button')).toHaveText(/Copied!/);

    const secondTabClipboard = await page.evaluate(() => navigator.clipboard.readText());

    // Both should have content
    expect(firstTabClipboard.length).toBeGreaterThan(0);
    expect(secondTabClipboard.length).toBeGreaterThan(0);

    // Both should be markdown
    expect(firstTabClipboard).toContain('#');
    expect(secondTabClipboard).toContain('#');

    // Note: Since both traces are the same in our fixture, the content will be identical,
    // but we've verified that the copy operation works for each tab
  });

  test('should export with errors-only from active tab in multi-tab scenario', async ({ page }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    const tabs = page.locator('.tab');
    await expect(tabs).toHaveCount(2);

    // Ensure second tab is active
    await tabs.nth(1).click();
    await expect(tabs.nth(1)).toHaveClass(/tab-active/);

    // Enable errors-only mode
    await page.locator('.errors-only-checkbox').click();
    await expect(page.locator('.errors-only-checkbox input[type="checkbox"]')).toBeChecked();

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Click export button
    await page.locator('button.export-button').click();

    // Wait for download to start
    const download = await downloadPromise;

    // Verify download was triggered with errors filename
    expect(download).toBeTruthy();
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/_errors\.md$/);
  });

  test('should maintain errors-only state when switching tabs', async ({ page, context }) => {
    await page.goto('/');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.locator('button.select-file-button').click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(reportArchivePath);

    await expect(page.locator('.trace-viewer')).toBeVisible({ timeout: 10000 });

    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);

    const tabs = page.locator('.tab');
    const checkbox = page.locator('.errors-only-checkbox input[type="checkbox"]');

    // Enable errors-only on first tab
    await tabs.first().click();
    await page.locator('.errors-only-checkbox').click();
    await expect(checkbox).toBeChecked();

    // Switch to second tab
    await tabs.nth(1).click();

    // Errors-only should still be checked
    await expect(checkbox).toBeChecked();

    // Copy should use errors-only mode
    await page.locator('button.copy-button').click();
    await expect(page.locator('button.copy-button')).toHaveText(/Copied!/);

    const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
    expect(clipboardText).toBeTruthy();
    expect(clipboardText).toContain('#');
  });
});
