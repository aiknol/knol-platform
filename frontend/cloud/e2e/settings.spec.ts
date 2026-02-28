/**
 * E2E tests for the Settings page (/settings).
 *
 * Uses the shared auth state (logged-in user).
 */
import { test, expect } from './fixtures';

test.describe('Settings page', () => {
  test.beforeEach(async ({ navigateTo }) => {
    await navigateTo('/settings');
  });

  /* ---------------------------------------------------------------- */
  /*  Page structure                                                   */
  /* ---------------------------------------------------------------- */

  test('renders Settings header', async ({ page }) => {
    // The page uses PageHeader component which renders an h2
    await expect(page.locator('main').getByRole('heading', { name: 'Settings' })).toBeVisible();
  });

  test('shows Profile section with full name and email', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Profile' })).toBeVisible();
    await expect(page.getByLabel('Full Name')).toBeVisible();
    // Email is read-only (displayed as text, not input). There are two
    // "cannot be changed" spans (slug + email), so use .first()
    await expect(page.getByText('cannot be changed').first()).toBeVisible();
  });

  test('shows Change Password section', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Change Password' })).toBeVisible();
    // Use element IDs to avoid ambiguity between "New Password" and "Confirm New Password"
    await expect(page.locator('#pw-current')).toBeVisible();
    await expect(page.locator('#pw-new')).toBeVisible();
    await expect(page.locator('#pw-confirm')).toBeVisible();
  });

  test('shows Danger Zone section', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Danger Zone' })).toBeVisible();
    await expect(page.getByRole('button', { name: /delete my account/i })).toBeVisible();
  });

  /* ---------------------------------------------------------------- */
  /*  Workspace section (owner/admin only)                             */
  /* ---------------------------------------------------------------- */

  test('shows Workspace section for owner/admin', async ({ page }) => {
    // The E2E user is the workspace owner (created via signup)
    // Use exact: true to avoid matching the h1 "E2E Test Workspace" in the header
    await expect(page.getByRole('heading', { name: 'Workspace', exact: true })).toBeVisible();
    await expect(page.getByLabel('Workspace Name')).toBeVisible();
    // Slug label is within the workspace section
    const workspaceSection = page.locator('section.card').filter({ hasText: 'Workspace Name' }).first();
    await expect(workspaceSection.getByText('Slug')).toBeVisible();
  });

  /* ---------------------------------------------------------------- */
  /*  Profile update                                                   */
  /* ---------------------------------------------------------------- */

  test('can update profile name', async ({ page }) => {
    const nameInput = page.getByLabel('Full Name');
    const originalName = await nameInput.inputValue();

    // Change name
    await nameInput.clear();
    await nameInput.fill('E2E Updated Name');
    await page.getByRole('button', { name: /save profile/i }).click();

    // Wait for success message
    const alert = page.locator('.alert-success');
    await expect(alert).toBeVisible({ timeout: 10_000 });
    await expect(alert).toContainText('Profile updated');

    // Restore original name
    await nameInput.clear();
    await nameInput.fill(originalName || 'E2E Tester');
    await page.getByRole('button', { name: /save profile/i }).click();
    await expect(alert).toContainText('Profile updated');
  });

  /* ---------------------------------------------------------------- */
  /*  Password change validation                                       */
  /* ---------------------------------------------------------------- */

  test('shows error when new passwords do not match', async ({ page }) => {
    await page.locator('#pw-current').fill('SomeOldPass123!');
    await page.locator('#pw-new').fill('NewPassword123!x');
    await page.locator('#pw-confirm').fill('DifferentPassword123!x');
    await page.getByRole('button', { name: /change password/i }).click();

    const alert = page.locator('.alert-error');
    await expect(alert).toBeVisible({ timeout: 5_000 });
    await expect(alert).toContainText(/do not match/i);
  });

  test('shows error when password is too short', async ({ page }) => {
    await page.locator('#pw-current').fill('SomeOldPass123!');
    // Bypass HTML5 minLength by setting value via JS, then dispatch input event
    await page.locator('#pw-new').evaluate((el: HTMLInputElement) => {
      el.value = 'Short1!aA';
      el.dispatchEvent(new Event('input', { bubbles: true }));
    });
    await page.locator('#pw-confirm').evaluate((el: HTMLInputElement) => {
      el.value = 'Short1!aA';
      el.dispatchEvent(new Event('input', { bubbles: true }));
    });
    // Submit form via JS to bypass HTML5 validation
    await page.locator('form').filter({ has: page.locator('#pw-current') }).evaluate(
      (form: HTMLFormElement) => form.requestSubmit(),
    );

    const alert = page.locator('.alert-error');
    await expect(alert).toBeVisible({ timeout: 5_000 });
    await expect(alert).toContainText(/12 characters/i);
  });

  test('shows error when password missing uppercase', async ({ page }) => {
    await page.locator('#pw-current').fill('SomeOldPass123!');
    await page.locator('#pw-new').fill('alllowercase1234!');
    await page.locator('#pw-confirm').fill('alllowercase1234!');
    await page.getByRole('button', { name: /change password/i }).click();

    const alert = page.locator('.alert-error');
    await expect(alert).toBeVisible({ timeout: 5_000 });
    await expect(alert).toContainText(/uppercase/i);
  });

  test('shows error when password missing special character', async ({ page }) => {
    await page.locator('#pw-current').fill('SomeOldPass123!');
    await page.locator('#pw-new').fill('NoSpecialChar1234A');
    await page.locator('#pw-confirm').fill('NoSpecialChar1234A');
    await page.getByRole('button', { name: /change password/i }).click();

    const alert = page.locator('.alert-error');
    await expect(alert).toBeVisible({ timeout: 5_000 });
    await expect(alert).toContainText(/special character/i);
  });

  test('shows password requirement hint', async ({ page }) => {
    await expect(page.getByText(/Min 12 characters/)).toBeVisible();
  });

  /* ---------------------------------------------------------------- */
  /*  Delete account UI interactions                                   */
  /* ---------------------------------------------------------------- */

  test('delete account: clicking button shows confirmation form', async ({ page }) => {
    const deleteBtn = page.getByRole('button', { name: /delete my account/i });
    await deleteBtn.click();

    // Confirmation form should now be visible
    await expect(page.locator('#delete-password')).toBeVisible();
    await expect(page.getByRole('button', { name: /confirm delete account/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /cancel/i })).toBeVisible();
  });

  test('delete account: cancel hides the confirmation form', async ({ page }) => {
    // Open confirmation
    await page.getByRole('button', { name: /delete my account/i }).click();
    await expect(page.locator('#delete-password')).toBeVisible();

    // Click cancel
    await page.getByRole('button', { name: /cancel/i }).click();

    // Form should be hidden, "Delete my account" button visible again
    await expect(page.locator('#delete-password')).not.toBeVisible();
    await expect(page.getByRole('button', { name: /delete my account/i })).toBeVisible();
  });

  test('delete account: shows warning about 30-day grace period', async ({ page }) => {
    await page.getByRole('button', { name: /delete my account/i }).click();
    await expect(page.getByText(/30-day grace period/i)).toBeVisible();
    await expect(page.getByText(/cannot be undone/i)).toBeVisible();
  });
});
