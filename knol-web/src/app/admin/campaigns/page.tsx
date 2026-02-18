'use client';

import { useState, useEffect } from 'react';
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

  // Edit mode
  const editMode = useEditMode<string>();

  // Local state
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [logs, setLogs] = useState<Record<string, CampaignLog[]>>({});
  const [expandedLogs, setExpandedLogs] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [editing, setEditing] = useState<EditingCampaign | null>(null);

  // Sync campaigns from fetch
  useEffect(() => {
    if (campaignsFetch.data) setCampaigns(campaignsFetch.data);
  }, [campaignsFetch.data]);

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
        channels
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

  // Loading state
  if (campaignsFetch.loading) {
    return <Loading message="Loading campaigns..." />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Campaigns"
        description="Manage publishing campaigns and schedules"
      />

      {/* Error messages */}
      {campaignsFetch.error && (
        <ErrorBanner
          message={campaignsFetch.error}
          onRetry={campaignsFetch.refetch}
        />
      )}

      {toggleAction.error && (
        <ErrorBanner message={toggleAction.error} />
      )}

      {saveAction.error && (
        <ErrorBanner message={saveAction.error} />
      )}

      {/* Success message */}
      {message && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <p className="text-green-400 text-sm">{message}</p>
        </div>
      )}

      {/* Campaigns List */}
      {campaigns.length > 0 ? (
        <div className="space-y-4">
          {campaigns.map((campaign) => (
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
                    <p className="text-xs text-dark-500 mt-1">
                      Standard cron format
                    </p>
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
                      placeholder="email, sms, webhook (comma-separated)"
                      className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
                    />
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
                  <div className="flex items-start justify-between mb-4">
                    <div>
                      <h3 className="text-lg font-semibold text-dark-50">
                        {campaign.name}
                      </h3>
                      {campaign.last_publish_at && (
                        <p className="text-sm text-dark-400 mt-1">
                          Last published:{' '}
                          {new Date(campaign.last_publish_at).toLocaleString()}
                        </p>
                      )}
                    </div>
                    <span
                      className={`px-3 py-1 rounded-full text-xs font-semibold ${
                        campaign.enabled
                          ? 'bg-green-500/20 text-green-400'
                          : 'bg-dark-500/20 text-dark-400'
                      }`}
                    >
                      {campaign.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </div>

                  {campaign.cron && (
                    <p className="text-sm text-dark-400 mb-2">
                      <span className="text-dark-500">Schedule:</span>{' '}
                      <code className="font-mono">{campaign.cron}</code>
                    </p>
                  )}

                  {campaign.channels && campaign.channels.length > 0 && (
                    <div className="mb-4">
                      <p className="text-sm text-dark-500 mb-1">Channels:</p>
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

                  <div className="flex space-x-2 pt-2">
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
                                <p className="text-sm text-dark-300">
                                  {log.message || 'No message'}
                                </p>
                                <p className="text-xs text-dark-500 mt-1">
                                  {new Date(log.timestamp).toLocaleString()}
                                </p>
                              </div>
                              <span
                                className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                                  log.status === 'success'
                                    ? 'bg-green-500/20 text-green-400'
                                    : 'bg-red-500/20 text-red-400'
                                }`}
                              >
                                {log.status}
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
      ) : (
        <EmptyState message="No campaigns configured" />
      )}
    </div>
  );
}
