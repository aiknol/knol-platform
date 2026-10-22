import { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { resolveAppSignupUrl } from '@/config/urls';
import { pageTitle } from '@/config/site';
import NoAccountNeeded from '@/components/marketing/NoAccountNeeded';

export const metadata: Metadata = {
  title: pageTitle('Get started'),
  description: 'Get started with Cortex without signing up: live demo, Cortex Local for MCP clients, and self-hosted Cortex OSS.',
  alternates: { canonical: '/signup/' },
  robots: { index: false, follow: true },
};

export default function SignupPage() {
  const url = resolveAppSignupUrl();
  if (/^https?:\/\//.test(url)) redirect(url);
  return <NoAccountNeeded heading="Start using Cortex — no signup" />;
}
