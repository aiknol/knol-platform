import { resolveAdminApiUrl } from '@/config/urls';
import { clearAuthSession, getAuthToken } from './session';

const API_URL = resolveAdminApiUrl();

if (
  typeof window !== 'undefined' &&
  process.env.NODE_ENV !== 'production' &&
  !process.env.NEXT_PUBLIC_ADMIN_API_URL
) {
  console.warn(
    `[Admin API] NEXT_PUBLIC_ADMIN_API_URL is not set. Falling back to ${API_URL}. ` +
    'Set this env var at build time for production deployments.'
  );
}

export type FetchOptions = RequestInit & {
  skipAuth?: boolean;
};

export async function apiFetch<T = any>(endpoint: string, options: FetchOptions = {}): Promise<T> {
  const { skipAuth = false, ...fetchOptions } = options;
  const url = `${API_URL}${endpoint}`;

  const headers = new Headers(fetchOptions.headers || {});
  headers.set('Content-Type', 'application/json');

  if (!skipAuth) {
    const token = getAuthToken();
    if (token) {
      headers.set('Authorization', `Bearer ${token}`);
    }
  }

  const response = await fetch(url, {
    ...fetchOptions,
    headers,
    // SECURITY: Include cookies so the HttpOnly JWT cookie is sent automatically.
    // This works alongside the Bearer token for backward compatibility.
    credentials: 'include',
  });

  if (response.status === 401) {
    if (typeof window !== 'undefined') {
      clearAuthSession();
      window.location.href = '/login';
    }
    throw new Error('Unauthorized');
  }

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`API Error ${response.status}: ${error}`);
  }

  const contentType = response.headers.get('content-type');
  if (contentType?.includes('application/json')) {
    return response.json() as Promise<T>;
  }
  return response.text() as unknown as T;
}
