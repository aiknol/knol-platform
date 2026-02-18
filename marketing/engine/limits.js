// =============================================================================
// Knol Marketing — Limit State Inspector
// =============================================================================

const limiter = require('./limit-intelligence');

function formatCooldown(until) {
  if (!until) return 'none';
  const delta = Math.ceil((new Date(until).getTime() - Date.now()) / 1000);
  if (delta <= 0) return 'expired';
  return `${delta}s`;
}

function main() {
  const snapshot = limiter.getStateSnapshot();
  const channels = Object.entries(snapshot.channels || {});

  if (!channels.length) {
    console.log('No rate-limit state yet. Run a campaign first.');
    return;
  }

  console.log('Knol Marketing — Rate Limit State\n');
  for (const [channel, s] of channels) {
    const dayCount = Object.values(s.day || {}).reduce((a, b) => a + b, 0);
    const monthCount = Object.values(s.month || {}).reduce((a, b) => a + b, 0);
    console.log(`[${channel}]`);
    console.log(`  cooldown: ${formatCooldown(s.cooldownUntil)}`);
    console.log(`  lastPostAt: ${s.lastPostAt || 'none'}`);
    console.log(`  dayUsageTotal: ${dayCount}`);
    console.log(`  monthUsageTotal: ${monthCount}`);
    const telemetryKeys = Object.keys(s.telemetry || {});
    if (telemetryKeys.length) {
      console.log(`  telemetry:`);
      for (const key of telemetryKeys) {
        const t = s.telemetry[key] || {};
        console.log(`    - ${key}: remaining=${t.remaining ?? 'n/a'} reset=${t.resetEpochSeconds ?? 'n/a'} updated=${t.updatedAt ?? 'n/a'}`);
      }
    }
    console.log('');
  }
}

if (require.main === module) {
  main();
}
