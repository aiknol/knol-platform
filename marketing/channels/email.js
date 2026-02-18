// =============================================================================
// Knol Marketing — Email Newsletter Channel
// Sends newsletters via SMTP (Gmail free: 500/day, custom SMTP: unlimited)
// Self-managed subscriber list (JSON file — no Mailchimp needed)
// =============================================================================

let nodemailerAvailable = false;
try { require('nodemailer'); nodemailerAvailable = true; } catch {}

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const CONFIG = {
  SUBSCRIBERS_FILE: path.join(__dirname, '..', 'data', 'subscribers.json'),
  SENT_LOG_FILE: path.join(__dirname, '..', 'data', 'email-log.json'),
  RATE_LIMIT: { perDay: 450 }, // Stay under Gmail's 500 limit
  FROM_NAME: 'Knol',
  UNSUBSCRIBE_URL: 'https://aiknol.com/unsubscribe',
};

// Load subscribers from JSON file
function loadSubscribers() {
  try {
    const data = fs.readFileSync(CONFIG.SUBSCRIBERS_FILE, 'utf8');
    return JSON.parse(data);
  } catch {
    return [];
  }
}

// Save subscribers
function saveSubscribers(subscribers) {
  const dir = path.dirname(CONFIG.SUBSCRIBERS_FILE);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(CONFIG.SUBSCRIBERS_FILE, JSON.stringify(subscribers, null, 2));
}

// Add a subscriber
function addSubscriber(email, name = '', source = 'manual') {
  const subscribers = loadSubscribers();
  const existing = subscribers.find(s => s.email === email);
  if (existing) return { success: false, error: 'Already subscribed' };

  subscribers.push({
    email,
    name,
    source,
    subscribedAt: new Date().toISOString(),
    confirmed: false,
    unsubscribed: false,
    token: crypto.randomBytes(16).toString('hex'),
  });
  saveSubscribers(subscribers);
  return { success: true };
}

// Create SMTP transport
function createSmtpTransport(credentials) {
  const { smtpHost, smtpPort, smtpUser, smtpPass } = credentials;

  if (!smtpHost || !smtpUser || !smtpPass || !nodemailerAvailable) {
    return null;
  }

  try {
    const nodemailer = require('nodemailer');
    return nodemailer.createTransport({
      host: smtpHost,
      port: parseInt(smtpPort) || 587,
      secure: parseInt(smtpPort) === 465,
      auth: { user: smtpUser, pass: smtpPass },
    });
  } catch {
    return null;
  }
}

// Send a single email
async function sendEmail(to, subject, htmlBody, textBody, credentials) {
  const transport = createSmtpTransport(credentials);

  if (!transport) {
    return {
      success: false,
      error: 'SMTP not configured — install nodemailer and set credentials',
      content: { to, subject, html: htmlBody, text: textBody },
      manual: true,
    };
  }

  try {
    const info = await transport.sendMail({
      from: `"${CONFIG.FROM_NAME}" <${credentials.smtpUser}>`,
      to,
      subject,
      text: textBody,
      html: htmlBody,
      headers: {
        'List-Unsubscribe': `<${CONFIG.UNSUBSCRIBE_URL}>`,
        'X-Mailer': 'knol-marketing/0.1.0',
      },
    });

    return { success: true, messageId: info.messageId };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Send newsletter to all active subscribers
async function sendNewsletter(subject, htmlBody, textBody, credentials) {
  const subscribers = loadSubscribers().filter(s => !s.unsubscribed && s.confirmed);
  const results = { sent: 0, failed: 0, errors: [] };

  for (const sub of subscribers) {
    // Personalize
    const personalizedHtml = htmlBody
      .replace(/\{\{name\}\}/g, sub.name || 'there')
      .replace(/\{\{unsubscribe_url\}\}/g, `${CONFIG.UNSUBSCRIBE_URL}?token=${sub.token}`);

    const personalizedText = textBody
      .replace(/\{\{name\}\}/g, sub.name || 'there')
      .replace(/\{\{unsubscribe_url\}\}/g, `${CONFIG.UNSUBSCRIBE_URL}?token=${sub.token}`);

    const result = await sendEmail(sub.email, subject, personalizedHtml, personalizedText, credentials);

    if (result.success) {
      results.sent++;
    } else {
      results.failed++;
      results.errors.push({ email: sub.email, error: result.error });
    }

    // Rate limit: 100ms between emails
    await new Promise(r => setTimeout(r, 100));

    // Stop if we hit daily limit
    if (results.sent >= CONFIG.RATE_LIMIT.perDay) break;
  }

  // Log
  logSend(subject, results);
  return results;
}

// Generate newsletter HTML
function generateNewsletterHtml(content) {
  return `<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"></head>
<body style="margin:0;padding:0;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#f8f9fa">
  <div style="max-width:600px;margin:0 auto;padding:20px">
    <div style="text-align:center;padding:20px 0;border-bottom:2px solid #6E56CF">
      <h1 style="color:#6E56CF;margin:0;font-size:24px">Knol</h1>
      <p style="color:#666;margin:4px 0 0;font-size:14px">Memory Infrastructure for AI</p>
    </div>
    <div style="padding:24px 0;color:#333;line-height:1.6">
      <p>Hi {{name}},</p>
      ${content}
    </div>
    <div style="border-top:1px solid #eee;padding:16px 0;text-align:center;color:#999;font-size:12px">
      <p>Knol — Open-source long-term memory for AI agents</p>
      <p><a href="https://github.com/aiknol/knol" style="color:#6E56CF">GitHub</a> ·
         <a href="https://aiknol.com" style="color:#6E56CF">Website</a> ·
         <a href="{{unsubscribe_url}}" style="color:#999">Unsubscribe</a></p>
    </div>
  </div>
</body>
</html>`;
}

function logSend(subject, results) {
  const dir = path.dirname(CONFIG.SENT_LOG_FILE);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });

  let log = [];
  try { log = JSON.parse(fs.readFileSync(CONFIG.SENT_LOG_FILE, 'utf8')); } catch {}
  log.push({ date: new Date().toISOString(), subject, ...results });
  fs.writeFileSync(CONFIG.SENT_LOG_FILE, JSON.stringify(log, null, 2));
}

module.exports = {
  addSubscriber, loadSubscribers, saveSubscribers,
  sendEmail, sendNewsletter,
  generateNewsletterHtml,
  CONFIG,
};
