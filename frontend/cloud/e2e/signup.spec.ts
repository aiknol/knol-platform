/**
 * E2E tests for the Signup page (/signup).
 *
 * Runs without auth state to test the registration flow.
 */
import { test, expect } from '@playwright/test';

test.use({ storageState: { cookies: [], origins: [] } });

test.describe('Signup page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/signup');
  });

  test('renders the signup form with all fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /create your free workspace/i })).toBeVisible();
    await expect(page.getByLabel('Company name')).toBeVisible();
    await expect(page.getByLabel('Full name')).toBeVisible();
    await expect(page.getByLabel('Work email')).toBeVisible();
    await expect(page.getByLabel('Password')).toBeVisible();
    await expect(page.getByRole('button', { name: /create free workspace/i })).toBeVisible();
  });

  test('shows link to login page', async ({ page }) => {
    const loginLink = page.getByRole('link', { name: /sign in/i });
    await expect(loginLink).toBeVisible();
    // trailingSlash: true means hrefs end with /
    await expect(loginLink).toHaveAttribute('href', /\/login/);
  });

  test('shows password validation hint', async ({ page }) => {
    await expect(page.getByText(/must include uppercase/i)).toBeVisible();
  });

  test('client-side validation rejects weak password', async ({ page }) => {
    await page.getByLabel('Company name').fill('Test Corp');
    await page.getByLabel('Full name').fill('Test User');
    await page.getByLabel('Work email').fill('weak-pw-test@example.com');
    // Set password value via JS to bypass HTML5 minLength validation
    await page.getByLabel('Password').evaluate((el: HTMLInputElement) => {
      el.value = 'short';
      el.dispatchEvent(new Event('input', { bubbles: true }));
    });
    // Submit via JS to bypass native form validation
    await page.locator('form').evaluate((form: HTMLFormElement) => form.requestSubmit());

    // Should show error about password requirements
    const errorBanner = page.locator('.text-red-300').first();
    await expect(errorBanner).toBeVisible({ timeout: 5_000 });
    await expect(errorBanner).toContainText(/password/i);
  });

  test('shows error for duplicate email', async ({ page }) => {
    const email = process.env.E2E_USER_EMAIL || 'e2e-test@knol-e2e.local';
    const password = process.env.E2E_USER_PASSWORD || 'E2eTestPass1234!';

    await page.getByLabel('Company name').fill('Duplicate Test Corp');
    await page.getByLabel('Full name').fill('Duplicate Tester');
    await page.getByLabel('Work email').fill(email);
    await page.getByLabel('Password').fill(password);
    await page.getByRole('button', { name: /create free workspace/i }).click();

    // Should show error (email already taken)
    const errorBanner = page.locator('.text-red-300').first();
    await expect(errorBanner).toBeVisible({ timeout: 10_000 });
  });

  test('password field has minLength constraint', async ({ page }) => {
    const passwordInput = page.getByLabel('Password');
    await expect(passwordInput).toHaveAttribute('type', 'password');
    await expect(passwordInput).toHaveAttribute('minlength', '12');
  });
});
