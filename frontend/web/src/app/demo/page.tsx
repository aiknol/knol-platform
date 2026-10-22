import { Metadata } from 'next';
import { pageTitle } from '@/config';
import { withSeo } from '@/config/site';

export const metadata: Metadata = withSeo('/demo/', {
  title: pageTitle('Live Demo'),
  description:
    'Interactive demo of Cortex — persistent memory for AI applications. Watch the AI learn, remember, and build a knowledge graph in real time.',
});

export default function DemoPage() {
  return (
    <div className="fixed inset-0 bg-[#09090b]">
      <iframe
        src="/live-demo/"
        className="w-full h-full border-0"
        title="Cortex Interactive Demo"
        allow="clipboard-write"
      />
    </div>
  );
}
