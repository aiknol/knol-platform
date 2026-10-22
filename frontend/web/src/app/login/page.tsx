import { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { resolveAppLoginUrl } from '@/config/urls';
import { pageTitle, withSeo } from '@/config/site';
import NoAccountNeeded from '@/components/marketing/NoAccountNeeded';

export const metadata: Metadata = withSeo('/login/', {
  title: pageTitle('Log in'),
  description: 'No login needed: try the Cortex live demo, run Cortex Local or self-host Cortex OSS for free.',
  robots: { index: false, follow: true },
});

export default function LoginPage() {
  const url = resolveAppLoginUrl();
  if (/^https?:\/\//.test(url)) redirect(url);
  return <NoAccountNeeded heading="No login needed" />;
}
