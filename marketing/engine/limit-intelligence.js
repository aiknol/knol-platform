// =============================================================================
// Knol Marketing — Limit Intelligence + Policy Engine
// Stateful guardrails to avoid crossing channel limits and posting spam-like content
// =============================================================================

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const DATA_DIR = path.join(__dirname, '..', 'data');
const STATE_FILE = path.join(DATA_DIR, 'rate-limit-state.json');
const POLICY_FILE = path.join(__dirname, '..', 'policies', 'channel-policies.json');

function ensureDataDir() {
  if (!fs.existsSync(DATA_DIR)) fs.mkdirSync(DATA_DIR, { recursive: true });
}

function loadPolicies() {
  try {
    return JSON.parse(fs.readFileSync(POLICY_FILE, 'utf8'));
  } catch {
    return { default: { safetyFactor: 0.8, duplicateWindowHours: 168 } };
  }
}

function loadState() {
  ensureDataDir();
  try {
    return JSON.parse(fs.readFileSync(STATE_FILE, 'utf8'));
  } catch {
    return {
      channels: {},
      history: [],
    };
  }
}

function saveState(state) {
  ensureDataDir();
  fs.writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
}

function getChannelState(state, channel) {
  if (!state.channels[channel]) {
    state.channels[channel] = {
      day: {},      // key: YYYY-MM-DD
      month: {},    // key: YYYY-MM
      minute: {},   // key: YYYY-MM-DDTHH:MM
      hour: {},     // key: YYYY-MM-DDTHH
      lastPostAt: null,
      cooldownUntil: null,
      telemetry: {}, // endpoint-level latest API telemetry
    };
  }
  return state.channels[channel];
}

function dayKey(date) {
  return date.toISOString().slice(0, 10);
}

function monthKey(date) {
  return date.toISOString().slice(0, 7);
}

function minuteKey(date) {
  return date.toISOString().slice(0, 16);
}

function hourKey(date) {
  return date.toISOString().slice(0, 13);
}

function safeLimit(limit, safetyFactor) {
  if (!limit || Number.isNaN(limit)) return null;
  return Math.max(1, Math.floor(limit * safetyFactor));
}

function contentHash(content) {
  const canonical = typeof content === 'string' ? content : JSON.stringify(content || {});
  return crypto.createHash('sha256').update(canonical).digest('hex');
}

function checkQuietHours(channelPolicy, nowUtcHour) {
  if (!Array.isArray(channelPolicy.quietHoursUtc) || channelPolicy.quietHoursUtc.length !== 2) return false;
  const [start, end] = channelPolicy.quietHoursUtc;
  if (start <= end) return nowUtcHour >= start && nowUtcHour < end;
  return nowUtcHour >= start || nowUtcHour < end;
}

function checkDuplicate(state, channel, hash, duplicateWindowHours, now) {
  const cutoff = now.getTime() - duplicateWindowHours * 60 * 60 * 1000;
  return state.history.some((item) =>
    item.channel === channel &&
    item.hash === hash &&
    new Date(item.timestamp).getTime() >= cutoff
  );
}

function pruneState(state, now) {
  const cutoffHistory = now.getTime() - 180 * 24 * 60 * 60 * 1000;
  state.history = (state.history || []).filter((item) => new Date(item.timestamp).getTime() >= cutoffHistory);

  for (const channelState of Object.values(state.channels || {})) {
    for (const [k, v] of Object.entries(channelState.minute || {})) {
      const ts = new Date(`${k}:00Z`).getTime();
      if (now.getTime() - ts > 120 * 60 * 1000 || v <= 0) delete channelState.minute[k];
    }
    for (const [k, v] of Object.entries(channelState.hour || {})) {
      const ts = new Date(`${k}:00:00Z`).getTime();
      if (now.getTime() - ts > 7 * 24 * 60 * 60 * 1000 || v <= 0) delete channelState.hour[k];
    }
    for (const [k, v] of Object.entries(channelState.day || {})) {
      const ts = new Date(`${k}T00:00:00Z`).getTime();
      if (now.getTime() - ts > 180 * 24 * 60 * 60 * 1000 || v <= 0) delete channelState.day[k];
    }
    for (const [k, v] of Object.entries(channelState.month || {})) {
      const ts = new Date(`${k}-01T00:00:00Z`).getTime();
      if (now.getTime() - ts > 365 * 24 * 60 * 60 * 1000 || v <= 0) delete channelState.month[k];
    }
  }
}

