# Knol — Zero-Cost Marketing Plan

**Goal:** 5,000 GitHub stars, 500 registered users, and $10K MRR within 6 months — spending $0 on paid ads.

**Core message:** *"The Rust-native memory engine for AI. One binary, one PostgreSQL, sub-5ms latency. Deploy in 60 seconds."*

---

## Phase 1: Foundation (Weeks 1–2)

### Open-Source Launch Prep

**GitHub Repository Polish**
- Write a killer README with animated GIF demo showing a 60-second Docker deploy
- Add "Quick Start" that works in 3 commands: `git clone`, `docker compose up`, `curl` the API
- Include benchmarks table: Knol vs Mem0 vs Zep (latency, memory footprint, dependency count)
- Add badges: CI status, license (Apache 2.0), Docker pulls, Discord members
- Create a `/examples` directory with copy-paste integrations: LangChain, CrewAI, MCP, raw Python, TypeScript

**Social Proof Anchors**
- Create a Discord server with channels: #general, #help, #showcase, #contributors
- Set up GitHub Discussions for Q&A (cheaper than Discourse, keeps traffic on GitHub)
- Write a `CONTRIBUTING.md` with "good first issues" tagged — this is a developer acquisition funnel

---

## Phase 2: Launch Week (Weeks 3–4)

### Day 1 — Hacker News

Post a "Show HN" with this format:

> **Show HN: Knol — Rust-native memory layer for AI agents (open source)**
>
> We built a persistent memory engine for LLM apps in Rust. Unlike Mem0 (Python + Qdrant + Neo4j) or Zep (Python/Go + Neo4j), Knol needs just PostgreSQL. The 50MB binary handles vector search, knowledge graphs, BM25, and memory decay — all at sub-5ms P95 latency.
>
> Key differentiators: bi-temporal memory model, 7-layer LLM cost optimization (75% token savings), conflict detection, and HMAC-signed webhooks. Self-host in 60 seconds with Docker Compose.

**HN Playbook:**
- Post at 8am ET Tuesday/Wednesday (peak HN engagement)
- Be in the comments immediately — answer every question within 10 minutes for the first 3 hours
- Have 3–5 friends/colleagues ready to upvote and leave genuine comments
- Never ask for upvotes publicly — HN penalizes this

### Day 2 — Reddit Blitz

Post to these subreddits (each needs a unique angle, not cross-posting):

- **r/rust** — "We built a production AI memory engine in Rust — here's what we learned about async, sqlx, and axum at scale"
- **r/LocalLLaMA** — "Open-source memory layer that gives your local LLM agents persistent context — just needs PostgreSQL"
- **r/MachineLearning** — "Knol: Bi-temporal knowledge graphs for LLM memory with conflict detection and decay scoring"
- **r/selfhosted** — "Self-hosted AI memory engine — single Docker Compose, no Neo4j or Qdrant needed"

### Day 3 — Dev.to + Hashnode

Publish: *"Why We Rewrote Our AI Memory Engine in Rust (and Cut Latency by 60x)"* — a technical deep-dive that subtly positions Knol against Python-based alternatives.

### Day 4 — Twitter/X Thread

Thread format:
1. Hook: "We just open-sourced a Rust memory engine for AI agents. Here's what makes it different from Mem0 and Zep 🧵"
2. Architecture diagram (single image)
3. Benchmark comparison
4. 60-second deploy GIF
5. "Star us on GitHub" CTA

### Day 5 — Product Hunt

- Category: Developer Tools → AI
- Maker comment with the "why" story
- Tagline: "Persistent memory for AI agents — Rust-native, sub-5ms, self-hosted"

---

## Phase 3: Content Engine (Weeks 5–12)

### Weekly Blog Posts (rotating topics)

Publish on your blog, cross-post to Dev.to, Hashnode, and Medium.

**Technical Deep-Dives (developer audience):**
- "How Knol's 7-Layer Extraction Pipeline Cuts LLM Costs by 75%"
- "Building a Sliding-Window Rate Limiter with Redis Lua Scripts"
- "Bi-Temporal Memory: Why Your AI Agent Needs to Know When It Learned Something"
- "Row-Level Security in PostgreSQL: Multi-Tenant Isolation Without Multiple Databases"
- "From 300ms to 5ms: Why We Chose Rust Over Python for AI Infrastructure"

**Integration Tutorials (AI/ML teams):**
- "Add Persistent Memory to Your LangChain Agent in 5 Minutes"
- "Building a Context-Aware Customer Support Bot with Knol + CrewAI"
- "Using Knol as an MCP Server with Claude Desktop"
- "Knol + RAG: How Graph-Enhanced Retrieval Beats Pure Vector Search"

**Thought Leadership (decision makers):**
- "The Case Against Vector-Only Memory for Production AI"
- "Why AI Memory Infrastructure Will Be the Next $1B Category"
- "Self-Hosted vs Cloud AI Memory: A Total Cost Analysis"

### Twitter/X Strategy (daily, 10 min/day)

- **Monday:** Technical tip or code snippet
- **Tuesday:** Benchmark or comparison stat
- **Wednesday:** User showcase or testimonial
- **Thursday:** Architecture insight or design decision
- **Friday:** Community highlight or contributor shoutout
- **Weekend:** Engagement — reply to AI/LLM conversations, add value without pitching

