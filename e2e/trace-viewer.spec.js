import { test, expect } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';

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

    // Wait for file input to be attached and set files
    const fileInput = page.locator('input[type="file"]');
    await fileInput.waitFor({ state: 'attached', timeout: 5000 });
    await fileInput.setInputFiles(tracePath);

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

    // Wait for file input and upload trace file
    const fileInput = page.locator('input[type="file"]');
    await fileInput.waitFor({ state: 'attached', timeout: 5000 });
    await fileInput.setInputFiles(tracePath);

    // Wait for actions to load
    await expect(page.locator('.action-item').first()).toBeVisible({ timeout: 10000 });

    // Click the first action
    await page.locator('.action-item').first().click();

    // Wait for action details to appear
    await expect(page.locator('.action-details')).toBeVisible();

    // Verify action is marked as selected
    await expect(page.locator('.action-item.selected')).toBeVisible();
  });
});
