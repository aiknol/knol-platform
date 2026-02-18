import { Metadata } from 'next';
import Link from 'next/link';
import { BLOG_POSTS } from '@/config';
import { pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Blog'),
  description: 'Updates, technical deep dives, and research from the Knol team on AI memory infrastructure.',
};

export default function BlogPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold text-dark-50 mb-4">Blog</h1>
        <p className="text-dark-300 text-lg mb-12">
          Updates, technical deep dives, and research from the Knol team.
        </p>

        <div className="space-y-8">
          {BLOG_POSTS.map((post) => (
            <article key={post.title} className="card group cursor-pointer">
              <div className="flex items-center gap-3 mb-3">
                <span className="text-xs px-2 py-0.5 rounded-full bg-brand-500/10 text-brand-400 border border-brand-500/20">
                  {post.tag}
                </span>
                <time className="text-xs text-dark-400">{post.date}</time>
              </div>
              <h2 className="text-xl font-semibold text-dark-100 group-hover:text-brand-400 transition-colors mb-2">
                <Link href={post.href}>{post.title}</Link>
              </h2>
              <p className="text-dark-300 text-sm">{post.description}</p>
            </article>
          ))}
        </div>
      </div>
    </div>
  );
}
