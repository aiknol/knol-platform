// =============================================================================
// Knol Marketing — Hacker News Channel Adapter
// Submits to HN via Firebase API + generates Show HN / Ask HN posts
// NOTE: HN has no official post API — this generates content for manual
// submission and monitors HN for relevant discussions to engage with.
// =============================================================================

const https = require('https');

const CONFIG = {
  API_BASE: 'https://hacker-news.firebaseio.com/v0',
  SITE_URL: 'https://news.ycombinator.com',
  STRATEGY: 'manual_submit_with_monitoring',
};

// Search HN for relevant discussions (via Algolia API — free, unlimited)
async function searchDiscussions(query) {
  return new Promise((resolve) => {
    const encodedQuery = encodeURIComponent(query);
    const req = https.request({
      hostname: 'hn.algolia.com',
      path: `/api/v1/search?query=${encodedQuery}&tags=story&hitsPerPage=10`,
      method: 'GET',
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          resolve({
            success: true,
            stories: (parsed.hits || []).map(h => ({
              id: h.objectID,
              title: h.title,
              url: h.url,
              points: h.points,
              comments: h.num_comments,
              author: h.author,
              hnUrl: `${CONFIG.SITE_URL}/item?id=${h.objectID}`,
              createdAt: h.created_at,
            })),
          });
        } catch (e) {
          resolve({ success: false, stories: [] });
        }
      });
    });

    req.on('error', () => resolve({ success: false, stories: [] }));
    req.end();
  });
}

// Search for discussions where Knol could be mentioned
async function findEngagementOpportunities() {
  const queries = [
    'memory layer LLM',
    'long term memory AI agent',
    'vector database Rust',
    'context window LLM memory',
    'AI memory management',
    'pgvector Rust',
    'knowledge graph LLM',
  ];

  const results = [];
  for (const q of queries) {
    const { stories } = await searchDiscussions(q);
    results.push(...stories.filter(s => s.points > 5 && s.comments > 2));
    await new Promise(r => setTimeout(r, 200)); // rate limit
  }

  // Deduplicate by id
  const seen = new Set();
  return results.filter(s => {
    if (seen.has(s.id)) return false;
    seen.add(s.id);
    return true;
  }).sort((a, b) => b.points - a.points);
}

// Generate a "Show HN" post (for manual submission)
function generateShowHN() {
  return {
    title: 'Show HN: Knol – Open-source long-term memory layer for AI agents (Rust)',
    url: 'https://github.com/aiknol/knol',
    text: null, // link post, not text post
    submitUrl: `${CONFIG.SITE_URL}/submitlink?u=${encodeURIComponent('https://github.com/aiknol/knol')}&t=${encodeURIComponent('Show HN: Knol – Open-source long-term memory layer for AI agents (Rust)')}`,
  };
}

// Generate engagement comment for a relevant discussion
function generateEngagementComment(story) {
  const templates = [
    `Interesting discussion! We built Knol (https://github.com/aiknol/knol) to address exactly this — a persistent memory layer for LLM agents using pgvector + knowledge graphs. It's open-source (Rust/Axum) and self-hostable. Would love feedback from this community.`,
    `This is a problem we've been tackling with Knol — an open-source Rust service that gives AI agents persistent, searchable memory via pgvector and automatic knowledge graph extraction. Happy to share what we've learned.`,
    `Related: we open-sourced Knol (Rust + pgvector + NATS), which implements long-term memory for AI agents with adaptive retrieval and entity graph extraction. Self-hostable, MIT-friendly. Might be useful for folks in this thread.`,
  ];

  return {
    storyId: story.id,
    storyTitle: story.title,
    hnUrl: story.hnUrl,
    suggestedComment: templates[Math.floor(Math.random() * templates.length)],
    manual: true, // always manual — HN detects automated comments
  };
}

module.exports = { searchDiscussions, findEngagementOpportunities, generateShowHN, generateEngagementComment, CONFIG };
