/**
 * E2E tests for the Dashboard page (/dashboard).
 *
 * Uses the shared auth state (logged-in user).
 */
import { test, expect } from './fixtures';

test.describe('Dashboard page', () => {
  test.beforeEach(async ({ navigateTo }) => {
    await navigateTo('/dashboard');
  });

  test('renders Overview header', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible();
  });

  test('shows workspace stats card', async ({ page }) => {
    // "Workspace" appears as a small uppercase label inside a card
    const workspaceLabel = page.locator('article.card').filter({ hasText: 'Workspace' }).first();
    await expect(workspaceLabel).toBeVisible();
    // Slug should be visible inside a code element
    await expect(workspaceLabel.locator('code').first()).toBeVisible();
  });

  test('shows usage stats card', async ({ page }) => {
    await expect(page.getByText('Usage This Month')).toBeVisible();
    await expect(page.getByText('operations')).toBeVisible();
  });

  test('shows account card with email', async ({ page }) => {
    const accountCard = page.locator('article.card').filter({ hasText: 'Account' });
    await expect(accountCard).toBeVisible();
    await expect(page.getByText(/Role:/)).toBeVisible();
  });

  test('shows Quick Actions section with navigation links', async ({ page }) => {
    await expect(page.getByText('Quick Actions')).toBeVisible();

    // Quick action cards are inside a grid, each is a link
    const quickActionsGrid = page.locator('section').filter({ hasText: 'Quick Actions' });
    await expect(quickActionsGrid.getByRole('link', { name: /api keys/i })).toBeVisible();
    await expect(quickActionsGrid.getByRole('link', { name: /billing/i })).toBeVisible();
    await expect(quickActionsGrid.getByRole('link', { name: /settings/i })).toBeVisible();
  });

  test('shows Quick Integration section with curl example', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /quick integration/i })).toBeVisible();
    await expect(page.locator('pre')).toContainText('curl');
  });

  test('API Keys link navigates correctly', async ({ page }) => {
    // Use the Quick Actions link specifically, not the nav link
    const quickActions = page.locator('section').filter({ hasText: 'Quick Actions' });
    await quickActions.getByRole('link', { name: /api keys/i }).click();
    await page.waitForURL('**/api-keys/**', { timeout: 10_000 });
    await expect(page).toHaveURL(/\/api-keys/);
  });

  test('Settings link navigates correctly', async ({ page }) => {
    // Use the main content area's Settings link (inside Quick Actions grid)
    // Account for trailingSlash: true which makes href="/settings/"
    await page.locator('main a[href*="settings"]').first().click();
    await page.waitForURL('**/settings/**', { timeout: 10_000 });
    await expect(page).toHaveURL(/\/settings/);
  });
});