function toWaitSeconds(untilIso, now) {
  if (!untilIso) return 0;
  const ms = new Date(untilIso).getTime() - now.getTime();
  return ms > 0 ? Math.ceil(ms / 1000) : 0;
}

function shouldAllowPublish(channel, endpoint, content) {
  const now = new Date();
  const state = loadState();
  const policies = loadPolicies();
  const defaults = policies.default || {};
  const channelPolicy = policies[channel] || {};
  const channelState = getChannelState(state, channel);
  const safetyFactor = defaults.safetyFactor || 0.8;
  const duplicateWindowHours = defaults.duplicateWindowHours || 168;
  const hash = contentHash(content);

  pruneState(state, now);

  if (channelState.cooldownUntil && new Date(channelState.cooldownUntil) > now) {
    return {
      allowed: false,
      reason: 'cooldown_active',
      waitSeconds: toWaitSeconds(channelState.cooldownUntil, now),
      nextAllowedAt: channelState.cooldownUntil,
    };
  }

  if (checkQuietHours(channelPolicy, now.getUTCHours())) {
    const next = new Date(now.getTime());
    next.setUTCHours(channelPolicy.quietHoursUtc[1], 0, 0, 0);
    if (next <= now) next.setUTCDate(next.getUTCDate() + 1);
    return {
      allowed: false,
      reason: 'quiet_hours',
      waitSeconds: Math.ceil((next.getTime() - now.getTime()) / 1000),
      nextAllowedAt: next.toISOString(),
    };
  }

  if (checkDuplicate(state, channel, hash, duplicateWindowHours, now)) {
    return {
      allowed: false,
      reason: 'duplicate_content_window',
      waitSeconds: 0,
      nextAllowedAt: null,
    };
  }

  if (channelPolicy.minSpacingSeconds && channelState.lastPostAt) {
    const elapsed = (now.getTime() - new Date(channelState.lastPostAt).getTime()) / 1000;
    if (elapsed < channelPolicy.minSpacingSeconds) {
      const wait = Math.ceil(channelPolicy.minSpacingSeconds - elapsed);
      const next = new Date(now.getTime() + wait * 1000);
      return {
        allowed: false,
        reason: 'min_spacing',
        waitSeconds: wait,
        nextAllowedAt: next.toISOString(),
      };
    }
  }

  const dKey = dayKey(now);
  const mKey = monthKey(now);
  const minKey = minuteKey(now);
  const hKey = hourKey(now);

  const maxPerDay = safeLimit(channelPolicy.maxPerDay, safetyFactor);
  const maxPerMonth = safeLimit(channelPolicy.maxPerMonth, safetyFactor);
  const maxPerMinute = safeLimit(channelPolicy.maxPerMinute, safetyFactor);
  const maxPerHour = safeLimit(channelPolicy.maxPerHour, safetyFactor);

  if (maxPerDay && (channelState.day[dKey] || 0) >= maxPerDay) {
    const next = new Date(`${dKey}T00:00:00Z`);
    next.setUTCDate(next.getUTCDate() + 1);
    return { allowed: false, reason: 'daily_limit', waitSeconds: Math.ceil((next.getTime() - now.getTime()) / 1000), nextAllowedAt: next.toISOString() };
  }
  if (maxPerMonth && (channelState.month[mKey] || 0) >= maxPerMonth) {
    const next = new Date(`${mKey}-01T00:00:00Z`);
    next.setUTCMonth(next.getUTCMonth() + 1);
    return { allowed: false, reason: 'monthly_limit', waitSeconds: Math.ceil((next.getTime() - now.getTime()) / 1000), nextAllowedAt: next.toISOString() };
  }
  if (maxPerMinute && (channelState.minute[minKey] || 0) >= maxPerMinute) {
    const next = new Date(`${minKey}:00Z`);
    next.setUTCMinutes(next.getUTCMinutes() + 1);
    return { allowed: false, reason: 'minute_limit', waitSeconds: Math.ceil((next.getTime() - now.getTime()) / 1000), nextAllowedAt: next.toISOString() };
  }
  if (maxPerHour && (channelState.hour[hKey] || 0) >= maxPerHour) {
    const next = new Date(`${hKey}:00:00Z`);
    next.setUTCHours(next.getUTCHours() + 1);
    return { allowed: false, reason: 'hour_limit', waitSeconds: Math.ceil((next.getTime() - now.getTime()) / 1000), nextAllowedAt: next.toISOString() };
  }

  return { allowed: true, reason: 'ok', hash, endpoint };
}

