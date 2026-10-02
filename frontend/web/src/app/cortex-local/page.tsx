import { Metadata } from 'next';
import CodeBlock from '@/components/ui/CodeBlock';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('cortex-local — Local MCP Memory for Claude & Cursor'),
  description:
    'cortex-local is a zero-setup, SQLite-backed MCP server and CLI that gives Claude Desktop, Cursor, and Claude Code persistent local memory. Install with npm, no Docker or API key required.',
  keywords: [
    'cortex-local', 'local memory', 'MCP server', 'Claude memory', 'Cursor memory',
    'Claude Code memory', 'SQLite memory', 'offline AI memory', 'npm cortex-local',
  ],
};

const setupCode = `# Install globally — auto-configures Claude Desktop & Cursor
npm install -g cortex-local

# Or configure manually for a specific client
cortex-local setup claude       # Claude Desktop
cortex-local setup cursor       # Cursor
cortex-local setup claude-code  # Claude Code CLI
cortex-local setup codex        # Codex (shows HTTP instructions)`;

const cliCode = `# Add a memory
cortex-local add "Prefer strict TypeScript and functional patterns" --tag coding

# Search memories
cortex-local search "TypeScript preferences" --limit 5

# List all memories
cortex-local list --limit 20 --tag coding

# Summary statistics
cortex-local stats

# Export / import
cortex-local export --out backup.json
cortex-local import backup.json

# Backup / restore the SQLite database
cortex-local backup --out ~/backups/
cortex-local restore ~/backups/memories-2026-05-09.db

# Start the HTTP REST API (for Codex or scripts)
cortex-local serve --port 3001 --key my-secret`;

const claudeCodeMcpCode = `# One-time setup via Claude Code CLI
claude mcp add cortex-local cortex-local

# Or add per-project in .claude/settings.json
{
  "mcpServers": {
    "cortex-local": { "command": "cortex-local" }
  }
}`;

const httpCode = `# Start the REST server
cortex-local serve --port 3001

# Add a memory
curl -X POST http://localhost:3001/memories \\
  -H "Content-Type: application/json" \\
  -d '{"content": "Prefer functional patterns", "tags": ["coding"]}'

# Search
curl "http://localhost:3001/memories/search?q=TypeScript&limit=5"

# Health check
curl http://localhost:3001/health`;

const mcpTools = [
  { tool: 'remember',       desc: 'Store a memory with optional tags and importance score (0–1).' },
  { tool: 'recall',         desc: 'Full-text search across memories. Returns ranked results with scores.' },
  { tool: 'forget',         desc: 'Delete a memory by ID.' },
  { tool: 'list_memories',  desc: 'List recent memories with optional tag filters and limit.' },
  { tool: 'update_memory',  desc: 'Update the content, tags, or importance of an existing memory.' },
  { tool: 'memory_stats',   desc: 'Return total count, oldest, and newest memory timestamps.' },
];

const httpEndpoints = [
  ['GET',    '/memories',         'List memories (supports ?tag=&limit=)'],
  ['POST',   '/memories',         'Add a memory { content, tags?, importance? }'],
  ['GET',    '/memories/search',  'Full-text search (?q=&limit=&tag=)'],
  ['DELETE', '/memories/:id',     'Delete a memory by ID'],
  ['GET',    '/export',           'Export all memories as JSON'],
  ['POST',   '/import',           'Import memories from JSON'],
  ['GET',    '/health',           'Health check'],
];

