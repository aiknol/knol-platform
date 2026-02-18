#!/usr/bin/env node
// =============================================================================
// Knol Marketing — Content Generation Engine
// Generates marketing content from templates + optional Claude API enhancement
// =============================================================================

const fs = require('fs');
const path = require('path');
const https = require('https');

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const PRODUCT = {
  name: 'Knol',
  tagline: 'Memory infrastructure for AI',
  description: 'Open-source long-term memory layer for LLM applications. Rust-powered microservices with vector search, knowledge graphs, and bi-temporal data.',
  url: 'https://aiknol.com',
  github: 'https://github.com/aiknol/knol',
  pypi: 'pip install knol',
  features: [
    'Vector similarity search (pgvector + HNSW)',
    'Bi-temporal knowledge graph',
    'Intent-aware hybrid retrieval (RRF fusion)',
    'Multi-tenant with PostgreSQL RLS',
    'Sub-10ms p99 retrieval latency',
    'Open-core: Apache 2.0 OSS + enterprise tier',
    '8 Rust microservices, ~30MB RAM per service',
    'LLM entity extraction via Claude Haiku',
  ],
  audiences: ['AI engineers', 'LLM application developers', 'Rust developers', 'DevOps/infra engineers', 'startup CTOs'],
  competitors: ['Mem0', 'Zep', 'Motorhead', 'LangChain Memory'],
  differentiators: ['Rust performance', 'bi-temporal graph', 'open-source', 'intent-aware search', 'multi-tenant isolation'],
};

const CONTENT_TYPES = {
  tweet: { maxLength: 280, platform: 'twitter' },
  tweet_thread: { maxLength: 280, count: 5, platform: 'twitter' },
  linkedin_post: { maxLength: 3000, platform: 'linkedin' },
  reddit_post: { maxLength: 10000, platform: 'reddit' },
  devto_article: { maxLength: 50000, platform: 'devto' },
  hn_post: { maxLength: 200, platform: 'hackernews' },
  blog_post: { maxLength: 50000, platform: 'blog' },
  email_newsletter: { maxLength: 20000, platform: 'email' },
  github_release: { maxLength: 5000, platform: 'github' },
  changelog: { maxLength: 3000, platform: 'github' },
};

// ---------------------------------------------------------------------------
// Template-based generation (zero cost, no API needed)
// ---------------------------------------------------------------------------

