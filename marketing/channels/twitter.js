// =============================================================================
// Knol Marketing — Twitter/X Channel Adapter
// Posts tweets via Twitter API v2 (Free tier: 1,500 tweets/mo, 50 per 24h)
// =============================================================================

const https = require('https');
const crypto = require('crypto');

const CONFIG = {
  API_BASE: 'https://api.twitter.com/2',
  RATE_LIMIT: { perDay: 50, perMonth: 1500 },
};

// OAuth 1.0a signature generation (required for Twitter API v2 free tier)
function oauthSign(method, url, params, consumerKey, consumerSecret, tokenKey, tokenSecret) {
  const nonce = crypto.randomBytes(16).toString('hex');
  const timestamp = Math.floor(Date.now() / 1000).toString();

  const oauthParams = {
    oauth_consumer_key: consumerKey,
    oauth_nonce: nonce,
    oauth_signature_method: 'HMAC-SHA1',
    oauth_timestamp: timestamp,
    oauth_token: tokenKey,
    oauth_version: '1.0',
  };

  const allParams = { ...oauthParams, ...params };
  const paramString = Object.keys(allParams)
    .sort()
    .map(k => `${encodeURIComponent(k)}=${encodeURIComponent(allParams[k])}`)
    .join('&');

  const baseString = `${method.toUpperCase()}&${encodeURIComponent(url)}&${encodeURIComponent(paramString)}`;
  const signingKey = `${encodeURIComponent(consumerSecret)}&${encodeURIComponent(tokenSecret)}`;
  const signature = crypto.createHmac('sha1', signingKey).update(baseString).digest('base64');

  return {
    ...oauthParams,
    oauth_signature: signature,
  };
}

function buildAuthHeader(oauthData) {
  const parts = Object.keys(oauthData)
    .sort()
    .map(k => `${encodeURIComponent(k)}="${encodeURIComponent(oauthData[k])}"`)
    .join(', ');
  return `OAuth ${parts}`;
}

async function postTweet(text, credentials) {
  const { apiKey, apiSecret, accessToken, accessTokenSecret } = credentials;

  if (!apiKey || !apiSecret || !accessToken || !accessTokenSecret) {
    return { success: false, error: 'Missing Twitter credentials' };
  }

  const url = `${CONFIG.API_BASE}/tweets`;
  const body = JSON.stringify({ text });

  const oauthData = oauthSign('POST', url, {}, apiKey, apiSecret, accessToken, accessTokenSecret);
  const authHeader = buildAuthHeader(oauthData);

  return new Promise((resolve) => {
    const urlObj = new URL(url);
    const req = https.request({
      hostname: urlObj.hostname,
      path: urlObj.pathname,
      method: 'POST',
      headers: {
        'Authorization': authHeader,
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
              tweetId: parsed.data?.id,
              url: `https://twitter.com/i/status/${parsed.data?.id}`,
              statusCode: res.statusCode,
              headers: res.headers,
            });
          } else {
            resolve({
              success: false,
              error: parsed.detail || parsed.title || `HTTP ${res.statusCode}`,
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

// Post a thread (array of tweets, each replying to previous)
async function postThread(tweets, credentials) {
  const results = [];
  let replyToId = null;

  for (const text of tweets) {
    const body = replyToId
      ? { text, reply: { in_reply_to_tweet_id: replyToId } }
      : { text };

    const result = await postTweet(typeof body === 'string' ? body : body.text, credentials);
    results.push(result);

    if (!result.success) break;
    replyToId = result.tweetId;

    // Rate limit: 1 second between tweets in a thread
    await new Promise(r => setTimeout(r, 1000));
  }

  return results;
}

module.exports = { postTweet, postThread, CONFIG };
