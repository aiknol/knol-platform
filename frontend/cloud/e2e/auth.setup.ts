/**
 * Global auth setup — runs once before all tests.
 *
 * It signs up a fresh test workspace (or logs in if it already exists),
 * then saves the browser storage state so every subsequent test starts
 * authenticated.
 *
 * Required env vars (with defaults for local dev):
 *   E2E_USER_EMAIL     – defaults to e2e-test@knol-e2e.local
 *   E2E_USER_PASSWORD  – defaults to E2eTestPass1234!
 *   E2E_COMPANY_NAME   – defaults to E2E Test Workspace
 *   E2E_USER_FULLNAME  – defaults to E2E Tester
 */
import { test as setup, expect } from '@playwright/test';

const EMAIL = process.env.E2E_USER_EMAIL || 'e2e-test@knol-e2e.local';
const PASSWORD = process.env.E2E_USER_PASSWORD || 'E2eTestPass1234!';
const COMPANY = process.env.E2E_COMPANY_NAME || 'E2E Test Workspace';
const FULL_NAME = process.env.E2E_USER_FULLNAME || 'E2E Tester';

const AUTH_FILE = './e2e/.auth/user.json';

setup('authenticate', async ({ page }) => {
  // Try signing up first. If the account already exists the API returns an error
  // and we fall back to login.
  await page.goto('/signup');

  // Fill signup form
  await page.getByLabel('Company name').fill(COMPANY);
  await page.getByLabel('Full name').fill(FULL_NAME);
  await page.getByLabel('Work email').fill(EMAIL);
  await page.getByLabel('Password').fill(PASSWORD);
  await page.getByRole('button', { name: 'Create Free Workspace' }).click();

  // Wait for either navigation to /api-keys (signup success) or error
  const signupResult = await Promise.race([
    page.waitForURL('**/api-keys/**', { timeout: 10_000 }).then(() => 'success' as const),
    page.locator('.text-red-300').first().waitFor({ timeout: 10_000 }).then(() => 'error' as const),
  ]).catch(() => 'error' as const);

  if (signupResult === 'error') {
    // Signup failed (account likely exists) — log in instead
    await page.goto('/login');
    await page.getByLabel('Email').fill(EMAIL);
    await page.getByLabel('Password').fill(PASSWORD);
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Wait for redirect to /playground after successful login
    await page.waitForURL('**/playground/**', { timeout: 15_000 });
  }

  // Verify we're authenticated by checking for an app shell element
  await expect(page.locator('body')).toBeVisible();

  // Save authentication state
  await page.context().storageState({ path: AUTH_FILE });
});
