import type { Metadata } from 'next';
import './globals.css';
import AdminShell from '@/components/AdminShell';
import { SITE, pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Admin'),
  description: `${SITE.name} managed control plane`,
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
        <AdminShell>{children}</AdminShell>
      </body>
    </html>
  );
}
