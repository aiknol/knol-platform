import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E configuration for Knol Cloud.
 *
 * Expects the Next.js dev server on http://localhost:3007 and
 * the enterprise tenant API on http://localhost:3002.
 *
 * Run:
 *   npm run test:e2e          – headless, all tests
 *   npm run test:e2e:ui       – interactive Playwright UI
 *   npm run test:e2e:headed   – headed Chromium
 */
export default defineConfig({
  testDir: './e2e',
  outputDir: './e2e/.results',

  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,

  /* Retry to handle flaky auth/network issues with dev servers. */
  retries: process.env.CI ? 2 : 2,

  /* Parallelise on CI, run serially locally (easier to debug). */
  workers: process.env.CI ? 2 : 1,

  /* Reporter */
  reporter: process.env.CI ? 'github' : 'html',

  /* Default timeout per test. */
  timeout: 30_000,

  /* Shared settings for all projects. */
  use: {
    baseURL: process.env.E2E_BASE_URL || 'http://localhost:3007',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    /* --- Unauthenticated tests (login, signup) run first --- */
    {
      name: 'no-auth',
      testMatch: /login\.spec\.ts|signup\.spec\.ts/,
      use: {
        ...devices['Desktop Chrome'],
      },
    },

    /* --- Auth setup (runs after unauthenticated tests) --- */
    {
      name: 'auth-setup',
      testMatch: /auth\.setup\.ts/,
      dependencies: ['no-auth'],
    },

    /* --- Authenticated tests (run after auth setup) --- */
    {
      name: 'chromium',
      testIgnore: /login\.spec\.ts|signup\.spec\.ts|auth\.setup\.ts/,
      use: {
        ...devices['Desktop Chrome'],
        storageState: './e2e/.auth/user.json',
      },
      dependencies: ['auth-setup'],
    },
  ],

  /* Start the Next.js dev server automatically when not already running. */
  webServer: {
    command: 'npm run dev -- -p 3007',
    url: 'http://localhost:3007',
    reuseExistingServer: true,
    timeout: 60_000,
  },
});