function reservePublish(channel, hash) {
  const now = new Date();
  const state = loadState();
  const channelState = getChannelState(state, channel);
  const dKey = dayKey(now);
  const mKey = monthKey(now);
  const minKey = minuteKey(now);
  const hKey = hourKey(now);

  channelState.day[dKey] = (channelState.day[dKey] || 0) + 1;
  channelState.month[mKey] = (channelState.month[mKey] || 0) + 1;
  channelState.minute[minKey] = (channelState.minute[minKey] || 0) + 1;
  channelState.hour[hKey] = (channelState.hour[hKey] || 0) + 1;
  channelState.lastPostAt = now.toISOString();

  state.history.push({
    channel,
    hash,
    timestamp: now.toISOString(),
  });

  pruneState(state, now);
  saveState(state);
}

function parseIntSafe(v) {
  const n = parseInt(String(v || ''), 10);
  return Number.isFinite(n) ? n : null;
}

function updateFromHeaders(channel, endpoint, headers) {
  const now = new Date();
  const state = loadState();
  const channelState = getChannelState(state, channel);
  const h = {};
  for (const [k, v] of Object.entries(headers || {})) h[String(k).toLowerCase()] = v;

  const xLimit = parseIntSafe(h['x-rate-limit-limit'] || h['x-ratelimit-limit']);
  const xRemaining = parseIntSafe(h['x-rate-limit-remaining'] || h['x-ratelimit-remaining']);
  const xReset = parseIntSafe(h['x-rate-limit-reset'] || h['x-ratelimit-reset']);
  const retryAfter = parseIntSafe(h['retry-after']);

  channelState.telemetry[endpoint || 'publish'] = {
    updatedAt: now.toISOString(),
    limit: xLimit,
    remaining: xRemaining,
    resetEpochSeconds: xReset,
    retryAfterSeconds: retryAfter,
  };

  if (retryAfter && retryAfter > 0) {
    channelState.cooldownUntil = new Date(now.getTime() + retryAfter * 1000).toISOString();
  } else if (xReset && xReset > Math.floor(now.getTime() / 1000) && xRemaining === 0) {
    channelState.cooldownUntil = new Date(xReset * 1000).toISOString();
  }

  saveState(state);
}

function markRateLimited(channel, endpoint, details = {}) {
  const now = new Date();
  const state = loadState();
  const channelState = getChannelState(state, channel);
  const retryAfter = parseIntSafe(details.retryAfterSeconds || details.retryAfter);
  const resetEpoch = parseIntSafe(details.resetEpochSeconds || details.resetEpoch);
  let until = null;

  if (retryAfter && retryAfter > 0) {
    until = new Date(now.getTime() + retryAfter * 1000);
  } else if (resetEpoch && resetEpoch > Math.floor(now.getTime() / 1000)) {
    until = new Date(resetEpoch * 1000);
  } else {
    // conservative fallback
    until = new Date(now.getTime() + 15 * 60 * 1000);
  }

  channelState.cooldownUntil = until.toISOString();
  channelState.telemetry[endpoint || 'publish'] = {
    updatedAt: now.toISOString(),
    rateLimited: true,
    retryAfterSeconds: retryAfter,
    resetEpochSeconds: resetEpoch,
  };
  saveState(state);
}

function getStateSnapshot() {
  return loadState();
}

module.exports = {
  shouldAllowPublish,
  reservePublish,
  updateFromHeaders,
  markRateLimited,
  getStateSnapshot,
};