const TEMPLATES = {
  // --- Twitter ---
  tweet_launch: [
    `🧠 ${PRODUCT.name} — ${PRODUCT.tagline}\n\nOpen-source memory layer for LLMs. Rust-powered, sub-10ms retrieval.\n\n${PRODUCT.pypi}\n${PRODUCT.github}`,
    `Most AI apps forget everything between sessions.\n\n${PRODUCT.name} fixes that — persistent memory with vector search + knowledge graphs.\n\nOpen-source, Apache 2.0.\n${PRODUCT.github}`,
    `Why does your AI assistant forget your preferences every conversation?\n\nBecause it has no memory layer.\n\n${PRODUCT.name}: ${PRODUCT.tagline}\n${PRODUCT.url}`,
    `"Give an LLM memory" used to mean hacking together a vector DB + prompt stuffing.\n\n${PRODUCT.name} is purpose-built memory infrastructure:\n→ Vector search\n→ Knowledge graph\n→ Bi-temporal queries\n→ Multi-tenant\n\n${PRODUCT.github}`,
  ],

  tweet_technical: [
    `How ${PRODUCT.name} searches memory in <10ms:\n\n1. Intent classification (4 types)\n2. Parallel vector + BM25 + graph search\n3. RRF fusion with intent-weighted scores\n4. Scope cascade (session→user→org)\n\nAll in Rust. All open-source.\n${PRODUCT.github}`,
    `${PRODUCT.name}'s architecture:\n\n• 8 Rust microservices (~30MB RAM each)\n• PostgreSQL + pgvector\n• NATS JetStream for async\n• Redis for caching\n• Claude Haiku for entity extraction\n\nRuns on a single $8/mo VPS.\n${PRODUCT.github}`,
    `Bi-temporal data in ${PRODUCT.name}:\n\nEvery memory has valid_from and valid_to.\n\nYou can replay your AI's knowledge at any point in time.\n\n"What did you know on March 15?"\n\nTime-travel for AI memory.\n${PRODUCT.url}`,
  ],

  tweet_comparison: [
    `${PRODUCT.name} vs in-memory LLM context:\n\n❌ Context window: forgets after 128K tokens\n✅ ${PRODUCT.name}: persistent, searchable, graph-linked\n\nYour AI should remember, not just process.\n${PRODUCT.url}`,
    `Why we built ${PRODUCT.name} in Rust instead of Python:\n\n→ 30MB per service (vs 500MB+ Python)\n→ Sub-10ms p99 latency\n→ Zero GC pauses\n→ Type-safe concurrency\n→ Runs 8 services on a $8/mo VPS\n\n${PRODUCT.github}`,
  ],

  // --- LinkedIn ---
  linkedin_launch: [
    `I'm excited to share ${PRODUCT.name} — open-source memory infrastructure for AI applications.\n\nThe problem: LLMs have no persistent memory. Every conversation starts from scratch. Context windows are finite. RAG helps with documents, but what about remembering user preferences, past decisions, and evolving knowledge?\n\n${PRODUCT.name} solves this with:\n\n→ Vector similarity search for semantic retrieval\n→ Bi-temporal knowledge graph for relationship tracking\n→ Intent-aware hybrid search (vector + BM25 + graph fusion)\n→ Multi-tenant isolation at the database level\n→ Sub-10ms retrieval latency\n\nBuilt in Rust. 8 microservices. Runs on a single VPS.\n\nOpen-source under Apache 2.0, with an enterprise tier for teams.\n\nCheck it out: ${PRODUCT.github}\n\n#AI #LLM #OpenSource #Rust #MemoryInfrastructure`,
  ],

  linkedin_technical: [
    `How we achieve sub-10ms memory retrieval in ${PRODUCT.name}:\n\nMost "AI memory" solutions are just a vector database with a prompt template. That works for simple cases but breaks down with complex, evolving knowledge.\n\n${PRODUCT.name} uses a 5-stage retrieval pipeline:\n\n1. Intent Classification — detect if the query is about preferences, temporal events, relationships, or general knowledge\n2. Parallel Search — vector similarity + BM25 full-text + graph traversal run concurrently\n3. Scope Cascade — search narrows from session → user → agent → organization\n4. RRF Fusion — Reciprocal Rank Fusion with intent-weighted scores combines all signals\n5. Confidence Filtering — results below threshold are excluded\n\nThe key insight: different queries need different retrieval strategies. "What does Alice prefer?" needs vector search. "Who manages Bob?" needs graph traversal. "What happened last Tuesday?" needs temporal + text.\n\nAll written in Rust with Axum, running on PostgreSQL + pgvector.\n\n${PRODUCT.github}\n\n#Engineering #AI #SystemDesign #Rust`,
  ],

  // --- Reddit ---
  reddit_rust: [
    {
      title: `${PRODUCT.name}: Open-source LLM memory infrastructure built in Rust (8 Axum microservices, pgvector, NATS)`,
      body: `Hi r/rust!\n\nI've been building ${PRODUCT.name}, an open-source memory layer for LLM applications. The entire backend is Rust — 8 microservices built with Axum.\n\n**What it does:** Gives AI applications persistent, searchable memory with vector search, knowledge graphs, and bi-temporal queries.\n\n**Architecture:**\n- 8 Axum microservices (gateway, write, retrieve, graph, admin, jobs, billing, ingest)\n- PostgreSQL + pgvector for vector similarity\n- NATS JetStream for async event processing\n- Redis for rate limiting + caching\n- Claude Haiku for entity extraction\n\n**Why Rust:**\n- Each service uses ~30MB RAM at idle\n- Sub-10ms p99 retrieval latency\n- All 8 services + NATS + MinIO fit on a single $8/mo VPS\n- No GC pauses in the retrieval path\n\n**Interesting patterns:**\n- Intent-aware search: classifies queries and adjusts retrieval weights (vector vs graph vs text)\n- Reciprocal Rank Fusion for combining multiple search signals\n- PostgreSQL RLS for zero-trust multi-tenancy\n- Bi-temporal data model (valid_from/valid_to on every memory)\n\nGitHub: ${PRODUCT.github}\nLicense: Apache 2.0\n\nWould love feedback on the architecture. The crate structure and Cargo workspace setup might be interesting to other Rust devs building microservices.`,
      subreddit: 'r/rust',
    },
  ],

  reddit_ml: [
    {
      title: `Open-source alternative to Mem0/Zep: ${PRODUCT.name} — persistent memory for LLM apps with knowledge graphs`,
      body: `Most LLM memory solutions are just "stuff vectors in a DB and hope for the best." That works for simple Q&A but breaks down when you need:\n\n- Tracking how knowledge evolves over time\n- Understanding relationships between entities\n- Different retrieval strategies for different query types\n\n${PRODUCT.name} takes a different approach:\n\n1. **Hybrid search** — Vector similarity + BM25 + knowledge graph, fused with Reciprocal Rank Fusion\n2. **Intent classification** — Automatically detects if a query needs preference lookup, temporal search, or relationship traversal\n3. **Bi-temporal data** — Every memory has valid_from and valid_to timestamps, enabling point-in-time replay\n4. **Entity extraction** — Claude Haiku automatically extracts entities and relationships from conversations\n\nBuilt in Rust, open-source (Apache 2.0).\n\nGitHub: ${PRODUCT.github}\n\nHappy to answer questions about the architecture or comparison with other memory solutions.`,
      subreddit: 'r/MachineLearning',
    },
  ],

  // --- Dev.to ---
  devto_tutorial: [
    {
      title: `Building AI Applications That Actually Remember: A Guide to ${PRODUCT.name}`,
      tags: ['ai', 'rust', 'opensource', 'llm'],
      body: `---\ntitle: Building AI Applications That Actually Remember\npublished: true\ntags: ai, rust, opensource, llm\n---\n\n## The Problem\n\nEvery AI chatbot you've used has amnesia. Close the tab, and it forgets everything. Even within a conversation, once you exceed the context window, information is lost.\n\nRAG (Retrieval Augmented Generation) helps with documents, but what about:\n- User preferences ("I prefer dark mode and use Vim")\n- Past decisions ("We decided to use Svelte last Tuesday")\n- Evolving relationships ("Alice now manages the engineering team")\n\nThis is the **memory problem** — and it's why AI assistants feel stateless.\n\n## The Solution: ${PRODUCT.name}\n\n${PRODUCT.name} is open-source memory infrastructure for LLM applications. Think of it as a purpose-built database for AI memory.\n\n### Quick Start\n\n\`\`\`python\nfrom knol import KnolClient\n\nclient = KnolClient(api_key="your-key", base_url="https://api.aiknol.com")\n\n# Store a memory\nclient.write(content="User prefers TypeScript over JavaScript", role="user")\n\n# Search memories\nresults = client.search(query="What programming language does the user prefer?")\n# → Returns: "User prefers TypeScript over JavaScript" (score: 0.94)\n\`\`\`\n\n### How It Works\n\n${PRODUCT.name} uses a 5-stage retrieval pipeline:\n\n1. **Intent Classification** — Detects query type (preference, temporal, relational, general)\n2. **Parallel Search** — Vector + BM25 + Graph run concurrently\n3. **Scope Cascade** — Narrows results: session → user → agent → org\n4. **RRF Fusion** — Combines signals with intent-weighted Reciprocal Rank Fusion\n5. **Confidence Filtering** — Returns only high-confidence results\n\n### Architecture\n\n8 Rust microservices:\n- **Gateway** — Auth, rate limiting, request routing\n- **Write** — Episode ingestion + NATS event publishing\n- **Retrieve** — Hybrid search with intent classification\n- **Graph** — LLM entity extraction + knowledge graph\n- **Admin** — CRUD, audit log, policies\n- **Jobs** — Background tasks (decay, dedup, consolidation)\n- **Billing** — Usage metering + plan enforcement\n- **Ingest** — Webhook + bulk connector framework\n\nInfrastructure: PostgreSQL + pgvector, Redis, NATS JetStream, MinIO\n\n### Why Rust?\n\nEach service uses ~30MB RAM. The entire stack (8 services + NATS + MinIO) runs on a single $8/mo Hetzner VPS. Try that with Python microservices.\n\n### Get Started\n\n\`\`\`bash\ngit clone ${PRODUCT.github}\ncd knol\ndocker compose up -d\n\`\`\`\n\nGitHub: ${PRODUCT.github}\nDocs: ${PRODUCT.url}/docs\n\nLicense: Apache 2.0. Contributions welcome.\n`,
    },
  ],

  // --- Hacker News ---
  hn_launch: [
    {
      title: `Show HN: Knol – Open-source memory infrastructure for LLMs (Rust, pgvector, knowledge graphs)`,
      url: PRODUCT.github,
    },
    {
      title: `Knol: Giving LLMs persistent memory with vector search + knowledge graphs (Apache 2.0)`,
      url: PRODUCT.github,
    },
  ],

  // --- Blog ---
  blog_launch: [
    {
      title: 'Introducing Knol: Memory Infrastructure for AI',
      slug: 'introducing-knol',
      description: 'Why we built an open-source memory layer for LLM applications, and how it works under the hood.',
    },
  ],

  blog_technical: [
    {
      title: 'How Knol Achieves Sub-10ms Memory Retrieval',
      slug: 'sub-10ms-retrieval',
      description: 'A deep dive into intent-aware hybrid search with RRF fusion.',
    },
    {
      title: 'Bi-Temporal Data for AI: Time Travel for LLM Memory',
      slug: 'bi-temporal-memory',
      description: 'How valid_from and valid_to enable point-in-time replay of AI knowledge.',
    },
    {
      title: 'Running 8 Microservices on a $8/mo VPS with Rust',
      slug: 'rust-microservices-cost',
      description: 'Why Rust\'s memory efficiency makes microservices economically viable at small scale.',
    },
    {
      title: 'Building a Knowledge Graph from Conversations with Claude Haiku',
      slug: 'llm-entity-extraction',
      description: 'Automatic entity and relationship extraction from unstructured conversation data.',
    },
  ],

  // --- Email Newsletter ---
  email_welcome: [
    {
      subject: `Welcome to ${PRODUCT.name} — Memory Infrastructure for AI`,
      preheader: 'Get started with persistent memory for your LLM applications',
    },
  ],

  email_weekly: [
    {
      subject: `This week in ${PRODUCT.name}: [TOPIC]`,
      preheader: 'New features, community highlights, and engineering deep dives',
    },
  ],
};

