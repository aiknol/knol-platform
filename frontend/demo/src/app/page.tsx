import { ensurePublicEnvIsValid, resolveAdminApiUrl } from '@/config/env';

ensurePublicEnvIsValid();
const demoSrc = `/live-demo/index.html?admin=${encodeURIComponent(resolveAdminApiUrl())}`;

export default function DemoHomePage() {
  return (
    <main className="demoRoot">
      <header className="toolbar">
        <div>
          <p className="label">Knol Demo</p>
          <h1 className="title">Interactive Sandbox</h1>
        </div>
      </header>
      <iframe title="Knol Interactive Demo" src={demoSrc} className="demoFrame" />
    </main>
  );
}
