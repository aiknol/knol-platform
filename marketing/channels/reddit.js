// =============================================================================
// Knol Marketing — Reddit Channel Adapter
// Posts to subreddits via Reddit API (free tier: 60 req/min, 100 posts/day)
// =============================================================================

const https = require('https');
const querystring = require('querystring');

const CONFIG = {
  API_BASE: 'https://oauth.reddit.com',
  AUTH_URL: 'https://www.reddit.com/api/v1/access_token',
  RATE_LIMIT: { perDay: 10, perMinute: 60 },
  TARGET_SUBREDDITS: [
    { name: 'rust', type: 'technical', minKarma: 10 },
    { name: 'programming', type: 'general', minKarma: 50 },
    { name: 'MachineLearning', type: 'ml', minKarma: 50 },
    { name: 'opensource', type: 'oss', minKarma: 10 },
    { name: 'selfhosted', type: 'infra', minKarma: 10 },
    { name: 'artificial', type: 'ai', minKarma: 10 },
    { name: 'LocalLLaMA', type: 'llm', minKarma: 10 },
  ],
};

// Get OAuth2 access token (script app type)
async function authenticate(credentials) {
  const { clientId, clientSecret, username, password } = credentials;

  if (!clientId || !clientSecret || !username || !password) {
    return { success: false, error: 'Missing Reddit credentials' };
  }

  const body = querystring.stringify({
    grant_type: 'password',
    username,
    password,
  });

  return new Promise((resolve) => {
    const auth = Buffer.from(`${clientId}:${clientSecret}`).toString('base64');
    const req = https.request({
      hostname: 'www.reddit.com',
      path: '/api/v1/access_token',
      method: 'POST',
      headers: {
        'Authorization': `Basic ${auth}`,
        'Content-Type': 'application/x-www-form-urlencoded',
        'User-Agent': 'knol-marketing/0.1.0',
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          if (parsed.access_token) {
            resolve({
              success: true,
              token: parsed.access_token,
              expiresIn: parsed.expires_in,
              statusCode: res.statusCode,
              headers: res.headers,
            });
          } else {
            resolve({
              success: false,
              error: parsed.error || 'Auth failed',
              rateLimited: res.statusCode === 429,
              statusCode: res.statusCode,
              headers: res.headers,
            });
          }
        } catch (e) {
          resolve({ success: false, error: `Parse error: ${e.message}` });
        }
      });
    });

    req.on('error', (e) => resolve({ success: false, error: e.message }));
    req.write(body);
    req.end();
  });
}

// Submit a link or self post
async function submitPost(subreddit, title, content, token, kind = 'self') {
  const body = querystring.stringify({
    sr: subreddit,
    kind,
    title,
    ...(kind === 'self' ? { text: content } : { url: content }),
    resubmit: true,
    send_replies: true,
  });

  return new Promise((resolve) => {
    const req = https.request({
      hostname: 'oauth.reddit.com',
      path: '/api/submit',
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/x-www-form-urlencoded',
        'User-Agent': 'knol-marketing/0.1.0',
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          if (parsed.success || (parsed.json?.data?.url)) {
            resolve({
              success: true,
              url: parsed.json?.data?.url || parsed.url,
              postId: parsed.json?.data?.id,
              statusCode: res.statusCode,
              headers: res.headers,
            });
          } else {
            const errors = parsed.json?.errors || [];
            resolve({
              success: false,
              error: errors.length ? errors.map(e => e.join(': ')).join('; ') : `HTTP ${res.statusCode}`,
              rateLimited: res.statusCode === 429,
              statusCode: res.statusCode,
              headers: res.headers,
            });
          }
        } catch (e) {
          resolve({ success: false, error: `Parse error: ${e.message}` });
        }
      });
    });

    req.on('error', (e) => resolve({ success: false, error: e.message }));
    req.write(body);
    req.end();
  });
}

// Post a comment on an existing thread
async function postComment(thingId, text, token) {
  const body = querystring.stringify({
    thing_id: thingId,
    text,
  });

  return new Promise((resolve) => {
    const req = https.request({
      hostname: 'oauth.reddit.com',
      path: '/api/comment',
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/x-www-form-urlencoded',
        'User-Agent': 'knol-marketing/0.1.0',
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        resolve({
          success: res.statusCode === 200,
          statusCode: res.statusCode,
        });
      });
    });

    req.on('error', (e) => resolve({ success: false, error: e.message }));
    req.write(body);
    req.end();
  });
}

module.exports = { authenticate, submitPost, postComment, CONFIG };
