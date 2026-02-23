import { Metadata } from 'next';
import Link from 'next/link';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('MCP Server — Give Claude Persistent Memory'),
  description:
    'Use Knol as an MCP server to give Claude, Cursor, and any MCP-compatible AI tool persistent memory. Install in one command, no configuration required.',
  keywords: ['MCP', 'Model Context Protocol', 'Claude memory', 'Cursor memory', 'AI memory', 'MCP server', 'persistent memory'],
};

export default function MCPPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">
        {/* Hero */}
        <section className="mb-16 text-center">
          <span className="text-xs px-3 py-1 rounded-full bg-brand-500/10 text-brand-400 border border-brand-500/20 mb-4 inline-block">
            Model Context Protocol
          </span>
          <h1 className="text-3xl sm:text-4xl md:text-5xl font-bold text-dark-50 mb-6">
            Give Claude Persistent Memory
          </h1>
          <p className="text-lg md:text-xl text-dark-300 max-w-2xl mx-auto mb-8">
            Knol&apos;s MCP server lets Claude Desktop, Cursor, Windsurf, and any MCP-compatible tool
            remember users, learn preferences, and build knowledge across sessions.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href={SITE.appUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium"
            >
              Get Started Free
            </a>
            <a href={SITE.docsUrl} target="_blank" rel="noopener noreferrer" className="btn-secondary px-6 py-3 rounded-lg font-medium">
              Read the Docs
            </a>
          </div>
        </section>

        {/* One-command install */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6 text-center">Install in One Command</h2>
          <div className="bg-dark-800 border border-dark-700 rounded-lg p-6 overflow-x-auto">
            <code className="text-sm text-brand-400 font-mono">
              npx @aiknol/knol-mcp-server
            </code>
          </div>
          <p className="text-dark-400 text-sm mt-3 text-center">
            No Docker, no PostgreSQL setup required for local use. Just run and connect.
          </p>
        </section>

        {/* How it works */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-8">How It Works</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <div className="text-3xl mb-3">1</div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Install the MCP Server</h3>
              <p className="text-dark-300 text-sm">
                Run the npx command or add Knol to your Claude Desktop MCP config. The server starts
                automatically and connects to your local Knol instance.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <div className="text-3xl mb-3">2</div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Claude Learns From Conversations</h3>
              <p className="text-dark-300 text-sm">
                As you chat, Knol automatically extracts facts, preferences, and relationships.
                These are stored as structured memories with temporal context.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <div className="text-3xl mb-3">3</div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">Every Session Gets Smarter</h3>
              <p className="text-dark-300 text-sm">
                Next time you open Claude, it already knows your projects, preferences, and context.
                No more repeating yourself. Memory persists across sessions.
              </p>
            </div>
          </div>
        </section>

        {/* Claude Desktop config */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Claude Desktop Configuration</h2>
          <p className="text-dark-300 mb-4">
            Add this to your Claude Desktop MCP configuration file:
          </p>
          <div className="bg-dark-800 border border-dark-700 rounded-lg p-6 overflow-x-auto mb-4">
            <pre className="text-sm text-dark-200 font-mono">{`{
  "mcpServers": {
    "knol-memory": {
      "command": "npx",
      "args": ["@aiknol/knol-mcp-server"],
      "env": {
        "KNOL_API_URL": "http://localhost:3000",
        "KNOL_API_KEY": "your-api-key"
      }
    }
  }
}`}</pre>
          </div>
          <p className="text-dark-400 text-sm">
            Config file location: <code className="text-dark-300">~/Library/Application Support/Claude/claude_desktop_config.json</code> (macOS)
            or <code className="text-dark-300">%APPDATA%/Claude/claude_desktop_config.json</code> (Windows)
          </p>
        </section>

        {/* MCP Tools */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Available MCP Tools</h2>
          <p className="text-dark-300 mb-6">
            Once connected, Claude can use these tools to manage persistent memory:
          </p>
          <div className="space-y-4">
            {[
              { tool: 'knol_store_memory', desc: 'Store a new memory from the current conversation — facts, preferences, decisions, or context.' },
              { tool: 'knol_search_memory', desc: 'Search past memories using natural language. Uses hybrid retrieval (vector + BM25 + graph) for accurate results.' },
              { tool: 'knol_get_user_context', desc: 'Retrieve a summary of everything known about the current user — preferences, history, and relationships.' },
              { tool: 'knol_list_memories', desc: 'List recent memories with optional filters by type, date range, or topic.' },
              { tool: 'knol_delete_memory', desc: 'Remove a specific memory by ID. Supports GDPR right-to-erasure compliance.' },
              { tool: 'knol_graph_query', desc: 'Traverse the knowledge graph to find relationships between entities (people, projects, concepts).' },
            ].map((item) => (
              <div key={item.tool} className="bg-dark-700/30 border border-dark-600 rounded-lg p-4 flex flex-col sm:flex-row gap-2 sm:gap-4">
                <code className="text-brand-400 font-mono text-sm sm:whitespace-nowrap">{item.tool}</code>
                <p className="text-dark-300 text-sm">{item.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Use cases */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">What You Can Build</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Personal AI Assistant</h3>
              <p className="text-dark-300 text-sm">
                Claude remembers your projects, coding preferences, writing style, and ongoing tasks.
                Ask &quot;what was I working on last week?&quot; and get accurate answers.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Team Knowledge Base</h3>
              <p className="text-dark-300 text-sm">
                Share memory across team members. Decisions, architecture choices, and project context
                are captured and retrievable by anyone on the team.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Cursor / Windsurf Integration</h3>
              <p className="text-dark-300 text-sm">
                Your AI coding assistant remembers your codebase patterns, tech stack preferences,
                and past debugging sessions. Context carries across projects.
              </p>
            </div>
            <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-brand-400 mb-2">Customer Support Agent</h3>
              <p className="text-dark-300 text-sm">
                Build support bots that remember every customer interaction. Prior tickets,
                preferences, and account details are available instantly.
              </p>
            </div>
          </div>
        </section>

        {/* Why Knol for MCP */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-dark-50 mb-6">Why Knol for MCP</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              Most MCP memory servers store flat key-value pairs or simple text blobs.
              Knol is a full context engineering platform with hybrid retrieval, knowledge graphs,
              memory decay, and conflict detection — all exposed through MCP tools.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
              <div className="text-center p-4">
                <div className="text-2xl font-bold text-brand-400 mb-1">&lt;5ms</div>
                <div className="text-sm text-dark-400">Memory retrieval latency</div>
              </div>
              <div className="text-center p-4">
                <div className="text-2xl font-bold text-brand-400 mb-1">Hybrid</div>
                <div className="text-sm text-dark-400">Vector + BM25 + Graph search</div>
              </div>
              <div className="text-center p-4">
                <div className="text-2xl font-bold text-brand-400 mb-1">Temporal</div>
                <div className="text-sm text-dark-400">Facts evolve over time</div>
              </div>
            </div>
          </div>
        </section>

        {/* Self-host CTA */}
        <section className="bg-dark-700/30 border border-dark-600 rounded-lg p-8 text-center">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">
            Ready to Give Your AI Persistent Memory?
          </h2>
          <p className="text-dark-300 mb-6 max-w-xl mx-auto">
            Knol is open-source and self-hostable. Deploy on your infrastructure,
            keep your data private, and give every AI tool you use persistent memory.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary px-6 py-3 rounded-lg font-medium text-center"
            >
              Star on GitHub
            </a>
            <a href={SITE.demoUrl} target="_blank" rel="noopener noreferrer" className="btn-secondary px-6 py-3 rounded-lg font-medium text-center">
              Try the Demo
            </a>
            <Link href="/pricing/" className="btn-secondary px-6 py-3 rounded-lg font-medium text-center">
              View Pricing
            </Link>
          </div>
        </section>
      </div>
    </div>
  );
}
