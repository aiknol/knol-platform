'use client';

import { useState } from 'react';
import { campaignsAPI, MarketingStats } from '@/features/admin/api';
import { useAdminFetch, useAdminAction } from '@/features/admin/hooks';
import {
  PageHeader,
  Loading,
  ErrorBanner,
  StatCard,
  AdminCard,
  Button,
} from '@/features/admin/components';

const PHASE_LABELS: Record<string, string> = {
  launch: 'Launch',
  content_engine: 'Content Engine',
  community: 'Community',
  conversion: 'Conversion',
};

const CHANNEL_COLORS: Record<string, string> = {
  twitter: 'bg-blue-500/20 text-blue-400',
  blog: 'bg-green-500/20 text-green-400',
  devto: 'bg-dark-500/20 text-dark-200',
  hashnode: 'bg-blue-600/20 text-blue-300',
  medium: 'bg-dark-400/20 text-dark-200',
  reddit: 'bg-orange-500/20 text-orange-400',
  linkedin: 'bg-blue-700/20 text-blue-300',
  email: 'bg-amber-500/20 text-amber-400',
  github: 'bg-purple-500/20 text-purple-400',
  producthunt: 'bg-red-500/20 text-red-400',
  hackernews: 'bg-orange-600/20 text-orange-300',
};

