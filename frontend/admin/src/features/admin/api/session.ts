export function getAuthToken(): string | null {
  // SECURITY: The JWT is now sent only via HttpOnly cookies (set by the backend).
  // We no longer read tokens from sessionStorage/localStorage to prevent XSS exfiltration.
  // This function returns null - the cookie is sent automatically via credentials: 'include'.
  return null;
}

export function getAuthUser(): { email: string; role: string } | null {
  if (typeof window === 'undefined') return null;
  const raw = sessionStorage.getItem('admin_user') || localStorage.getItem('admin_user');
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function setAuthSession(_token: string, admin: unknown) {
  if (typeof window === 'undefined') return;
  // SECURITY: Do NOT store the JWT in sessionStorage/localStorage.
  // The backend sets an HttpOnly cookie, which is safe from XSS.
  // We only store the user profile (non-sensitive) for UI display.
  sessionStorage.setItem('admin_user', JSON.stringify(admin));
  // Clear any legacy token storage.
  sessionStorage.removeItem('admin_token');
  localStorage.removeItem('admin_token');
  localStorage.removeItem('admin_user');
}

export function setAuthUser(admin: unknown) {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem('admin_user', JSON.stringify(admin));
  localStorage.removeItem('admin_user');
}

export function clearAuthSession() {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem('admin_user');
  // Clear legacy token storage
  sessionStorage.removeItem('admin_token');
  localStorage.removeItem('admin_token');
  localStorage.removeItem('admin_user');
}
