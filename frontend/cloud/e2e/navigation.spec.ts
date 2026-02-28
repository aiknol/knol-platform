/**
 * E2E tests for app navigation and layout.
 *
 * Verifies the tab navigation bar, route guards, and responsive layout.
 */
import { test, expect } from './fixtures';

test.describe('App navigation (authenticated)', () => {
  test('tab bar shows all main navigation items', async ({ page, navigateTo }) => {
    await navigateTo('/dashboard');

    // The nav items are rendered in the <nav> element inside the AppShell
    const nav = page.locator('nav').first();
    await expect(nav.getByRole('link', { name: 'Playground' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Overview' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'API Keys' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Billing' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Settings' })).toBeVisible();
  });

  test('can navigate between all main pages', async ({ page, navigateTo }) => {
    // Dashboard
    await navigateTo('/dashboard');
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible();

    // Settings — click nav link
    const nav = page.locator('nav');
    await nav.getByRole('link', { name: 'Settings' }).click();
    await page.waitForURL('**/settings/**', { timeout: 10_000 });
    await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();

    // API Keys
    await nav.getByRole('link', { name: 'API Keys' }).click();
    await page.waitForURL('**/api-keys/**', { timeout: 10_000 });
    await expect(page.getByRole('heading', { name: 'API Keys' })).toBeVisible();

    // Playground
    await nav.getByRole('link', { name: 'Playground' }).click();
    await page.waitForURL('**/playground/**', { timeout: 10_000 });
    await expect(page.getByRole('heading', { name: 'Playground' })).toBeVisible();
  });

  test('header shows workspace name and logout button', async ({ page, navigateTo }) => {
    await navigateTo('/dashboard');

    await expect(page.locator('header').getByText('Knol Cloud')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Logout' })).toBeVisible();
  });
});

test.describe('Route guards (unauthenticated)', () => {
  test.use({ storageState: { cookies: [], origins: [] } });

  test('unauthenticated user visiting /dashboard is redirected to /login', async ({ page }) => {
    await page.goto('/dashboard');

    // Should redirect to login or show login-like content
    await page.waitForURL('**/login/**', { timeout: 15_000 });
    await expect(page).toHaveURL(/\/login/);
  });

  test('unauthenticated user visiting /settings is redirected to /login', async ({ page }) => {
    await page.goto('/settings');

    await page.waitForURL('**/login/**', { timeout: 15_000 });
    await expect(page).toHaveURL(/\/login/);
  });

  test('/login is accessible without auth', async ({ page }) => {
    await page.goto('/login');
    await expect(page.getByRole('heading', { name: /sign in/i })).toBeVisible();
  });

  test('/signup is accessible without auth', async ({ page }) => {
    await page.goto('/signup');
    await expect(page.getByRole('heading', { name: /create your free workspace/i })).toBeVisible();
  });
});
