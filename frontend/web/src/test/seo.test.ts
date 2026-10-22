import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, existsSync } from 'fs';
import path from 'path';
import { withSeo, pageTitle } from '@/config/site';

const APP = path.resolve(__dirname, '../app');

describe('withSeo', () => {
  it('adds a canonical and page-specific Open Graph / Twitter tags', () => {
    const meta = withSeo('/pricing/', { title: pageTitle('Pricing'), description: 'Plans' });
    expect(meta.alternates?.canonical).toBe('/pricing/');
    expect(meta.openGraph).toMatchObject({ title: 'Pricing | DoAide Cortex', description: 'Plans', url: '/pricing/' });
    expect(meta.twitter).toMatchObject({ title: 'Pricing | DoAide Cortex', description: 'Plans' });
  });
});

describe('every route has its own metadata', () => {
  const routes = readdirSync(APP, { withFileTypes: true })
    .filter((d) => d.isDirectory() && existsSync(path.join(APP, d.name, 'page.tsx')))
    .map((d) => d.name);

  it.each(routes)('/%s uses withSeo with its own path', (route) => {
    const src = readFileSync(path.join(APP, route, 'page.tsx'), 'utf-8');
    expect(src).toContain(`withSeo('/${route}/'`);
  });

  it('page titles are unique', () => {
    const titles = routes.map((r) => readFileSync(path.join(APP, r, 'page.tsx'), 'utf-8').match(/title: (pageTitle\([^)]*\))/)?.[1]);
    expect(new Set(titles).size).toBe(titles.length);
  });
});

describe('no fabricated social proof', () => {
  it('has no testimonials component or rating schema', () => {
    expect(existsSync(path.resolve(__dirname, '../components/marketing/Testimonials.tsx'))).toBe(false);
    const layout = readFileSync(path.join(APP, 'layout.tsx'), 'utf-8');
    const home = readFileSync(path.join(APP, 'page.tsx'), 'utf-8');
    for (const src of [layout, home]) {
      expect(src).not.toMatch(/aggregateRating|ratingValue|reviewCount|Testimonials/);
    }
  });
});