export default function MarketingDashboard() {
  const [days, setDays] = useState(30);

  const statsFetch = useAdminFetch(
    () => campaignsAPI.getStats(days),
    [days]
  );

  const recordAction = useAdminAction();
  const [metricForm, setMetricForm] = useState({ name: '', value: '' });
  const [message, setMessage] = useState('');

  const stats = statsFetch.data as MarketingStats | null;

  const handleRecordMetric = async () => {
    if (!metricForm.name || !metricForm.value) return;
    const result = await recordAction.run(() =>
      campaignsAPI.recordMetric(metricForm.name, parseFloat(metricForm.value))
    );
    if (result !== null) {
      setMessage(`Metric "${metricForm.name}" recorded`);
      setMetricForm({ name: '', value: '' });
      statsFetch.refetch();
      setTimeout(() => setMessage(''), 3000);
    }
  };

  if (statsFetch.loading) {
    return <Loading message="Loading marketing analytics..." />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Marketing Analytics"
        description="Zero-cost marketing strategy performance dashboard"
        action={
          <div className="flex items-center gap-2">
            {[7, 14, 30, 90].map((d) => (
              <button
                key={d}
                onClick={() => setDays(d)}
                className={`px-3 py-1 rounded-full text-xs font-medium border transition-colors ${
                  days === d
                    ? 'bg-brand-500/20 text-brand-400 border-brand-500/30'
                    : 'bg-dark-700/30 text-dark-400 border-dark-600/30 hover:text-dark-200'
                }`}
              >
                {d}d
              </button>
            ))}
          </div>
        }
      />

      {statsFetch.error && (
        <ErrorBanner message={statsFetch.error} onRetry={statsFetch.refetch} />
      )}

      {message && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <p className="text-green-400 text-sm">{message}</p>
        </div>
      )}

      {stats && (
        <>
          {/* Summary Stats */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <StatCard
              label="Total Publishes"
              value={stats.summary.total_publishes}
              icon="📢"
            />
            <StatCard
              label="Successful"
              value={stats.summary.successful}
              icon="✅"
            />
            <StatCard
              label="Success Rate"
              value={`${stats.summary.success_rate}%`}
              icon="📈"
            />
            <StatCard
              label="Active Channels"
              value={stats.by_channel.length}
              icon="📡"
            />
          </div>

          {/* Strategy Badge */}
          <AdminCard>
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-lg font-semibold text-dark-50">Strategy: Zero-Cost</h3>
                <p className="text-sm text-dark-400 mt-1">
                  Target $0 ad spend. Organic growth through developer content, community engagement, and cross-platform syndication.
                </p>
              </div>
              <span className="px-4 py-2 rounded-full text-sm font-bold bg-green-500/20 text-green-400 border border-green-500/30">
                $0 spend
              </span>
            </div>
          </AdminCard>

          {/* Channel Performance */}
          <AdminCard>
            <h3 className="text-lg font-semibold text-dark-50 mb-4">Channel Performance</h3>
            {stats.by_channel.length > 0 ? (
              <div className="space-y-3">
                {stats.by_channel.map((ch) => (
                  <div key={ch.channel} className="flex items-center justify-between p-4 bg-dark-700/20 rounded-lg">
                    <div className="flex items-center gap-3">
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold ${CHANNEL_COLORS[ch.channel] || 'bg-dark-500/20 text-dark-300'}`}>
                        {ch.channel}
                      </span>
                    </div>
                    <div className="flex items-center gap-6">
                      <div className="text-right">
                        <div className="text-sm font-medium text-dark-200">{ch.total} posts</div>
                        <div className="text-xs text-dark-500">{ch.successful} successful</div>
                      </div>
                      <div className="w-24">
                        <div className="bg-dark-700 rounded-full h-2 overflow-hidden">
                          <div
                            className={`h-2 rounded-full ${ch.success_rate >= 90 ? 'bg-green-500' : ch.success_rate >= 70 ? 'bg-amber-500' : 'bg-red-500'}`}
                            style={{ width: `${ch.success_rate}%` }}
                          />
                        </div>
                        <div className="text-xs text-dark-500 text-right mt-1">{ch.success_rate}%</div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-dark-400 text-sm">No channel data yet. Publish some campaigns to see performance.</p>
            )}
          </AdminCard>

          {/* Phase Performance */}
          <AdminCard>
            <h3 className="text-lg font-semibold text-dark-50 mb-4">Phase Performance</h3>
            {stats.by_phase.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                {stats.by_phase.map((p) => (
                  <div key={p.phase} className="bg-dark-700/20 rounded-lg p-4 text-center">
                    <div className="text-xs font-bold text-brand-400 mb-2">
                      {PHASE_LABELS[p.phase] || p.phase}
                    </div>
                    <div className="text-2xl font-bold text-dark-100">{p.total}</div>
                    <div className="text-xs text-dark-500">
                      {p.successful} successful
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-dark-400 text-sm">No phase data yet.</p>
            )}
          </AdminCard>

          {/* Daily Activity */}
          {stats.daily.length > 0 && (
            <AdminCard>
              <h3 className="text-lg font-semibold text-dark-50 mb-4">Daily Activity</h3>
              <div className="overflow-x-auto">
                <div className="flex gap-1 items-end" style={{ minHeight: 120 }}>
                  {stats.daily.map((d) => {
                    const maxTotal = Math.max(...stats.daily.map(dd => dd.total), 1);
                    const height = Math.max((d.total / maxTotal) * 100, 4);
                    const successPct = d.total > 0 ? (d.successful / d.total) * 100 : 0;
                    return (
                      <div key={d.date} className="flex flex-col items-center gap-1 flex-1 min-w-[24px]" title={`${d.date}: ${d.total} posts, ${d.successful} success`}>
                        <div
                          className={`w-full rounded-t ${successPct >= 90 ? 'bg-green-500/60' : successPct >= 70 ? 'bg-amber-500/60' : 'bg-red-500/60'}`}
                          style={{ height: `${height}px` }}
                        />
                        <div className="text-[8px] text-dark-600 rotate-45 origin-left whitespace-nowrap">
                          {d.date.slice(5)}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </AdminCard>
          )}

          {/* Custom Metrics */}
          <AdminCard>
            <h3 className="text-lg font-semibold text-dark-50 mb-4">Growth Metrics</h3>
            {stats.metrics.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                {stats.metrics.map((m) => (
                  <div key={m.name} className="bg-dark-700/20 rounded-lg p-4">
                    <div className="text-xs text-dark-500 mb-1">{m.name}</div>
                    <div className="text-2xl font-bold text-dark-100">
                      {m.name.includes('mrr') || m.name.includes('revenue')
                        ? `$${m.value.toLocaleString()}`
                        : m.value.toLocaleString()}
                    </div>
                    <div className="text-xs text-dark-600 mt-1">
                      as of {m.recorded_at}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-dark-400 text-sm mb-4">No metrics recorded yet. Track key growth indicators below.</p>
            )}

            {/* Record Metric Form */}
            <div className="pt-4 border-t border-dark-600/30">
              <h4 className="text-sm font-medium text-dark-300 mb-3">Record a Metric</h4>
              <div className="flex gap-3">
                <select
                  value={metricForm.name}
                  onChange={(e) => setMetricForm({ ...metricForm, name: e.target.value })}
                  className="flex-1 px-3 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-sm text-dark-100 focus:outline-none focus:border-brand-500/50"
                >
                  <option value="">Select metric...</option>
                  <option value="github_stars">GitHub Stars</option>
                  <option value="discord_members">Discord Members</option>
                  <option value="docker_pulls">Docker Pulls</option>
                  <option value="blog_monthly_visits">Blog Monthly Visits</option>
                  <option value="cloud_signups">Cloud Signups</option>
                  <option value="mrr">MRR ($)</option>
                  <option value="twitter_followers">Twitter Followers</option>
                  <option value="npm_downloads">NPM Downloads</option>
                </select>
                <input
                  type="number"
                  value={metricForm.value}
                  onChange={(e) => setMetricForm({ ...metricForm, value: e.target.value })}
                  placeholder="Value"
                  className="w-32 px-3 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-sm text-dark-100 focus:outline-none focus:border-brand-500/50"
                />
                <Button
                  onClick={handleRecordMetric}
                  disabled={recordAction.loading || !metricForm.name || !metricForm.value}
                  variant="primary"
                  size="sm"
                >
                  {recordAction.loading ? 'Recording...' : 'Record'}
                </Button>
              </div>
              {recordAction.error && (
                <p className="text-red-400 text-xs mt-2">{recordAction.error}</p>
              )}
            </div>
          </AdminCard>
        </>
      )}
    </div>
  );
}
