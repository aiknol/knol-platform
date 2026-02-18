import { MetadataRoute } from 'next';

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = 'https://aiknol.com';
  const routes = ['', '/docs/', '/pricing/', '/comparison', '/blog/', '/demo/', '/about/', '/privacy/', '/terms/'];
  const blogSlugs = ['introducing-knol', 'context-engineering', 'hybrid-retrieval', 'llm-cost-optimization', 'memory-intelligence', 'migration-guide'];

  return [
    ...routes.map((route) => ({
      url: `${baseUrl}${route}`,
      lastModified: new Date('2026-02-15'),
      changeFrequency: 'weekly' as const,
      priority: route === '' ? 1 : 0.8,
    })),
    ...blogSlugs.map((slug) => ({
      url: `${baseUrl}/blog/${slug}`,
      lastModified: new Date('2026-02-15'),
      changeFrequency: 'monthly' as const,
      priority: 0.6,
    })),
  ];
}
