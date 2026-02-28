/**
 * Shared test fixtures and helpers for Knol Cloud E2E tests.
 *
 * Re-export a `test` object that is pre-configured with:
 *  - A logged-in page (via storageState from auth.setup.ts)
 *  - Handy page-object helpers
 */
import { test as base, expect, type Page } from '@playwright/test';

/* ------------------------------------------------------------------ */
/*  Page helpers                                                       */
/* ------------------------------------------------------------------ */

/** Navigation helper — waits for Next.js client nav to settle. */
export async function navigateTo(page: Page, path: string) {
  await page.goto(path);
  // Wait for Next.js hydration / loading states to finish
  await page.waitForLoadState('networkidle');

  // If we were redirected to /login, the auth cookie may not have
  // been attached yet (race condition). Retry once after a short wait.
  if (page.url().includes('/login') && !path.includes('/login')) {
    await page.waitForTimeout(1_000);
    await page.goto(path);
    await page.waitForLoadState('networkidle');
  }
}

/** Wait for loading spinners / skeleton text to disappear. */
export async function waitForLoaded(page: Page) {
  // Common loading indicators in Knol Cloud pages
  const loadingTexts = [
    'Loading settings...',
    'Loading dashboard...',
    'Loading API keys...',
    'Loading team...',
    'Loading billing...',
    'Loading playground...',
    'Loading workspace...',
  ];
  for (const text of loadingTexts) {
    const locator = page.getByText(text);
    if (await locator.isVisible({ timeout: 500 }).catch(() => false)) {
      await locator.waitFor({ state: 'hidden', timeout: 15_000 });
    }
  }
}

/** Assert an alert-success message appears. */
export async function expectSuccess(page: Page, text?: string) {
  const alert = page.locator('.alert-success');
  await expect(alert).toBeVisible({ timeout: 10_000 });
  if (text) {
    await expect(alert).toContainText(text);
  }
}

/** Assert an alert-error message appears. */
export async function expectError(page: Page, text?: string) {
  const alert = page.locator('.alert-error');
  await expect(alert).toBeVisible({ timeout: 10_000 });
  if (text) {
    await expect(alert).toContainText(text);
  }
}

/* ------------------------------------------------------------------ */
/*  Extended test fixture                                              */
/* ------------------------------------------------------------------ */

type KnolFixtures = {
  /** Navigate to a route and wait for load. */
  navigateTo: (path: string) => Promise<void>;
};

export const test = base.extend<KnolFixtures>({
  navigateTo: async ({ page }, provide) => {
    await provide(async (path: string) => {
      await navigateTo(page, path);
      await waitForLoaded(page);
    });
  },
});

export { expect };
