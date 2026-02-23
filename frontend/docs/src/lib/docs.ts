import fs from 'node:fs';
import path from 'node:path';
import { DOCS_SITE } from '@/config/site';

export type DocKind = 'markdown' | 'html';

export interface DocEntry {
  slug: string;
  title: string;
  summary: string;
  sourcePath: string;
  kind: DocKind;
  section: 'Tenant' | 'OSS';
}

const DOCS: DocEntry[] = [
  {
    slug: 'tenant-service-guide',
    title: 'Tenant Service Guide',
    summary: 'Complete tenant API and workspace guide for cloud tenants.',
    sourcePath: 'docs/tenant-service-guide.md',
    kind: 'markdown',
    section: 'Tenant',
  },
  {
    slug: 'enterprise-readme',
    title: 'Enterprise README',
    summary: 'Enterprise service list and local run details.',
    sourcePath: 'knol-enterprise/README.md',
    kind: 'markdown',
    section: 'Tenant',
  },
  {
    slug: 'oss-readme',
    title: 'OSS README',
    summary: 'Canonical OSS quickstart, architecture, and API references.',
    sourcePath: 'knol-oss/README.md',
    kind: 'markdown',
    section: 'OSS',
  },
  {
    slug: 'oss-sdk-typescript',
    title: 'OSS SDK: TypeScript',
    summary: 'TypeScript SDK setup and usage documentation.',
    sourcePath: 'knol-oss/sdk/typescript/README.md',
    kind: 'markdown',
    section: 'OSS',
  },
  {
    slug: 'oss-sdk-mcp',
    title: 'OSS SDK: MCP',
    summary: 'MCP server integration docs for Knol.',
    sourcePath: 'knol-oss/sdk/mcp/README.md',
    kind: 'markdown',
    section: 'OSS',
  },
  {
    slug: 'oss-sdk-mcp-api-mapping',
    title: 'OSS SDK: MCP API Mapping',
    summary: 'Detailed mapping between MCP tools and REST endpoints.',
    sourcePath: 'knol-oss/sdk/mcp/API_MAPPING.md',
    kind: 'markdown',
    section: 'OSS',
  },
  {
    slug: 'oss-sdk-mcp-test-guide',
    title: 'OSS SDK: MCP Test Guide',
    summary: 'Validation and test strategy for the MCP server.',
    sourcePath: 'knol-oss/sdk/mcp/TEST_GUIDE.md',
    kind: 'markdown',
    section: 'OSS',
  },
  {
    slug: 'oss-security',
    title: 'OSS Security',
    summary: 'Security model, reporting policy, and hardening guidance.',
    sourcePath: 'knol-oss/SECURITY.md',
    kind: 'markdown',
    section: 'OSS',
  },
];

const repoRoot = path.resolve(process.cwd(), '..', '..');

export function getAllDocs(): DocEntry[] {
  return DOCS;
}

export function getDocsBySection(section: DocEntry['section']): DocEntry[] {
  return DOCS.filter((entry) => entry.section === section);
}

export function getDocBySlug(slug: string): DocEntry | undefined {
  return DOCS.find((entry) => entry.slug === slug);
}

export function readDocContent(entry: DocEntry): string {
  const absolutePath = path.join(repoRoot, entry.sourcePath);
  return fs.readFileSync(absolutePath, 'utf8');
}

export function githubSourceUrl(entry: DocEntry): string {
  return `${DOCS_SITE.githubRepoUrl}/blob/main/${entry.sourcePath}`;
}
