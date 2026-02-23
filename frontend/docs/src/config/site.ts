import { ensurePublicEnvIsValid } from './env';

ensurePublicEnvIsValid();

const IS_DEV = process.env.NODE_ENV !== 'production';

function readEnv(name: string): string | null {
  const value = process.env[name]?.trim();
  return value ? value : null;
}

function stripTrailingSlash(value: string): string {
  return value.replace(/\/+$/, '');
}

function ensureTrailingSlash(value: string): string {
  return `${stripTrailingSlash(value)}/`;
}

function defaultUrl(devUrl: string, prodUrl: string): string {
  return IS_DEV ? devUrl : prodUrl;
}

const docsUrl = ensureTrailingSlash(
  readEnv('NEXT_PUBLIC_DOCS_URL') || defaultUrl('http://localhost:3009', 'https://docs.aiknol.com'),
);

const apiBaseUrl = stripTrailingSlash(
  readEnv('NEXT_PUBLIC_API_BASE_URL') || defaultUrl('http://localhost:3000', 'https://api.aiknol.com'),
);

const tenantSwaggerUrl = ensureTrailingSlash(
  readEnv('NEXT_PUBLIC_TENANT_SWAGGER_URL') || `${apiBaseUrl}/docs`,
);

const githubRepoUrl = stripTrailingSlash(
  readEnv('NEXT_PUBLIC_GITHUB_REPO_URL') || 'https://github.com/aiknol/knol',
);

export const DOCS_SITE = {
  name: 'Knol Docs',
  tagline: 'Tenant + OSS Documentation',
  siteUrl: docsUrl,
  apiBaseUrl,
  tenantSwaggerUrl,
  githubRepoUrl,
};

export const DOCS_LINKS = {
  gatewayReadme: `${githubRepoUrl}/blob/main/knol-oss/README.md`,
  mcpApiMapping: `${githubRepoUrl}/blob/main/knol-oss/sdk/mcp/API_MAPPING.md`,
};
