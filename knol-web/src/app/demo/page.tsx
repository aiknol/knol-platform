import { Metadata } from 'next';
import { pageTitle } from '@/config';

export const metadata: Metadata = {
  title: pageTitle('Live Demo'),
  description:
    'Interactive demo of Knol — persistent memory for AI applications. Watch the AI learn, remember, and build a knowledge graph in real time.',
};

export default function DemoPage() {
  return (
    <div className="fixed inset-0 bg-[#09090b]">
      <iframe
        src="/demo/index.html"
        className="w-full h-full border-0"
        title="Knol Interactive Demo"
        allow="clipboard-write"
      />
    </div>
  );
}
