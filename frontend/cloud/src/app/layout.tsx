import type { Metadata } from 'next';
import './globals.css';
import AppShell from '@/components/AppShell';
import HMRErrorHandler from '@/components/HMRErrorHandler';
import { SITE, pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Cloud'),
  description: `${SITE.name} tenant workspace`,
  icons: {
    icon: '/favicon.svg',
    shortcut: '/favicon.svg',
    apple: '/favicon.svg',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="font-sans bg-gradient-dark min-h-screen">
        <HMRErrorHandler />
        <AppShell>{children}</AppShell>
      </body>
    </html>
  );
}
