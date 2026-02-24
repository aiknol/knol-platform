import type { Metadata } from 'next';
import './globals.css';
import HMRErrorHandler from '@/components/HMRErrorHandler';

export const metadata: Metadata = {
  title: 'Knol Demo',
  description: 'Knol interactive demo',
  icons: {
    icon: '/favicon.svg',
    shortcut: '/favicon.svg',
    apple: '/favicon.svg',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <HMRErrorHandler />
        {children}
      </body>
    </html>
  );
}
