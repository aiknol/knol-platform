/**
 * E2E tests for the Login page (/login).
 *
 * These tests run WITHOUT the global auth state so they can
 * exercise the unauthenticated login flow.
 */
import { test, expect } from '@playwright/test';

// Override: do NOT use the shared auth state for login tests
test.use({ storageState: { cookies: [], origins: [] } });

test.describe('Login page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
  });

  test('renders the login form', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /sign in/i })).toBeVisible();
    await expect(page.getByLabel('Email')).toBeVisible();
    await expect(page.getByLabel('Password')).toBeVisible();
    await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
  });

  test('shows link to signup page', async ({ page }) => {
    // The link text is "Create a free workspace" inside a paragraph
    const signupLink = page.getByRole('link', { name: /free workspace/i });
    await expect(signupLink).toBeVisible();
    // trailingSlash: true means hrefs end with /
    await expect(signupLink).toHaveAttribute('href', /\/signup/);
  });

  test('shows error on invalid credentials', async ({ page }) => {
    await page.getByLabel('Email').fill('nonexistent@example.com');
    await page.getByLabel('Password').fill('WrongPassword123!');
    await page.getByRole('button', { name: /sign in/i }).click();

    // Error banner should appear
    const errorBanner = page.locator('.text-red-300').first();
    await expect(errorBanner).toBeVisible({ timeout: 10_000 });
  });

  test('shows error on empty form submission', async ({ page }) => {
    // HTML5 validation should prevent submission — check that no navigation occurs
    await page.getByRole('button', { name: /sign in/i }).click();
    await expect(page).toHaveURL(/\/login/);
  });

  test('successful login redirects to playground', async ({ page }) => {
    const email = process.env.E2E_USER_EMAIL || 'e2e-test@knol-e2e.local';
    const password = process.env.E2E_USER_PASSWORD || 'E2eTestPass1234!';

    await page.getByLabel('Email').fill(email);
    await page.getByLabel('Password').fill(password);
    await page.getByRole('button', { name: /sign in/i }).click();

    await page.waitForURL('**/playground/**', { timeout: 15_000 });
    await expect(page).toHaveURL(/\/playground/);
  });

  test('email field enforces type=email validation', async ({ page }) => {
    const emailInput = page.getByLabel('Email');
    await expect(emailInput).toHaveAttribute('type', 'email');
    await expect(emailInput).toHaveAttribute('required', '');
  });

  test('password field is masked', async ({ page }) => {
    const passwordInput = page.getByLabel('Password');
    await expect(passwordInput).toHaveAttribute('type', 'password');
  });
});
