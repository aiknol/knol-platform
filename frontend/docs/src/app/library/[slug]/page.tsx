import { notFound } from 'next/navigation';
import { marked } from 'marked';
import { getAllDocs, getDocBySlug, githubSourceUrl, readDocContent } from '@/lib/docs';

interface DocPageProps {
  params: Promise<{ slug: string }>;
}

export function generateStaticParams() {
  return getAllDocs().map((doc) => ({ slug: doc.slug }));
}

export default async function DocPage({ params }: DocPageProps) {
  const { slug } = await params;
  const doc = getDocBySlug(slug);
  if (!doc) notFound();

  const content = readDocContent(doc);

  return (
    <>
      <section className="docHeader">
        <h1>{doc.title}</h1>
        <p>{doc.summary}</p>
        <div className="docActions">
          <a className="btn btnPrimary" href={githubSourceUrl(doc)} target="_blank" rel="noopener noreferrer">
            View Source on GitHub
          </a>
        </div>
      </section>

      {doc.kind === 'markdown' ? (
        <article
          className="docContent markdown"
          dangerouslySetInnerHTML={{ __html: marked.parse(content) as string }}
        />
      ) : (
        <iframe className="docFrame" srcDoc={content} title={doc.title} />
      )}
    </>
  );
}
