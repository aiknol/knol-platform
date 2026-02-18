// =============================================================================
// Knol Marketing — Blog Auto-Publisher
// Generates markdown blog posts → pushes to GitHub Pages (free hosting)
// Also cross-posts to Dev.to for wider reach
// =============================================================================

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const CONFIG = {
  BLOG_DIR: path.join(__dirname, '..', 'blog'),
  POSTS_DIR: path.join(__dirname, '..', 'blog', '_posts'),
  SITE_URL: 'https://blog.aiknol.com',
  REPO: 'knol-dev/blog',
};

// Generate a blog post markdown file
function createPost(title, content, tags = [], author = 'Knol Team') {
  const date = new Date().toISOString().split('T')[0];
  const slug = title.toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
  const filename = `${date}-${slug}.md`;

  const frontMatter = [
    '---',
    `title: "${title}"`,
    `date: ${new Date().toISOString()}`,
    `author: ${author}`,
    `tags: [${tags.map(t => `"${t}"`).join(', ')}]`,
    `description: "${content.substring(0, 160).replace(/"/g, "'")}"`,
    `slug: ${slug}`,
    '---',
    '',
  ].join('\n');

  const fullContent = frontMatter + content;

  // Ensure directory exists
  if (!fs.existsSync(CONFIG.POSTS_DIR)) {
    fs.mkdirSync(CONFIG.POSTS_DIR, { recursive: true });
  }

  const filepath = path.join(CONFIG.POSTS_DIR, filename);
  fs.writeFileSync(filepath, fullContent);

  return {
    success: true,
    filename,
    filepath,
    url: `${CONFIG.SITE_URL}/${date.replace(/-/g, '/')}/${slug}`,
    slug,
  };
}

// Initialize Jekyll blog structure for GitHub Pages
function initBlog() {
  const dirs = [CONFIG.BLOG_DIR, CONFIG.POSTS_DIR];
  dirs.forEach(d => { if (!fs.existsSync(d)) fs.mkdirSync(d, { recursive: true }); });

  // _config.yml
  const config = `title: Knol Blog
description: Engineering insights on AI memory infrastructure
url: "${CONFIG.SITE_URL}"
baseurl: ""
theme: minima
markdown: kramdown
plugins:
  - jekyll-feed
  - jekyll-seo-tag
  - jekyll-sitemap

author:
  name: Knol Team
  url: https://aiknol.com

social:
  name: Knol
  links:
    - https://github.com/aiknol/knol
    - https://twitter.com/knoldev

google_analytics:

defaults:
  - scope:
      path: ""
      type: "posts"
    values:
      layout: "post"
      author: "Knol Team"
`;

  // index.md
  const index = `---
layout: home
title: Knol Blog
---

Engineering insights on building memory infrastructure for AI agents.
Open-source, Rust-powered, production-ready.
`;

  // about.md
  const about = `---
layout: page
title: About
permalink: /about/
---

Knol is an open-source long-term memory layer for AI agents and LLM applications.

Built in Rust with Axum, pgvector, NATS, and Redis, Knol provides persistent,
searchable memory with automatic knowledge graph extraction.

- **GitHub**: [github.com/aiknol/knol](https://github.com/aiknol/knol)
- **Website**: [aiknol.com](https://aiknol.com)
- **License**: Apache 2.0
`;

  // Gemfile
  const gemfile = `source "https://rubygems.org"
gem "github-pages", group: :jekyll_plugins
gem "jekyll-feed", "~> 0.12"
gem "jekyll-seo-tag", "~> 2.8"
`;

  // Write files
  fs.writeFileSync(path.join(CONFIG.BLOG_DIR, '_config.yml'), config);
  fs.writeFileSync(path.join(CONFIG.BLOG_DIR, 'index.md'), index);
  fs.writeFileSync(path.join(CONFIG.BLOG_DIR, 'about.md'), about);
  fs.writeFileSync(path.join(CONFIG.BLOG_DIR, 'Gemfile'), gemfile);
  fs.writeFileSync(path.join(CONFIG.BLOG_DIR, '.gitignore'), '_site\n.sass-cache\n.jekyll-cache\n.jekyll-metadata\nvendor\n');

  return { success: true, path: CONFIG.BLOG_DIR };
}

// Push blog updates to GitHub (assumes git is configured)
function pushToGitHub() {
  try {
    execSync(`cd ${CONFIG.BLOG_DIR} && git add -A && git commit -m "New blog post" && git push`, {
      stdio: 'pipe',
    });
    return { success: true };
  } catch (e) {
    return { success: false, error: e.message, manual: true };
  }
}

// List all published posts
function listPosts() {
  if (!fs.existsSync(CONFIG.POSTS_DIR)) return [];
  return fs.readdirSync(CONFIG.POSTS_DIR)
    .filter(f => f.endsWith('.md'))
    .sort()
    .reverse()
    .map(f => {
      const content = fs.readFileSync(path.join(CONFIG.POSTS_DIR, f), 'utf8');
      const titleMatch = content.match(/^title:\s*"(.+)"$/m);
      const dateMatch = content.match(/^date:\s*(.+)$/m);
      return {
        filename: f,
        title: titleMatch?.[1] || f,
        date: dateMatch?.[1] || f.substring(0, 10),
      };
    });
}

// Generate SEO-optimized blog content topics
function generateTopicIdeas() {
  return [
    { title: 'Why Your AI Agent Needs Long-Term Memory', tags: ['ai', 'llm', 'memory'], priority: 'high' },
    { title: 'Building a Knowledge Graph from Chat History with Rust', tags: ['rust', 'knowledge-graph', 'tutorial'], priority: 'high' },
    { title: 'pgvector vs Pinecone vs Weaviate: Self-Hosted Vector Search Benchmarks', tags: ['benchmarks', 'vector-search', 'comparison'], priority: 'high' },
    { title: 'Adaptive Retrieval: Beyond Simple RAG', tags: ['rag', 'retrieval', 'ai'], priority: 'medium' },
    { title: 'How We Built a Multi-Tenant Memory Service in 3,000 Lines of Rust', tags: ['rust', 'architecture', 'open-source'], priority: 'medium' },
    { title: 'Entity Extraction Without Fine-Tuning: Using Claude for Knowledge Graphs', tags: ['claude', 'entity-extraction', 'llm'], priority: 'medium' },
    { title: 'Self-Hosting AI Infrastructure on a $8/month VPS', tags: ['devops', 'self-hosted', 'cost'], priority: 'high' },
    { title: 'NATS JetStream for Event-Driven AI Pipelines', tags: ['nats', 'architecture', 'rust'], priority: 'low' },
    { title: 'Memory Decay: How AI Agents Should Forget', tags: ['ai', 'research', 'memory'], priority: 'medium' },
    { title: 'From Zero to Production: Deploying Knol with Docker Compose', tags: ['tutorial', 'docker', 'deployment'], priority: 'high' },
  ];
}

module.exports = { createPost, initBlog, pushToGitHub, listPosts, generateTopicIdeas, CONFIG };
