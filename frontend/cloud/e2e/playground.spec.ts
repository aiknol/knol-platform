/**
 * E2E tests for the Playground page (/playground).
 *
 * Uses the shared auth state (logged-in user).
 */
import { test, expect } from './fixtures';

test.describe('Playground page', () => {
  test.beforeEach(async ({ navigateTo }) => {
    await navigateTo('/playground');
  });

  /* ---------------------------------------------------------------- */
  /*  Page structure                                                   */
  /* ---------------------------------------------------------------- */

  test('renders Playground header', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Playground' })).toBeVisible();
  });

  test('shows API key selector with label', async ({ page }) => {
    // The label text is "Select API Key"
    await expect(page.getByLabel('Select API Key')).toBeVisible();
  });

  test('shows operation selector with label', async ({ page }) => {
    // The label text is "Operation"
    await expect(page.getByLabel('Operation')).toBeVisible();
  });

  test('shows Send Request button', async ({ page }) => {
    await expect(page.getByRole('button', { name: 'Send Request' })).toBeVisible();
  });

  test('shows Request and Response panels', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Request' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Response' })).toBeVisible();
  });

  /* ---------------------------------------------------------------- */
  /*  Operation selection                                              */
  /* ---------------------------------------------------------------- */

  test('operation dropdown contains memory and graph operations', async ({ page }) => {
    const operationSelect = page.getByLabel('Operation');

    // Check that common operations are available in the dropdown
    const options = operationSelect.locator('option');
    const optionTexts = await options.allTextContents();

    // Operations are formatted as "METHOD - Label" e.g. "POST - Search Memories"
    expect(optionTexts.some(t => /Search Memories/i.test(t))).toBeTruthy();
    expect(optionTexts.some(t => /Write Memory/i.test(t) || /Create/i.test(t))).toBeTruthy();
    expect(optionTexts.some(t => /List Entities/i.test(t))).toBeTruthy();
  });

  test('operation dropdown is grouped by category', async ({ page }) => {
    const operationSelect = page.getByLabel('Operation');
    const optgroups = operationSelect.locator('optgroup');
    const groupLabels = await optgroups.evaluateAll(els => els.map(el => el.getAttribute('label')));

    // Should have Memory and Graph groups
    expect(groupLabels).toContain('Memory');
    expect(groupLabels).toContain('Graph');
  });

  /* ---------------------------------------------------------------- */
  /*  Empty response state                                             */
  /* ---------------------------------------------------------------- */

  test('shows empty state message before sending request', async ({ page }) => {
    await expect(page.getByText('Send a request to see the response here.')).toBeVisible();
  });

  /* ---------------------------------------------------------------- */
  /*  API key interaction                                              */
  /* ---------------------------------------------------------------- */

  test('can enter a manual API key', async ({ page }) => {
    // Use the input ID directly to avoid label ambiguity
    const apiKeyInput = page.locator('#pg-api-key');
    await apiKeyInput.fill('knol_test_fake_key_for_e2e');
    await expect(apiKeyInput).toHaveValue('knol_test_fake_key_for_e2e');
  });

  test('API key visibility can be toggled', async ({ page }) => {
    const apiKeyInput = page.locator('#pg-api-key');
    // Initially password type (hidden)
    await expect(apiKeyInput).toHaveAttribute('type', 'password');

    // Click Show button
    await page.getByRole('button', { name: 'Show' }).click();
    await expect(apiKeyInput).toHaveAttribute('type', 'text');

    // Click Hide button
    await page.getByRole('button', { name: 'Hide' }).click();
    await expect(apiKeyInput).toHaveAttribute('type', 'password');
  });

  test('key selector has "Enter manually" option', async ({ page }) => {
    const keySelect = page.locator('#pg-key-select');
    const manualOption = keySelect.locator('option[value="manual"]');
    await expect(manualOption).toHaveText('Enter manually');
  });
});
