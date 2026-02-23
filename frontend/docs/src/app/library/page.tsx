import Link from 'next/link';
import { getAllDocs } from '@/lib/docs';

export default function LibraryPage() {
  const docs = getAllDocs();

  return (
    <>
      <section className="docHeader">
        <h1>Documentation Library</h1>
        <p>All tenant and OSS docs available from this public documentation site.</p>
      </section>
      <section className="grid">
        {docs.map((doc) => (
          <Link key={doc.slug} href={`/library/${doc.slug}/`} className="card">
            <h3>{doc.title}</h3>
            <p>{doc.summary}</p>
            <div className="cardMeta">
              <span className="badge">{doc.section}</span>
              <span>{doc.kind.toUpperCase()}</span>
            </div>
          </Link>
        ))}
      </section>
    </>
  );
}
