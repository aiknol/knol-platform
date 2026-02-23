const HTTP_URL_RE = /^https?:\/\//i;

function readEnv(name: string): string | null {
  const value = process.env[name]?.trim();
  return value ? value : null;
}

function assertHttpUrl(name: string): void {
  const value = readEnv(name);
  if (!value) return;
  if (!HTTP_URL_RE.test(value)) {
    throw new Error(`[env] ${name} must start with http:// or https://. Received: "${value}"`);
  }
}

let validated = false;

export function ensurePublicEnvIsValid(): void {
  if (validated) return;

  assertHttpUrl('NEXT_PUBLIC_DOCS_URL');
  assertHttpUrl('NEXT_PUBLIC_API_BASE_URL');
  assertHttpUrl('NEXT_PUBLIC_TENANT_SWAGGER_URL');
  assertHttpUrl('NEXT_PUBLIC_GITHUB_REPO_URL');

  validated = true;
}
