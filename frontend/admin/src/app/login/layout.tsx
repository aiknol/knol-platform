import type { Metadata } from 'next';
import { pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Admin Login'),
  description: 'Admin panel login',
};

export default function LoginLayout({ children }: { children: React.ReactNode }) {
  return children;
}
