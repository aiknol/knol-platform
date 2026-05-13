import type { AppUser, TenantProfile } from './types';

export function getAppAuthToken(): string | null {
  // Auth is now cookie-based (HttpOnly). Return null so apiFetch relies on
  // credentials: 'include' to send the cookie automatically.
  // Clean up any legacy token storage on read.
  if (typeof window !== 'undefined') {
    sessionStorage.removeItem('app_token');
    localStorage.removeItem('app_token');
  }
  return null;
}

export function getAppAuthUser(): AppUser | null {
  if (typeof window === 'undefined') return null;
  const raw = sessionStorage.getItem('app_user') || localStorage.getItem('app_user');
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function getAppTenant(): TenantProfile | null {
  if (typeof window === 'undefined') return null;
  const raw = sessionStorage.getItem('app_tenant') || localStorage.getItem('app_tenant');
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function setAppAuthSession(_token: string, user: AppUser, tenant: TenantProfile, initialApiKey?: string) {
  // Token is now stored in an HttpOnly cookie set by the backend - never in
  // browser-accessible storage. We only keep non-sensitive profile data for
  // UI rendering.
  if (typeof window === 'undefined') return;
  sessionStorage.setItem('app_user', JSON.stringify(user));
  sessionStorage.setItem('app_tenant', JSON.stringify(tenant));

  // Store the initial API key so it can be displayed once on the API Keys page
  // after signup. consumeInitialApiKey() will read and remove it.
  if (initialApiKey) {
    sessionStorage.setItem('app_initial_api_key', initialApiKey);
  }

  // Clean up legacy storage
  sessionStorage.removeItem('app_token');
  localStorage.removeItem('app_token');
  localStorage.removeItem('app_user');
  localStorage.removeItem('app_tenant');
}

export function setAppProfile(user: AppUser, tenant: TenantProfile) {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem('app_user', JSON.stringify(user));
  sessionStorage.setItem('app_tenant', JSON.stringify(tenant));
  localStorage.removeItem('app_user');
  localStorage.removeItem('app_tenant');
}

export function clearAppAuthSession() {
  if (typeof window === 'undefined') return;
  // Clean up all browser storage (token should never be here, but clean legacy)
  sessionStorage.removeItem('app_token');
  sessionStorage.removeItem('app_user');
  sessionStorage.removeItem('app_tenant');
  sessionStorage.removeItem('app_initial_api_key');
  sessionStorage.removeItem('csrf_token');
  localStorage.removeItem('app_token');
  localStorage.removeItem('app_user');
  localStorage.removeItem('app_tenant');
}

export function consumeInitialApiKey(): string | null {
  if (typeof window === 'undefined') return null;
  const key = sessionStorage.getItem('app_initial_api_key');
  if (key) {
    sessionStorage.removeItem('app_initial_api_key');
  }
  return key;
}

/**
 * Read the initial API key without consuming it. Used by the Playground
 * to pre-populate the key field after signup.
 */
export function getInitialApiKey(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem('app_initial_api_key');
}

// ---------------------------------------------------------------------------
// Session key vault – stores full API key values created during this browser
// session so the Playground can use them.  SessionStorage is tab-scoped and
// cleared when the tab closes, which is the appropriate security boundary.
// ---------------------------------------------------------------------------
const SESSION_KEYS_STORAGE = 'app_session_api_keys';

interface SessionKeyEntry {
  id: string;
  name: string;
  role: string;
  api_key: string;
}

export function storeSessionApiKey(entry: SessionKeyEntry): void {
  if (typeof window === 'undefined') return;
  const existing = getSessionApiKeys();
  const filtered = existing.filter((e) => e.id !== entry.id);
  filtered.push(entry);
  sessionStorage.setItem(SESSION_KEYS_STORAGE, JSON.stringify(filtered));
}

export function getSessionApiKeys(): SessionKeyEntry[] {
  if (typeof window === 'undefined') return [];
  const raw = sessionStorage.getItem(SESSION_KEYS_STORAGE);
  if (!raw) return [];
  try {
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

export function getSessionApiKeyValue(id: string): string | null {
  const entries = getSessionApiKeys();
  return entries.find((e) => e.id === id)?.api_key ?? null;
}