export default function CortexLocalPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">

        {/* Hero */}
        <section className="mb-16 text-center">
          <span className="text-xs px-3 py-1 rounded-full bg-brand-500/10 text-brand-400 border border-brand-500/20 mb-4 inline-block font-mono">
            npm install -g cortex-local
          </span>
          <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold text-dark-50 mb-6">
            Local Memory for AI Assistants
          </h1>
          <p className="text-lg md:text-xl text-dark-300 max-w-2xl mx-auto mb-8">
            <strong className="text-dark-100">cortex-local</strong> is a lightweight MCP server and CLI backed by SQLite.
            Zero setup — no Docker, no PostgreSQL, no API key. Install once and your AI tools remember everything.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href="https://www.npmjs.com/package/cortex-local"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium"
            >
              View on npm
            </a>
            <a
              href="https://github.com/doaide/cortex-local"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-secondary px-6 py-3 rounded-lg font-medium"
            >
              GitHub →
            </a>
            <a href="/docs/#cortex-local" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              API Reference →
            </a>
          </div>
        </section>

        {/* Feature highlights */}
        <section className="mb-16">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {[
              { badge: 'SQLite + FTS5',     title: 'No server required',    desc: 'All data lives in ~/.cortex-local/memories.db. Works fully offline.' },
              { badge: 'MCP native',        title: 'Claude, Cursor & more', desc: 'Exposes 6 tools via the Model Context Protocol. Auto-configures on install.' },
              { badge: 'HTTP REST API',     title: 'Works with Codex',      desc: 'Run cortex-local serve for a local REST endpoint any tool can hit.' },
              { badge: 'MIT · Node 18+',    title: 'Open source',           desc: 'Uses built-in node:sqlite on Node 22.5+. Zero extra deps for most users.' },
            ].map((f) => (
              <div key={f.title} className="card">
                <span className="inline-block mb-2 px-2 py-0.5 rounded text-xs font-mono text-brand-300 bg-brand-500/10 border border-brand-500/20">
                  {f.badge}
                </span>
                <h3 className="text-sm font-semibold text-dark-100 mb-1">{f.title}</h3>
                <p className="text-dark-400 text-xs leading-relaxed">{f.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Install & setup */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Install & Setup</h2>
          <p className="text-dark-300 mb-6">
            The postinstall script automatically patches any existing Claude Desktop or Cursor config files.
            Nothing is created if the config file doesn&apos;t already exist — no surprises.
          </p>
          <CodeBlock code={setupCode} language="bash" title="Installation" />
        </section>

        {/* How it works */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-8">How It Works</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {[
              {
                num: '01',
                title: 'Install once',
                desc: 'npm install -g cortex-local patches your Claude Desktop and Cursor MCP configs automatically. No manual JSON editing.',
              },
              {
                num: '02',
                title: 'AI tools use the MCP tools',
                desc: 'Claude calls remember, recall, and list_memories during your conversations. Memories persist across sessions in a local SQLite file.',
              },
              {
                num: '03',
                title: 'Every session picks up where you left off',
                desc: 'Your projects, preferences, and decisions are recalled automatically. You never repeat yourself between sessions.',
              },
            ].map((step) => (
              <div key={step.num} className="card">
                <span className="text-brand-500 font-mono text-sm font-bold">{step.num}</span>
                <h3 className="text-lg font-semibold text-dark-50 mt-3 mb-2">{step.title}</h3>
                <p className="text-dark-300 text-sm leading-relaxed">{step.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* MCP tools */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">MCP Tools</h2>
          <p className="text-dark-300 mb-6">
            Once connected, Claude (or any MCP client) can call these six tools automatically:
          </p>
          <div className="space-y-3">
            {mcpTools.map((item) => (
              <div
                key={item.tool}
                className="bg-dark-700/30 border border-dark-600 rounded-lg p-4 flex flex-col sm:flex-row gap-2 sm:gap-6"
              >
                <code className="text-brand-400 font-mono text-sm sm:w-36 shrink-0">{item.tool}</code>
                <p className="text-dark-300 text-sm">{item.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Claude Code */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Claude Code Setup</h2>
          <p className="text-dark-300 mb-6">
            Claude Code uses its own config. Add cortex-local globally with one command, or per-project via{' '}
            <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">.claude/settings.json</code>.
          </p>
          <CodeBlock code={claudeCodeMcpCode} language="bash" title="Claude Code CLI" />
        </section>

        {/* CLI reference */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">CLI Reference</h2>
          <p className="text-dark-300 mb-6">
            Manage memories directly from the terminal — useful for scripting, backup automation, or quick lookups.
          </p>
          <CodeBlock code={cliCode} language="bash" title="CLI commands" />
        </section>

        {/* HTTP API */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">HTTP REST API</h2>
          <p className="text-dark-300 mb-6">
            Run <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">cortex-local serve</code> to start a local
            REST server — useful for Codex, scripts, or any tool without native MCP support.
            Optionally protect it with <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">--key</code>.
          </p>
          <div className="overflow-x-auto mb-6">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-600/30">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Method</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Path</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Description</th>
                </tr>
              </thead>
              <tbody>
                {httpEndpoints.map(([method, path, desc]) => (
                  <tr key={path + method} className="border-b border-dark-600/20">
                    <td className="py-3 px-4">
                      <span className={`font-mono text-xs px-2 py-0.5 rounded ${
                        method === 'GET'    ? 'bg-green-900/30 text-green-400' :
                        method === 'POST'   ? 'bg-blue-900/30 text-blue-400'  :
                        method === 'DELETE' ? 'bg-red-900/30 text-red-400'    :
                                             'bg-yellow-900/30 text-yellow-400'
                      }`}>{method}</span>
                    </td>
                    <td className="py-3 px-4 font-mono text-dark-200 text-xs">{path}</td>
                    <td className="py-3 px-4 text-dark-300">{desc}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <CodeBlock code={httpCode} language="bash" title="HTTP API examples" />
        </section>

        {/* Config */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">Configuration</h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-600/30">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Variable</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Default</th>
                  <th className="text-left py-3 px-4 text-dark-300 font-medium">Description</th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-b border-dark-600/20">
                  <td className="py-3 px-4 font-mono text-brand-400 text-xs">KNOL_LOCAL_DB</td>
                  <td className="py-3 px-4 font-mono text-dark-400 text-xs">~/.cortex-local/memories.db</td>
                  <td className="py-3 px-4 text-dark-300">Override the SQLite database path</td>
                </tr>
              </tbody>
            </table>
          </div>
          <p className="text-dark-400 text-sm mt-4">
            Node 22.5+ uses the built-in{' '}
            <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">node:sqlite</code> — no extra
            dependencies. Older runtimes (Claude Desktop embeds Node 18) auto-install{' '}
            <code className="text-brand-200 bg-brand-500/15 px-1.5 rounded">better-sqlite3</code> during postinstall.
          </p>
        </section>

        {/* vs full Cortex */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">cortex-local vs Cortex</h2>
          <div className="overflow-x-auto rounded-xl border border-dark-600/30">
            <table className="w-full text-sm min-w-[480px]">
              <thead>
                <tr className="bg-dark-800/80">
                  <th className="text-left py-3 px-4 text-dark-300 font-medium"></th>
                  <th className="text-center py-3 px-4 text-dark-300 font-medium">cortex-local</th>
                  <th className="text-center py-3 px-4 text-brand-400 font-semibold">Cortex (full)</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ['Setup',              'npm install -g',         'Docker / self-host / cloud'],
                  ['Storage',           'SQLite (local file)',    'PostgreSQL + pgvector'],
                  ['Retrieval',         'Full-text (FTS5)',       'Vector + BM25 + graph (hybrid)'],
                  ['Knowledge graph',   '—',                     '✓'],
                  ['Multi-user',        '—',                     '✓ (RLS isolation)'],
                  ['Memory decay',      '—',                     '✓'],
                  ['HTTP REST API',     '✓ (local)',              '✓ (production-grade)'],
                  ['MCP tools',         '6 tools',               '6 tools + graph query'],
                  ['Conflict detection','—',                     '✓'],
                  ['Best for',         'Personal use / local dev','Teams & production apps'],
                ].map(([feature, local, full]) => (
                  <tr key={feature} className="border-t border-dark-600/20 hover:bg-dark-800/30 transition-colors">
                    <td className="py-3 px-4 text-dark-200">{feature}</td>
                    <td className="py-3 px-4 text-center text-dark-300 text-xs">{local}</td>
                    <td className="py-3 px-4 text-center text-dark-300 text-xs bg-brand-500/5">{full}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* CTA */}
        <section className="bg-dark-700/30 border border-dark-600 rounded-xl p-8 text-center">
          <h2 className="text-2xl font-bold text-dark-50 mb-3">
            Ready to give your AI persistent memory?
          </h2>
          <p className="text-dark-300 mb-6 max-w-xl mx-auto">
            One command. Works immediately with Claude Desktop, Cursor, and Claude Code.
            Upgrade to the full Cortex stack when you need teams, graphs, or hybrid retrieval.
          </p>
          <div className="bg-dark-800 border border-dark-600 rounded-lg px-6 py-4 font-mono text-brand-300 text-sm mb-6 inline-block">
            npm install -g cortex-local
          </div>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href="https://www.npmjs.com/package/cortex-local"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium"
            >
              View on npm
            </a>
            <a
              href="https://github.com/doaide/cortex-local"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-secondary px-6 py-3 rounded-lg font-medium"
            >
              Star on GitHub
            </a>
            <a href="/pricing/" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              See Full Cortex Plans
            </a>
          </div>
        </section>

      </div>
    </div>
  );
}
