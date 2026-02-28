/**
 * E2E tests for the API Keys page (/api-keys).
 *
 * Uses the shared auth state (logged-in user).
 */
import { test, expect } from './fixtures';

test.describe('API Keys page', () => {
  test.beforeEach(async ({ navigateTo }) => {
    await navigateTo('/api-keys');
  });

  test('renders API Keys header', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'API Keys' })).toBeVisible();
  });

  test('shows Create API Key form', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /create api key/i })).toBeVisible();
    await expect(page.getByLabel('Name')).toBeVisible();
    await expect(page.getByLabel('Role')).toBeVisible();
    await expect(page.getByLabel(/expires/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /create key/i })).toBeVisible();
  });

  test('shows Quick Integration with curl example', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /quick integration/i })).toBeVisible();
    await expect(page.locator('pre')).toContainText('curl');
  });

  test('shows Your Keys section', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /your keys/i })).toBeVisible();
  });

  test('role select has expected options', async ({ page }) => {
    const roleSelect = page.getByLabel('Role');
    await expect(roleSelect.locator('option[value="developer"]')).toHaveText('Developer');
    await expect(roleSelect.locator('option[value="admin"]')).toHaveText('Admin');
    await expect(roleSelect.locator('option[value="read_only"]')).toHaveText('Read only');
  });

  test('can create and see a new API key', async ({ page }) => {
    const uniqueName = `e2e-test-key-${Date.now()}`;

    await page.getByLabel('Name').fill(uniqueName);
    await page.getByLabel('Role').selectOption('developer');
    await page.getByRole('button', { name: /create key/i }).click();

    // Success banner with the newly created key should appear
    const successBanner = page.locator('.alert-success');
    await expect(successBanner).toBeVisible({ timeout: 10_000 });
    await expect(successBanner).toContainText(/shown once/i);

    // The key should now appear in the list
    await expect(page.getByText(uniqueName)).toBeVisible();
  });
});
