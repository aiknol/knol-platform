// =============================================================================
// Knol Marketing — Initial Setup
// Validates credentials, initializes blog, creates data directories
// =============================================================================

const fs = require('fs');
const path = require('path');
const blog = require('../channels/blog');

const DIRS = [
  path.join(__dirname, '..', 'data'),
  path.join(__dirname, '..', 'reports'),
  path.join(__dirname, '..', 'blog', '_posts'),
];

function checkEnvVar(name, required = false) {
  const value = process.env[name];
  if (value) {
    console.log(`  ✓ ${name} = ${value.substring(0, 4)}${'*'.repeat(Math.max(0, value.length - 4))}`);
    return true;
  } else {
    console.log(`  ${required ? '✗' : '○'} ${name} — ${required ? 'MISSING (required)' : 'not set (optional)'}`);
    return false;
  }
}

async function main() {
  console.log('Knol Marketing — Setup\n');

  // 1. Create directories
  console.log('1. Creating directories...');
  for (const dir of DIRS) {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
      console.log(`   Created: ${dir}`);
    } else {
      console.log(`   Exists: ${dir}`);
    }
  }

  // 2. Initialize blog
  console.log('\n2. Initializing blog...');
  const blogResult = blog.initBlog();
  console.log(`   Blog initialized at: ${blogResult.path}`);

  // 3. Check credentials
  console.log('\n3. Checking credentials...\n');

  console.log('  Twitter (for daily tweets):');
  checkEnvVar('TWITTER_API_KEY');
  checkEnvVar('TWITTER_API_SECRET');
  checkEnvVar('TWITTER_ACCESS_TOKEN');
  checkEnvVar('TWITTER_ACCESS_TOKEN_SECRET');

  console.log('\n  Reddit (for weekly posts):');
  checkEnvVar('REDDIT_CLIENT_ID');
  checkEnvVar('REDDIT_CLIENT_SECRET');
  checkEnvVar('REDDIT_USERNAME');
  checkEnvVar('REDDIT_PASSWORD');

  console.log('\n  Dev.to (for articles):');
  checkEnvVar('DEVTO_API_KEY');

  console.log('\n  LinkedIn (optional — manual fallback):');
  checkEnvVar('LINKEDIN_ACCESS_TOKEN');
  checkEnvVar('LINKEDIN_PERSON_URN');

  console.log('\n  GitHub (for repo stats + releases):');
  checkEnvVar('GITHUB_TOKEN');

  console.log('\n  Email / SMTP (for newsletter):');
  checkEnvVar('SMTP_HOST');
  checkEnvVar('SMTP_PORT');
  checkEnvVar('SMTP_USER');
  checkEnvVar('SMTP_PASS');

  console.log('\n  Claude API (optional — enhances content):');
  checkEnvVar('ANTHROPIC_API_KEY');

  // 4. Initialize subscriber list if empty
  const subsFile = path.join(__dirname, '..', 'data', 'subscribers.json');
  if (!fs.existsSync(subsFile)) {
    fs.writeFileSync(subsFile, '[]');
    console.log('\n4. Created empty subscriber list');
  }

  // 5. Initialize state file
  const stateFile = path.join(__dirname, '..', 'data', 'scheduler-state.json');
  if (!fs.existsSync(stateFile)) {
    fs.writeFileSync(stateFile, JSON.stringify({ lastRun: {}, history: [] }, null, 2));
    console.log('5. Created scheduler state file');
  }

  // 6. Initialize rate-limit state
  const rateLimitFile = path.join(__dirname, '..', 'data', 'rate-limit-state.json');
  if (!fs.existsSync(rateLimitFile)) {
    fs.writeFileSync(rateLimitFile, JSON.stringify({ channels: {}, history: [] }, null, 2));
    console.log('6. Created rate-limit state file');
  }

  // 7. Initialize deferred queue
  const deferredFile = path.join(__dirname, '..', 'data', 'deferred-queue.json');
  if (!fs.existsSync(deferredFile)) {
    fs.writeFileSync(deferredFile, '[]');
    console.log('7. Created deferred queue file');
  }

  console.log('\n✓ Setup complete!');
  console.log('\nNext steps:');
  console.log('  1. Copy .env.example to .env and fill in your credentials');
  console.log('  2. Run: node engine/scheduler.js --run daily --dry-run');
  console.log('  3. Run: node engine/scheduler.js --run daily');
  console.log('  4. Set up GitHub Actions for autonomous operation');
  console.log('  5. Run: node engine/limits.js');
  console.log('  6. Run: node analytics/report.js --dashboard\n');
}

main().catch(console.error);
