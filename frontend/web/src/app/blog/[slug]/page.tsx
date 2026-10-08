import Link from 'next/link';
import { Metadata } from 'next';
import { BLOG_POSTS, BlogPost } from '@/config';
import { pageTitle, SITE } from '@/config/site';

type PageProps = {
  params: Promise<{ slug: string }>;
};

export async function generateStaticParams() {
  return BLOG_POSTS.map((post) => ({
    slug: post.slug,
  }));
}

export async function generateMetadata(props: PageProps): Promise<Metadata> {
  const params = await props.params;
  const post = BLOG_POSTS.find((p) => p.slug === params.slug);

  if (!post) {
    return {
      title: pageTitle('Post Not Found'),
      description: 'The blog post you are looking for does not exist.',
    };
  }

  return {
    title: pageTitle(post.title),
    description: post.description,
  };
}

function renderBody(body: string) {
  const parts = body.split('\n\n');

  return parts.map((part, index) => {
    if (part.startsWith('## ')) {
      const headerText = part.replace(/^## /, '');
      return (
        <h2 key={index} className="text-2xl font-bold text-dark-100 mt-8 mb-4">
          {headerText}
        </h2>
      );
    }

    if (part.startsWith('```')) {
      const codeContent = part.replace(/^```[\w]*\n?/, '').replace(/\n?```$/, '');
      return (
        <pre
          key={index}
          className="bg-dark-800 border border-dark-700 rounded-lg p-4 overflow-x-auto mb-4"
        >
          <code className="text-sm text-dark-200 font-mono">{codeContent}</code>
        </pre>
      );
    }

    if (part.trim()) {
      return (
        <p key={index} className="text-dark-300 leading-relaxed mb-4">
          {part}
        </p>
      );
    }

    return null;
  });
}

function buildArticleJsonLd(post: BlogPost) {
  return {
    '@context': 'https://schema.org',
    '@type': 'Article',
    headline: post.title,
    description: post.description,
    datePublished: new Date(post.date).toISOString(),
    dateModified: new Date(post.date).toISOString(),
    url: `${SITE.url.replace(/\/+$/, '')}/blog/${post.slug}/`,
    author: {
      '@type': 'Organization',
      name: 'DoAide',
      url: 'https://doaide.com',
    },
    publisher: {
      '@type': 'Organization',
      name: 'DoAide',
      url: 'https://doaide.com',
      logo: {
        '@type': 'ImageObject',
        url: `${SITE.url.replace(/\/+$/, '')}/favicon.svg`,
      },
    },
    mainEntityOfPage: {
      '@type': 'WebPage',
      '@id': `${SITE.url.replace(/\/+$/, '')}/blog/${post.slug}/`,
    },
  };
}

export default async function BlogPostPage(props: PageProps) {
  const params = await props.params;
  const post = BLOG_POSTS.find((p) => p.slug === params.slug);

  if (!post) {
    return (
      <div className="px-4 sm:px-6 lg:px-8 py-16">
        <div className="max-w-2xl mx-auto">
          <h1 className="text-4xl font-bold text-dark-50 mb-4">Post Not Found</h1>
          <p className="text-dark-300 mb-6">The blog post you are looking for does not exist.</p>
          <Link href="/blog" className="text-brand-400 hover:text-brand-300 transition-colors">
            &larr; Back to Blog
          </Link>
        </div>
      </div>
    );
  }

  const currentIndex = BLOG_POSTS.findIndex((p) => p.slug === params.slug);
  const previousPost = currentIndex > 0 ? BLOG_POSTS[currentIndex - 1] : null;
  const nextPost = currentIndex < BLOG_POSTS.length - 1 ? BLOG_POSTS[currentIndex + 1] : null;

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-2xl mx-auto">
        <Link
          href="/blog"
          className="text-brand-400 hover:text-brand-300 transition-colors text-sm font-medium mb-8 inline-block"
        >
          &larr; Back to Blog
        </Link>

        <article>
          <header className="mb-8">
            <div className="flex items-center gap-3 mb-4">
              <span className="text-xs px-2 py-0.5 rounded-full bg-brand-500/10 text-brand-400 border border-brand-500/20">
                {post.tag}
              </span>
              <time className="text-xs text-dark-400">{post.date}</time>
            </div>
            <h1 className="text-4xl font-bold text-dark-50 mb-4">{post.title}</h1>
            <p className="text-dark-300 text-lg">{post.description}</p>
          </header>

          <div className="prose prose-invert max-w-none mb-12">
            {renderBody(post.body)}
          </div>
        </article>

        <nav className="border-t border-dark-700 pt-8 mt-12">
          <div className="grid grid-cols-2 gap-6">
            {previousPost ? (
              <Link
                href={previousPost.href}
                className="group text-left hover:opacity-75 transition-opacity"
              >
                <span className="text-xs text-dark-400 mb-2 block">&larr; Previous Post</span>
                <h3 className="text-lg font-semibold text-dark-100 group-hover:text-brand-400 transition-colors">
                  {previousPost.title}
                </h3>
              </Link>
            ) : (
              <div />
            )}

            {nextPost ? (
              <Link
                href={nextPost.href}
                className="group text-right hover:opacity-75 transition-opacity"
              >
                <span className="text-xs text-dark-400 mb-2 block">Next Post &rarr;</span>
                <h3 className="text-lg font-semibold text-dark-100 group-hover:text-brand-400 transition-colors">
                  {nextPost.title}
                </h3>
              </Link>
            ) : (
              <div />
            )}
          </div>
        </nav>
      </div>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(buildArticleJsonLd(post)) }}
      />
    </div>
  );
}
