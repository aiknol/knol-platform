import Link from 'next/link';

export default function NotFound() {
  return (
    <section className="card">
      <h1>Document not found</h1>
      <p>The requested page does not exist in the docs library.</p>
      <p>
        <Link href="/library/">Go back to Documentation Library</Link>
      </p>
    </section>
  );
}
