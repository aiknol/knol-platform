'use client';

import { useState, useEffect, useMemo } from 'react';
import { campaignsAPI, Campaign, CampaignLog } from '@/features/admin/api';
import { useAdminFetch, useAdminAction, useEditMode } from '@/features/admin/hooks';
import {
  PageHeader,
  Loading,
  ErrorBanner,
  Button,
  AdminCard,
  EmptyState,
} from '@/features/admin/components';

interface EditingCampaign extends Campaign {
  channelsInput?: string;
}

const PHASE_LABELS: Record<string, string> = {
  launch: 'Launch',
  content_engine: 'Content Engine',
  community: 'Community',
  conversion: 'Conversion',
};

const PHASE_COLORS: Record<string, string> = {
  launch: 'bg-red-500/20 text-red-400 border-red-500/30',
  content_engine: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  community: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
  conversion: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
};

const DOW_ROTATION = [
  { day: 'Mon', category: 'Tip', desc: 'Developer tip or trick' },
  { day: 'Tue', category: 'Benchmark', desc: 'Performance benchmark' },
  { day: 'Wed', category: 'Showcase', desc: 'Feature showcase' },
  { day: 'Thu', category: 'Architecture', desc: 'Architecture deep-dive' },
  { day: 'Fri', category: 'Community', desc: 'Community highlight' },
];

