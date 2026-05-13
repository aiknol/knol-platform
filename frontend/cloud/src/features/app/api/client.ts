import { resolveAppApiUrl } from '@/config/urls';
import { clearAppAuthSession, getAppAuthToken } from './session';

const API_URL = resolveAppApiUrl();

export type FetchOptions = RequestInit & {
  skipAuth?: boolean;
  skipRedirect?: boolean;
  _isRefreshAttempt?: boolean;
};

const CSRF_SESSION_KEY = 'csrf_token';

function getCsrfToken(): string | null {
  // Prefer sessionStorage (set from response headers — works cross-origin)
  if (typeof sessionStorage !== 'undefined') {
    const stored = sessionStorage.getItem(CSRF_SESSION_KEY);
    if (stored) return stored;
  }
  // Fallback: read from cookie (works same-origin / local dev)
  if (typeof document === 'undefined') return null;
  const match = document.cookie.match(/csrf_token=([^;]+)/);
  return match ? match[1] : null;
}

/** Store CSRF token from a response header (cross-origin support). */
function captureCsrfToken(response: Response): void {
  const token = response.headers.get('x-csrf-token');
  if (token && typeof sessionStorage !== 'undefined') {
    sessionStorage.setItem(CSRF_SESSION_KEY, token);
  }
}

export async function apiFetch<T = any>(endpoint: string, options: FetchOptions = {}): Promise<T> {
  const { skipAuth = false, skipRedirect = false, _isRefreshAttempt = false, ...fetchOptions } = options;
  const url = `${API_URL}${endpoint}`;

  const headers = new Headers(fetchOptions.headers || {});
  headers.set('Content-Type', 'application/json');

  if (!skipAuth) {
    const token = getAppAuthToken();
    if (token) {
      headers.set('Authorization', `Bearer ${token}`);
    }
  }

  // Attach CSRF token on mutating requests
  const method = (fetchOptions.method || 'GET').toUpperCase();
  if (['POST', 'PUT', 'DELETE'].includes(method)) {
    const csrfToken = getCsrfToken();
    if (csrfToken) {
      headers.set('X-CSRF-Token', csrfToken);
    }
  }

  const response = await fetch(url, {
    ...fetchOptions,
    headers,
    credentials: 'include',
  });

  // Capture CSRF token from response header (cross-origin support).
  captureCsrfToken(response);

  if (response.status === 401) {
    // Try auto-refresh once before giving up
    if (!_isRefreshAttempt && !skipRedirect && !skipAuth) {
      try {
        await apiFetch('/app/auth/refresh', { method: 'POST', _isRefreshAttempt: true });
        // Retry original request
        return apiFetch(endpoint, { ...options, _isRefreshAttempt: true });
      } catch {
        // Refresh failed, fall through to redirect
      }
    }
    if (!skipRedirect && typeof window !== 'undefined') {
      clearAppAuthSession();
      window.location.href = '/login';
    }
    throw new Error('Unauthorized');
  }

  if (!response.ok) {
    let message = `API Error ${response.status}`;
    const contentType = response.headers.get('content-type') || '';
    if (contentType.includes('application/json')) {
      const body = await response.json();
      if (body?.error) {
        message = body.error;
      }
    } else {
      const text = await response.text();
      if (text) {
        message = text;
      }
    }
    throw new Error(message);
  }

  const contentType = response.headers.get('content-type') || '';
  if (contentType.includes('application/json')) {
    return response.json() as Promise<T>;
  }
  return response.text() as unknown as T;
}
