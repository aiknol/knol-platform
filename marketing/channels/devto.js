// =============================================================================
// Knol Marketing — Dev.to Channel Adapter
// Publishes articles via Dev.to API (free, 30 articles/day)
// =============================================================================

const https = require('https');

const CONFIG = {
  API_BASE: 'https://dev.to/api',
  RATE_LIMIT: { perDay: 30 },
};

async function publishArticle(article, credentials) {
  const { apiKey } = credentials;

  if (!apiKey) {
    return { success: false, error: 'Missing Dev.to API key', manual: true, content: article };
  }

  const body = JSON.stringify({
    article: {
      title: article.title,
      body_markdown: article.body,
      published: article.published !== false,
      tags: (article.tags || ['rust', 'ai', 'opensource', 'webdev']).slice(0, 4),
      series: article.series || null,
      canonical_url: article.canonicalUrl || null,
    },
  });

  return new Promise((resolve) => {
    const req = https.request({
      hostname: 'dev.to',
      path: '/api/articles',
      method: 'POST',
      headers: {
        'api-key': apiKey,
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          if (res.statusCode === 201) {
            resolve({
              success: true,
              url: parsed.url,
              articleId: parsed.id,
              slug: parsed.slug,
              statusCode: res.statusCode,
              headers: res.headers,
            });
          } else {
            resolve({
              success: false,
              error: parsed.error || `HTTP ${res.statusCode}`,
              rateLimited: res.statusCode === 429,
              statusCode: res.statusCode,
              headers: res.headers,
            });
          }
        } catch (e) {
          resolve({ success: false, error: `Parse error: ${e.message}`, statusCode: res.statusCode, headers: res.headers });
        }
      });
    });

    req.on('error', (e) => resolve({ success: false, error: e.message }));
    req.write(body);
    req.end();
  });
}

// Get existing articles (to avoid duplicates)
async function listArticles(credentials, page = 1) {
  const { apiKey } = credentials;
  if (!apiKey) return { success: false, articles: [] };

  return new Promise((resolve) => {
    const req = https.request({
      hostname: 'dev.to',
      path: `/api/articles/me?page=${page}&per_page=30`,
      method: 'GET',
      headers: { 'api-key': apiKey },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const articles = JSON.parse(data);
          resolve({ success: true, articles });
        } catch (e) {
          resolve({ success: false, articles: [] });
        }
      });
    });

    req.on('error', () => resolve({ success: false, articles: [] }));
    req.end();
  });
}

module.exports = { publishArticle, listArticles, CONFIG };