### GitHub Activity (ongoing)

- Respond to every issue within 24 hours
- Merge community PRs quickly and thank contributors publicly
- Ship a release every 2 weeks with a changelog that tells a story
- Tag releases with fun names related to memory ("Total Recall", "Memento", etc.)

---

## Phase 4: Community & Ecosystem (Weeks 8–16)

### MCP Ecosystem Play

This is your highest-leverage zero-cost channel. MCP (Model Context Protocol) is exploding and Knol has a native MCP server.

- List Knol on the MCP server directory (modelcontextprotocol.io)
- Write a tutorial: "Give Claude Persistent Memory with Knol MCP Server"
- Engage in MCP Discord/forums — be the go-to memory solution
- Build example MCP workflows that showcase Knol's knowledge graph

### Developer Relations (earned, not paid)

- **Conference Talks:** Submit CFPs to RustConf, AI Engineer Summit, PyCon (for the Python SDK), local meetups. Most conferences have free speaker slots.
- **Podcast Circuit:** Pitch yourself to: Changelog, Latent Space, Practical AI, Rustacean Station. Story angle: "Why we built AI infrastructure in Rust"
- **YouTube:** Record 5-minute "build with Knol" videos. Screen recording is free. Post to YouTube + Twitter.

### Strategic Partnerships (mutual benefit, $0)

- **LangChain / LlamaIndex:** Contribute an official Knol integration to their repos. This puts you in their docs and ecosystem.
- **Ollama / LocalAI:** Position as the memory layer for local LLM setups. Write a joint blog post.
- **Supabase / Neon:** Both offer managed PostgreSQL. Write "Deploy Knol on Supabase/Neon in 5 Minutes" — they'll amplify it because it drives their usage.

---

## Phase 5: Conversion & Growth (Weeks 12–24)

### GitHub → Cloud Funnel

The funnel: GitHub star → self-host → hit limits → upgrade to cloud.

- Add a tasteful banner in the admin dashboard: "Running in self-hosted mode. Upgrade to Knol Cloud for managed hosting, 99.9% SLA, and zero ops."
- Include a `knol cloud` CLI command that migrates a self-hosted instance to managed cloud
- Weekly email digest (opt-in during self-hosted setup): usage stats, new features, cloud benefits

### SEO (Long-Term, Zero Cost)

Target these keywords with blog content:

- "AI memory layer" / "AI agent memory"
- "LLM context management"
- "persistent memory for AI"
- "Mem0 alternative" / "Zep alternative"
- "knowledge graph for LLM"
- "vector search PostgreSQL"
- "context engineering"

Each blog post targets 1–2 keywords. Internal linking between posts builds domain authority over time.

### Community-Led Growth

- **Knol Champions Program:** Identify your 10 most active community members. Give them early access to features, a Discord role, and a "Knol Champion" badge for their GitHub profile. They become your unpaid evangelists.
- **Showcase Page:** Maintain a `/showcase` page of projects built with Knol. People love being featured and will share the link.
- **Templates Gallery:** Pre-built Knol configurations for common use cases (customer support bot, tutoring agent, enterprise RAG). Each template is a landing page that ranks for its niche.

---

## Metrics & Tracking

| Metric | Week 4 | Week 12 | Week 24 |
|---|---|---|---|
| GitHub Stars | 1,000 | 3,000 | 5,000 |
| Discord Members | 100 | 400 | 1,000 |
| Docker Pulls | 500 | 3,000 | 10,000 |
| Blog Visits/mo | 2,000 | 10,000 | 25,000 |
| Cloud Signups | 20 | 150 | 500 |
| MRR | $0 | $3,000 | $10,000 |

Track with free tools: Plausible (self-hosted analytics), GitHub Insights, Discord member count, PostHog (self-hosted product analytics on the free tier).

---

## Weekly Time Investment

| Activity | Time/Week |
|---|---|
| Blog writing | 3 hours |
| Twitter/X | 1 hour |
| Community (Discord, GitHub issues) | 2 hours |
| Reddit/HN engagement | 1 hour |
| Content cross-posting | 30 min |
| **Total** | **~7.5 hours** |

---

## What NOT to Do

- **Don't** buy Twitter followers or GitHub stars. It destroys credibility.
- **Don't** spam subreddits. One authentic post per sub. Build reputation over time.
- **Don't** use AI-generated generic content. Your audience is developers — they can smell it.
- **Don't** trash competitors. Position Knol as a different category (infrastructure vs tool), not as "better Mem0."
- **Don't** gate the docs behind signup. Open docs = SEO = organic discovery.
- **Don't** wait for the product to be "perfect." Ship, get feedback, iterate publicly.

---

## The 30-Second Pitch

> Knol is the Nginx of AI memory. While others build Python SDKs on top of three databases, we built a single Rust binary that handles vector search, knowledge graphs, and temporal memory on just PostgreSQL. Sub-5ms latency. 50MB footprint. Deploy in 60 seconds. Apache 2.0.

Use this everywhere: HN comments, Twitter bio, conference intros, cold DMs.
