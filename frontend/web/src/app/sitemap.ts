import { MetadataRoute } from 'next';
import { SITE } from '@/config/site';
import { BLOG_POSTS } from '@/config/marketing';

export const dynamic = 'force-static';

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = SITE.url.replace(/\/+$/, '');
  const routes = ['', '/docs/', '/pricing/', '/comparison', '/blog/', '/demo/', '/about/', '/privacy/', '/terms/', '/mcp/', '/cortex-local/', '/cortex-oss/', '/cortex-platform/'];

  return [
    ...routes.map((route) => ({
      url: `${baseUrl}${route}`,
      lastModified: new Date('2026-10-08'),
      changeFrequency: 'weekly' as const,
      priority: route === '' ? 1 : 0.8,
    })),
    ...BLOG_POSTS.map((post) => ({
      url: `${baseUrl}/blog/${post.slug}`,
      lastModified: new Date('2026-10-08'),
      changeFrequency: 'monthly' as const,
      priority: 0.6,
    })),
  ];
}
