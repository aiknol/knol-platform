import { Metadata } from 'next';
import Link from 'next/link';
import { pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Not Found'),
};

export default function NotFound() {
  return (
    <div className="min-h-[60vh] flex flex-col items-center justify-center px-4">
      <h1 className="text-6xl font-bold gradient-text mb-4">404</h1>
      <p className="text-xl text-dark-300 mb-8">Page not found</p>
      <Link href="/" className="btn-primary">
        Go Home
      </Link>
    </div>
  );
}
