import Link from 'next/link';
import { DOCS_SITE } from '@/config/site';
import { getDocsBySection } from '@/lib/docs';

export default function HomePage() {
  const tenantDocs = getDocsBySection('Tenant');
  const ossDocs = getDocsBySection('OSS');

  return (
    <>
      <section className="hero">
        <h1>Documentation Hub for Knol</h1>
        <p>
          This site centralizes Knol OSS documentation and complete tenant-service documentation.
          Use it as the single source across environments at
          <code> {DOCS_SITE.siteUrl}</code>.
        </p>
        <div className="heroActions">
          <Link href="/api/" className="btn btnPrimary">
            Open API Reference
          </Link>
          <Link href="/library/" className="btn btnSubtle">
            Browse Documentation Library
          </Link>
        </div>
      </section>

      <section className="section">
        <h2>Tenant Documentation</h2>
        <div className="grid">
          {tenantDocs.map((doc) => (
            <Link key={doc.slug} href={`/library/${doc.slug}/`} className="card">
              <h3>{doc.title}</h3>
              <p>{doc.summary}</p>
              <div className="cardMeta">
                <span className="badge">Tenant</span>
                <span>{doc.kind.toUpperCase()}</span>
              </div>
            </Link>
          ))}
        </div>
      </section>

      <section className="section">
        <h2>OSS Code Documentation</h2>
        <div className="grid">
          {ossDocs.map((doc) => (
            <Link key={doc.slug} href={`/library/${doc.slug}/`} className="card">
              <h3>{doc.title}</h3>
              <p>{doc.summary}</p>
              <div className="cardMeta">
                <span className="badge">OSS</span>
                <span>{doc.kind.toUpperCase()}</span>
              </div>
            </Link>
          ))}
        </div>
      </section>

      <section className="section">
        <h2>API Documentation Coverage</h2>
        <div className="card">
          <ul className="inlineList">
            <li>Gateway REST API (`/v1/memory`, `/v1/memory/search`, update/delete/export/import)</li>
            <li>Authentication and tenant-scoped API key behavior</li>
            <li>Config-driven service URLs for API calls and docs links</li>
            <li>Tenant service OpenAPI/Swagger and workspace endpoint references</li>
          </ul>
          <div className="docActions">
            <Link href="/api/" className="btn btnPrimary">
              Go to API Reference
            </Link>
          </div>
        </div>
      </section>
    </>
  );
}