// ---------------------------------------------------------------------------
// Claude API enhancement (optional, $0 if using free credits)
// ---------------------------------------------------------------------------

async function enhanceWithClaude(template, contentType, context) {
  const apiKey = process.env.ANTHROPIC_API_KEY;
  if (!apiKey) {
    console.log('  → No ANTHROPIC_API_KEY set, using template-only mode');
    return template;
  }

  const model = process.env.ANTHROPIC_MODEL || 'claude-haiku-4-5-20251001';

  const systemPrompt = `You are a developer marketing writer for ${PRODUCT.name}, an open-source memory infrastructure platform for AI/LLM applications.

Voice: Technical but approachable. Write like a senior engineer explaining something to a peer — precise, no buzzwords, no hype. Use concrete numbers and technical details.

Product facts:
- Name: ${PRODUCT.name}
- Tagline: ${PRODUCT.tagline}
- Built in Rust (8 Axum microservices, ~30MB RAM each)
- PostgreSQL + pgvector for vector search
- Bi-temporal knowledge graph
- Intent-aware hybrid retrieval (RRF fusion)
- Multi-tenant with PostgreSQL RLS
- Sub-10ms p99 latency
- Open-source Apache 2.0
- GitHub: ${PRODUCT.github}
- Website: ${PRODUCT.url}

CRITICAL: Never use buzzwords like "revolutionary", "game-changing", "cutting-edge". Be factual. Show, don't tell.`;

  const userPrompt = `Improve this ${contentType} content. Keep the same structure and key points but make it more engaging and natural. ${context || ''}

Original:
${typeof template === 'string' ? template : JSON.stringify(template, null, 2)}

Return ONLY the improved content, no explanation.`;

  return new Promise((resolve) => {
    const data = JSON.stringify({
      model,
      max_tokens: 2048,
      system: systemPrompt,
      messages: [{ role: 'user', content: userPrompt }],
    });

    const options = {
      hostname: 'api.anthropic.com',
      path: '/v1/messages',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': apiKey,
        'anthropic-version': '2023-06-01',
        'Content-Length': Buffer.byteLength(data),
      },
    };

    const req = https.request(options, (res) => {
      let body = '';
      res.on('data', (chunk) => (body += chunk));
      res.on('end', () => {
        try {
          const response = JSON.parse(body);
          const content = response.content?.[0]?.text || template;
          resolve(content);
        } catch {
          resolve(template);
        }
      });
    });
    req.on('error', () => resolve(template));
    req.setTimeout(15000, () => { req.destroy(); resolve(template); });
    req.write(data);
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Content generation
// ---------------------------------------------------------------------------

function pickRandom(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

async function generateContent(contentType, templateCategory, options = {}) {
  const templates = TEMPLATES[templateCategory];
  if (!templates || templates.length === 0) {
    throw new Error(`No templates found for: ${templateCategory}`);
  }

  const template = options.index !== undefined ? templates[options.index] : pickRandom(templates);

  if (options.enhance) {
    return await enhanceWithClaude(template, contentType, options.context);
  }

  return template;
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

async function main() {
  const args = process.argv.slice(2);
  const type = args[0] || 'tweet';
  const category = args[1] || 'tweet_launch';
  const enhance = args.includes('--enhance');

  console.log(`\n[Knol Marketing] Generating ${type} from ${category}...`);
  const content = await generateContent(type, category, { enhance });

  console.log('\n--- Generated Content ---');
  if (typeof content === 'string') {
    console.log(content);
  } else {
    console.log(JSON.stringify(content, null, 2));
  }
  console.log('--- End ---\n');

  // Write to output file
  const outDir = path.join(__dirname, '..', 'output');
  if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });

  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const outFile = path.join(outDir, `${category}-${timestamp}.json`);
  fs.writeFileSync(outFile, JSON.stringify({ type, category, content, timestamp: new Date().toISOString() }, null, 2));
  console.log(`Saved to: ${outFile}`);
}

// Export for use by other modules
module.exports = { generateContent, TEMPLATES, PRODUCT, CONTENT_TYPES, enhanceWithClaude, pickRandom };

if (require.main === module) {
  main().catch(console.error);
}