export default function CampaignsPage() {
  // Data fetching
  const campaignsFetch = useAdminFetch(
    () => campaignsAPI.list(),
    []
  );

  // Mutations
  const toggleAction = useAdminAction();
  const saveAction = useAdminAction();
  const logsAction = useAdminAction();
  const triggerAction = useAdminAction();

  // Edit mode
  const editMode = useEditMode<string>();

  // Local state
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [logs, setLogs] = useState<Record<string, CampaignLog[]>>({});
  const [expandedLogs, setExpandedLogs] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [editing, setEditing] = useState<EditingCampaign | null>(null);
  const [filterPhase, setFilterPhase] = useState<string>('all');

  // Sync campaigns from fetch
  useEffect(() => {
    if (campaignsFetch.data) setCampaigns(campaignsFetch.data);
  }, [campaignsFetch.data]);

  // Group campaigns by phase
  const groupedCampaigns = useMemo(() => {
    const filtered = filterPhase === 'all'
      ? campaigns
      : campaigns.filter(c => c.phase === filterPhase);

    const groups: Record<string, Campaign[]> = {};
    for (const c of filtered) {
      const phase = c.phase || 'unknown';
      if (!groups[phase]) groups[phase] = [];
      groups[phase].push(c);
    }
    return groups;
  }, [campaigns, filterPhase]);

  const phases = useMemo(() => {
    const set = new Set(campaigns.map(c => c.phase || 'unknown'));
    return Array.from(set);
  }, [campaigns]);

  const handleEdit = (campaign: Campaign) => {
    editMode.startEdit(campaign.name);
    setEditing({
      ...campaign,
      channelsInput: campaign.channels?.join(', ') || '',
    });
  };

  const handleToggle = async (campaign: Campaign) => {
    const result = await toggleAction.run(async () => {
      await campaignsAPI.update(campaign.name, !campaign.enabled);
    });

    if (result !== null) {
      setCampaigns(campaigns.map((c) =>
        c.name === campaign.name ? { ...c, enabled: !c.enabled } : c
      ));
      setMessage(`Campaign "${campaign.name}" ${!campaign.enabled ? 'enabled' : 'disabled'}`);
      setTimeout(() => setMessage(''), 3000);
    }
  };

  const handleSave = async () => {
    if (!editing || !editMode.editingKey) return;

    const result = await saveAction.run(async () => {
      const channels = editing.channelsInput
        ? editing.channelsInput.split(',').map((c) => c.trim())
        : undefined;

      await campaignsAPI.update(
        editMode.editingKey!,
        editing.enabled,
        editing.cron,
        channels,
        editing.phase,
        editing.description,
      );

      return channels;
    });

    if (result !== null) {
      setCampaigns(campaigns.map((c) =>
        c.name === editMode.editingKey
          ? { ...editing, channels: result }
          : c
      ));

      setMessage(`Campaign "${editMode.editingKey}" updated successfully`);
      editMode.stopEdit();
      setEditing(null);
      setTimeout(() => setMessage(''), 3000);
    }
  };

  const handleTrigger = async (campaign: Campaign) => {
    const force = campaign.phase === 'launch' && !campaign.enabled;
    const result = await triggerAction.run(async () => {
      return await campaignsAPI.trigger(campaign.name, force);
    });

    if (result !== null) {
      setMessage(`Campaign "${campaign.name}" triggered${force ? ' (force mode)' : ''}`);
      setTimeout(() => setMessage(''), 5000);
      // Refresh to pick up new publish log
      campaignsFetch.refetch();
    }
  };

  const handleViewLogs = async (campaignName: string) => {
    if (expandedLogs === campaignName) {
      setExpandedLogs(null);
      return;
    }
    setExpandedLogs(campaignName);
    if (!logs[campaignName]) {
      const result = await logsAction.run(() => campaignsAPI.getLogs(campaignName, 20));
      if (result) {
        setLogs((prev) => ({ ...prev, [campaignName]: result }));
      }
    }
  };

  // Enable all launch campaigns at once
  const handleLaunchSequence = async () => {
    const launchCampaigns = campaigns.filter(c => c.phase === 'launch');
    for (const lc of launchCampaigns) {
      if (!lc.enabled) {
        await campaignsAPI.update(lc.name, true);
      }
    }
    setCampaigns(campaigns.map(c =>
      c.phase === 'launch' ? { ...c, enabled: true } : c
    ));
    setMessage(`All ${launchCampaigns.length} launch campaigns enabled! Ready for launch day.`);
    setTimeout(() => setMessage(''), 5000);
  };

  // Loading state
  if (campaignsFetch.loading) {
    return <Loading message="Loading campaigns..." />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Marketing Campaigns"
        description="Zero-cost marketing strategy — manage campaigns across all phases"
      />

      {/* Error messages */}
      {campaignsFetch.error && (
        <ErrorBanner
          message={campaignsFetch.error}
          onRetry={campaignsFetch.refetch}
        />
      )}

      {toggleAction.error && <ErrorBanner message={toggleAction.error} />}
      {saveAction.error && <ErrorBanner message={saveAction.error} />}
      {triggerAction.error && <ErrorBanner message={triggerAction.error} />}

      {/* Success message */}
      {message && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <p className="text-green-400 text-sm">{message}</p>
        </div>
      )}

      {/* Phase Filter & Strategy Overview */}
      <AdminCard>
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h3 className="text-sm font-medium text-dark-200 mb-2">Filter by Phase</h3>
            <div className="flex flex-wrap gap-2">
              <button
                onClick={() => setFilterPhase('all')}
                className={`px-3 py-1 rounded-full text-xs font-medium border transition-colors ${
                  filterPhase === 'all'
                    ? 'bg-brand-500/20 text-brand-400 border-brand-500/30'
                    : 'bg-dark-700/30 text-dark-400 border-dark-600/30 hover:text-dark-200'
                }`}
              >
                All ({campaigns.length})
              </button>
              {phases.map((phase) => (
                <button
                  key={phase}
                  onClick={() => setFilterPhase(phase)}
                  className={`px-3 py-1 rounded-full text-xs font-medium border transition-colors ${
                    filterPhase === phase
                      ? PHASE_COLORS[phase] || 'bg-dark-500/20 text-dark-300 border-dark-500/30'
                      : 'bg-dark-700/30 text-dark-400 border-dark-600/30 hover:text-dark-200'
                  }`}
                >
                  {PHASE_LABELS[phase] || phase} ({campaigns.filter(c => c.phase === phase).length})
                </button>
              ))}
            </div>
          </div>
          <div className="flex gap-2">
            <Button
              onClick={handleLaunchSequence}
              variant="danger"
              size="sm"
            >
              Activate Launch Sequence
            </Button>
            <a href="/marketing" className="inline-flex items-center px-3 py-1.5 text-xs font-medium text-brand-400 bg-brand-500/10 border border-brand-500/30 rounded-lg hover:bg-brand-500/20 transition-colors">
              View Analytics
            </a>
          </div>
        </div>
      </AdminCard>

      {/* DOW Tweet Rotation Preview */}
      {(filterPhase === 'all' || filterPhase === 'content_engine') && (
        <AdminCard>
          <h3 className="text-sm font-semibold text-dark-200 mb-3">Day-of-Week Tweet Rotation</h3>
          <div className="grid grid-cols-5 gap-2">
            {DOW_ROTATION.map(({ day, category, desc }) => (
              <div key={day} className="bg-dark-700/20 rounded-lg p-3 text-center">
                <div className="text-xs font-bold text-brand-400 mb-1">{day}</div>
                <div className="text-sm font-medium text-dark-200">{category}</div>
                <div className="text-xs text-dark-500 mt-1">{desc}</div>
              </div>
            ))}
          </div>
        </AdminCard>
      )}

      {/* Campaigns by Phase */}
      {Object.keys(groupedCampaigns).length > 0 ? (
        Object.entries(groupedCampaigns).map(([phase, phaseCampaigns]) => (
          <div key={phase} className="space-y-4">
            <div className="flex items-center gap-3">
              <span className={`px-3 py-1 rounded-full text-xs font-semibold border ${PHASE_COLORS[phase] || 'bg-dark-500/20 text-dark-300 border-dark-500/30'}`}>
                {PHASE_LABELS[phase] || phase}
              </span>
              <span className="text-sm text-dark-500">
                {phaseCampaigns.length} campaign{phaseCampaigns.length !== 1 ? 's' : ''}
              </span>
            </div>

            {phaseCampaigns.map((campaign) => (
              <AdminCard key={campaign.name}>
                {editMode.isEditing(campaign.name) && editing ? (
                  // Edit Mode
                  <div className="space-y-4">
                    <div className="flex items-center justify-between mb-4">
                      <h3 className="text-lg font-semibold text-dark-50">
                        {campaign.name}
                      </h3>
                      <label className="flex items-center space-x-3">
                        <span className="text-sm text-dark-300">Enabled</span>
                        <input
                          type="checkbox"
                          checked={editing.enabled}
                          onChange={(e) =>
                            setEditing({ ...editing, enabled: e.target.checked })
                          }
                          className="w-5 h-5 rounded bg-dark-700/50 border-dark-600/50 text-brand-600 focus:ring-brand-500"
                        />
                      </label>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-dark-200 mb-2">
                        Description
                      </label>
                      <input
                        type="text"
                        value={editing.description || ''}
                        onChange={(e) =>
                          setEditing({ ...editing, description: e.target.value })
                        }
                        placeholder="Campaign description"
                        className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
                      />
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-dark-200 mb-2">
                          Cron Schedule
                        </label>
                        <input
                          type="text"
                          value={editing.cron || ''}
                          onChange={(e) =>
                            setEditing({ ...editing, cron: e.target.value })
                          }
                          placeholder="0 0 * * * (cron format)"
                          className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-dark-200 mb-2">
                          Phase
                        </label>
                        <select
                          value={editing.phase || ''}
                          onChange={(e) =>
                            setEditing({ ...editing, phase: e.target.value })
                          }
                          className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
                        >
                          <option value="launch">Launch</option>
                          <option value="content_engine">Content Engine</option>
                          <option value="community">Community</option>
                          <option value="conversion">Conversion</option>
                        </select>
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-dark-200 mb-2">
                        Channels
                      </label>
                      <input
                        type="text"
                        value={editing.channelsInput || ''}
                        onChange={(e) =>
                          setEditing({
                            ...editing,
                            channelsInput: e.target.value,
                          })
                        }
                        placeholder="twitter, blog, devto, hashnode, medium, reddit, linkedin, email, github, producthunt, hackernews"
                        className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
                      />
                      <p className="text-xs text-dark-500 mt-1">
                        Comma-separated: twitter, blog, devto, hashnode, medium, reddit, linkedin, email, github, producthunt, hackernews
                      </p>
                    </div>

                    <div className="flex space-x-2 pt-2">
                      <Button
                        onClick={handleSave}
                        disabled={saveAction.loading}
                        variant="primary"
                        size="sm"
                      >
                        {saveAction.loading ? 'Saving...' : 'Save'}
                      </Button>
                      <Button
                        onClick={() => {
                          editMode.stopEdit();
                          setEditing(null);
                        }}
                        variant="secondary"
                        size="sm"
                      >
                        Cancel
                      </Button>
                    </div>
                  </div>
                ) : (
                  // View Mode
                  <>
                    <div className="flex items-start justify-between mb-2">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <h3 className="text-lg font-semibold text-dark-50">
                            {campaign.name}
                          </h3>
                          <span
                            className={`px-2 py-0.5 rounded-full text-[10px] font-semibold ${
                              campaign.enabled
                                ? 'bg-green-500/20 text-green-400'
                                : 'bg-dark-500/20 text-dark-400'
                            }`}
                          >
                            {campaign.enabled ? 'Enabled' : 'Disabled'}
                          </span>
                        </div>
                        {campaign.description && (
                          <p className="text-sm text-dark-400 mb-2">{campaign.description}</p>
                        )}
                      </div>
                      {/* Stats Badge */}
                      {campaign.stats && campaign.stats.total_publishes > 0 && (
                        <div className="text-right">
                          <div className="text-xs text-dark-500">
                            {campaign.stats.total_publishes} publishes
                          </div>
                          <div className={`text-sm font-bold ${campaign.stats.success_rate >= 90 ? 'text-green-400' : campaign.stats.success_rate >= 70 ? 'text-amber-400' : 'text-red-400'}`}>
                            {campaign.stats.success_rate}% success
                          </div>
                        </div>
                      )}
                    </div>

                    <div className="flex flex-wrap gap-x-6 gap-y-1 text-sm text-dark-400 mb-3">
                      {campaign.cron && (
                        <span>
                          <span className="text-dark-500">Schedule:</span>{' '}
                          <code className="font-mono text-xs">{campaign.cron}</code>
                        </span>
                      )}
                      {campaign.last_publish && (
                        <span>
                          <span className="text-dark-500">Last:</span>{' '}
                          <span className={campaign.last_publish.success ? 'text-green-400' : 'text-red-400'}>
                            {campaign.last_publish.channel}
                          </span>{' '}
                          {new Date(campaign.last_publish.published_at).toLocaleString()}
                        </span>
                      )}
                    </div>

                    {campaign.channels && campaign.channels.length > 0 && (
                      <div className="mb-4">
                        <div className="flex flex-wrap gap-2">
                          {campaign.channels.map((channel) => (
                            <span
                              key={channel}
                              className="px-2 py-1 rounded text-xs bg-brand-500/20 text-brand-400 border border-brand-500/30"
                            >
                              {channel}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}

                    <div className="flex flex-wrap gap-2 pt-2">
                      <Button
                        onClick={() => handleToggle(campaign)}
                        disabled={toggleAction.loading}
                        variant={campaign.enabled ? 'danger' : 'primary'}
                        size="sm"
                      >
                        {toggleAction.loading
                          ? 'Updating...'
                          : campaign.enabled
                            ? 'Disable'
                            : 'Enable'}
                      </Button>
                      <Button
                        onClick={() => handleTrigger(campaign)}
                        disabled={triggerAction.loading}
                        variant="primary"
                        size="sm"
                      >
                        {triggerAction.loading ? 'Triggering...' : 'Trigger Now'}
                      </Button>
                      <Button
                        onClick={() => handleEdit(campaign)}
                        variant="secondary"
                        size="sm"
                      >
                        Edit
                      </Button>
                      <Button
                        onClick={() => handleViewLogs(campaign.name)}
                        variant="ghost"
                        size="sm"
                      >
                        {expandedLogs === campaign.name ? 'Hide Logs' : 'View Logs'}
                      </Button>
                    </div>

                    {/* Per-campaign Publish Logs */}
                    {expandedLogs === campaign.name && (
                      <div className="mt-4 pt-4 border-t border-dark-600/30">
                        <h4 className="text-sm font-semibold text-dark-200 mb-3">Publish History</h4>
                        {logsAction.loading ? (
                          <p className="text-dark-400 text-sm">Loading logs...</p>
                        ) : logs[campaign.name] && logs[campaign.name].length > 0 ? (
                          <div className="space-y-2">
                            {logs[campaign.name].map((log, idx) => (
                              <div
                                key={idx}
                                className="flex items-start justify-between p-3 bg-dark-700/20 rounded-lg"
                              >
                                <div className="flex-1">
                                  <div className="flex items-center gap-2 mb-1">
                                    <span className="text-xs font-medium text-brand-400">{log.channel}</span>
                                    {log.url && (
                                      <a
                                        href={log.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-xs text-dark-500 hover:text-brand-400 truncate max-w-xs"
                                      >
                                        {log.url}
                                      </a>
                                    )}
                                  </div>
                                  {log.error && (
                                    <p className="text-xs text-red-400">{log.error}</p>
                                  )}
                                  <p className="text-xs text-dark-500 mt-1">
                                    {new Date(log.published_at).toLocaleString()}
                                  </p>
                                </div>
                                <span
                                  className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                                    log.success
                                      ? 'bg-green-500/20 text-green-400'
                                      : 'bg-red-500/20 text-red-400'
                                  }`}
                                >
                                  {log.success ? 'success' : 'failed'}
                                </span>
                              </div>
                            ))}
                          </div>
                        ) : (
                          <p className="text-dark-400 text-sm">No publish history</p>
                        )}
                      </div>
                    )}
                  </>
                )}
              </AdminCard>
            ))}
          </div>
        ))
      ) : (
        <EmptyState message="No campaigns configured" />
      )}
    </div>
  );
}
