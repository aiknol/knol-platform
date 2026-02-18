// =============================================================================
// Knol Marketing — GitHub Presence Automation
// Manages releases, README badges, discussion engagement, star campaigns
// Uses GitHub API (free tier: 5,000 req/hr authenticated)
// =============================================================================

const https = require('https');

const CONFIG = {
  API_BASE: 'https://api.github.com',
  REPO: 'knol-dev/knol',
  RATE_LIMIT: { perHour: 5000 },
};

function ghRequest(method, path, token, body = null) {
  return new Promise((resolve) => {
    const options = {
      hostname: 'api.github.com',
      path,
      method,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/vnd.github+json',
        'User-Agent': 'knol-marketing/0.1.0',
        'X-GitHub-Api-Version': '2022-11-28',
      },
    };

    if (body) {
      const bodyStr = JSON.stringify(body);
      options.headers['Content-Type'] = 'application/json';
      options.headers['Content-Length'] = Buffer.byteLength(bodyStr);
    }

    const req = https.request(options, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          resolve({
            success: res.statusCode < 400,
            status: res.statusCode,
            statusCode: res.statusCode,
            rateLimited: res.statusCode === 429,
            headers: res.headers,
            data: JSON.parse(data || '{}'),
          });
        } catch {
          resolve({
            success: res.statusCode < 400,
            status: res.statusCode,
            statusCode: res.statusCode,
            rateLimited: res.statusCode === 429,
            headers: res.headers,
            data: {},
          });
        }
      });
    });

    req.on('error', (e) => resolve({ success: false, error: e.message }));
    if (body) req.write(JSON.stringify(body));
    req.end();
  });
}

// Create a GitHub release with auto-generated release notes
async function createRelease(tag, name, body, credentials) {
  const { token } = credentials;
  if (!token) return { success: false, error: 'Missing GitHub token' };

  return ghRequest('POST', `/repos/${CONFIG.REPO}/releases`, token, {
    tag_name: tag,
    name: name || `Knol ${tag}`,
    body,
    draft: false,
    prerelease: tag.includes('alpha') || tag.includes('beta'),
    generate_release_notes: true,
  });
}

// Create a discussion in the repo
async function createDiscussion(title, body, category, credentials) {
  // GitHub Discussions API requires GraphQL — using REST for issues instead
  const { token } = credentials;
  if (!token) return { success: false, error: 'Missing GitHub token' };

  return ghRequest('POST', `/repos/${CONFIG.REPO}/issues`, token, {
    title,
    body,
    labels: ['discussion', 'community'],
  });
}

// Update repo description and topics
async function updateRepoMetadata(description, topics, credentials) {
  const { token } = credentials;
  if (!token) return { success: false, error: 'Missing GitHub token' };

  const results = [];

  if (description) {
    results.push(await ghRequest('PATCH', `/repos/${CONFIG.REPO}`, token, {
      description,
      homepage: 'https://aiknol.com',
    }));
  }

  if (topics?.length) {
    results.push(await ghRequest('PUT', `/repos/${CONFIG.REPO}/topics`, token, {
      names: topics,
    }));
  }

  return results;
}

// Get repo stats for analytics
async function getRepoStats(credentials) {
  const { token } = credentials;
  if (!token) return { success: false, error: 'Missing GitHub token' };

  const [repo, traffic, clones] = await Promise.all([
    ghRequest('GET', `/repos/${CONFIG.REPO}`, token),
    ghRequest('GET', `/repos/${CONFIG.REPO}/traffic/views`, token),
    ghRequest('GET', `/repos/${CONFIG.REPO}/traffic/clones`, token),
  ]);

  return {
    success: true,
    stars: repo.data?.stargazers_count || 0,
    forks: repo.data?.forks_count || 0,
    watchers: repo.data?.subscribers_count || 0,
    openIssues: repo.data?.open_issues_count || 0,
    views14d: traffic.data?.count || 0,
    uniqueVisitors14d: traffic.data?.uniques || 0,
    clones14d: clones.data?.count || 0,
    uniqueCloners14d: clones.data?.uniques || 0,
  };
}

// Generate README badge markdown
function generateBadges() {
  const repo = CONFIG.REPO;
  return [
    `[![GitHub stars](https://img.shields.io/github/stars/${repo}?style=flat-square)](https://github.com/${repo}/stargazers)`,
    `[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE)`,
    `[![Rust](https://img.shields.io/badge/rust-1.75+-orange?style=flat-square)](https://www.rust-lang.org/)`,
    `[![Docker](https://img.shields.io/badge/docker-ready-blue?style=flat-square)](https://github.com/${repo}/pkgs/container/knol-oss)`,
    `[![CI](https://img.shields.io/github/actions/workflow/status/${repo}/ci.yml?style=flat-square)](https://github.com/${repo}/actions)`,
  ].join('\n');
}

// Find trending repos to cross-promote with
async function findRelatedRepos(credentials) {
  const { token } = credentials;
  const queries = ['memory+LLM+language:rust', 'vector+database+language:rust', 'knowledge+graph+ai'];
  const repos = [];

  for (const q of queries) {
    const result = await ghRequest('GET', `/search/repositories?q=${encodeURIComponent(q)}&sort=stars&per_page=5`, token);
    if (result.success && result.data?.items) {
      repos.push(...result.data.items.map(r => ({
        name: r.full_name,
        stars: r.stargazers_count,
        description: r.description,
        url: r.html_url,
      })));
    }
    await new Promise(r => setTimeout(r, 500));
  }

  return repos;
}

module.exports = { createRelease, createDiscussion, updateRepoMetadata, getRepoStats, generateBadges, findRelatedRepos, CONFIG };
