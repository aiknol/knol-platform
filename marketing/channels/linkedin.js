// =============================================================================
// Knol Marketing — LinkedIn Channel Adapter
// Posts articles/updates via LinkedIn API (free tier w/ OAuth app)
// Fallback: generates ready-to-post content for manual sharing
// =============================================================================

const https = require('https');

const CONFIG = {
  API_BASE: 'https://api.linkedin.com/v2',
  RATE_LIMIT: { perDay: 25 },
};

// Post a text update to LinkedIn personal profile
async function postUpdate(text, credentials) {
  const { accessToken, personUrn } = credentials;

  if (!accessToken || !personUrn) {
    return {
      success: false,
      error: 'Missing LinkedIn credentials — saving content for manual posting',
      content: text,
      manual: true,
    };
  }

  const body = JSON.stringify({
    author: personUrn,
    lifecycleState: 'PUBLISHED',
    specificContent: {
      'com.linkedin.ugc.ShareContent': {
        shareCommentary: { text },
        shareMediaCategory: 'NONE',
      },
    },
    visibility: {
      'com.linkedin.ugc.MemberNetworkVisibility': 'PUBLIC',
    },
  });

  return new Promise((resolve) => {
    const req = https.request({
      hostname: 'api.linkedin.com',
      path: '/v2/ugcPosts',
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
        'X-Restli-Protocol-Version': '2.0.0',
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        if (res.statusCode === 201) {
          resolve({ success: true, postId: res.headers['x-restli-id'], statusCode: res.statusCode, headers: res.headers });
        } else {
          resolve({
            success: false,
            error: `HTTP ${res.statusCode}: ${data.substring(0, 200)}`,
            content: text,
            manual: true,
            rateLimited: res.statusCode === 429,
            statusCode: res.statusCode,
            headers: res.headers,
          });
        }
      });
    });

    req.on('error', (e) => resolve({ success: false, error: e.message, content: text, manual: true }));
    req.write(body);
    req.end();
  });
}

// Post an article with link preview
async function postArticle(title, text, url, credentials) {
  const fullText = `${text}\n\n${url}`;
  return postUpdate(fullText, credentials);
}

// Generate content formatted for LinkedIn (when no API access)
function formatForManualPost(content) {
  return {
    text: content,
    hashtags: '#opensource #memorylayer #rustlang #ai #llm #devtools',
    bestTimes: ['Tuesday 10am', 'Wednesday 12pm', 'Thursday 9am'],
  };
}

module.exports = { postUpdate, postArticle, formatForManualPost, CONFIG };
