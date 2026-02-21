import { apiFetch } from './client';

export interface Campaign {
  id: string;
  name: string;
  enabled: boolean;
  cron?: string;
  channels?: string[];
  phase?: string;
  description?: string;
  created_at?: string;
  updated_at?: string;
  last_publish?: {
    channel: string;
    success: boolean;
    published_at: string;
  } | null;
  stats?: {
    total_publishes: number;
    successful: number;
    success_rate: number;
  } | null;
}

export interface CampaignLog {
  campaign: string;
  channel: string;
  success: boolean;
  message_id?: string;
  url?: string;
  error?: string;
  published_at: string;
}

export interface MarketingStats {
  period_days: number;
  strategy: string;
  summary: {
    total_publishes: number;
    successful: number;
    success_rate: number;
  };
  by_channel: Array<{
    channel: string;
    total: number;
    successful: number;
    success_rate: number;
  }>;
  by_phase: Array<{
    phase: string;
    total: number;
    successful: number;
  }>;
  daily: Array<{
    date: string;
    total: number;
    successful: number;
  }>;
  metrics: Array<{
    name: string;
    value: number;
    recorded_at: string;
    metadata?: Record<string, unknown>;
  }>;
}

export const campaignsAPI = {
  list: async (): Promise<Campaign[]> => apiFetch('/admin/campaigns'),
  update: async (
    name: string,
    enabled?: boolean,
    cron?: string,
    channels?: string[],
    phase?: string,
    description?: string,
  ) =>
    apiFetch(`/admin/campaigns/${name}`, {
      method: 'PUT',
      body: JSON.stringify({
        ...(enabled !== undefined && { enabled }),
        ...(cron && { cron }),
        ...(channels && { channels }),
        ...(phase && { phase }),
        ...(description && { description }),
      }),
    }),
  getLogs: async (campaignName: string, limit: number = 50): Promise<CampaignLog[]> =>
    apiFetch(`/admin/campaigns/${campaignName}/logs?limit=${limit}`),
  trigger: async (campaignName: string, force: boolean = false) =>
    apiFetch(`/admin/campaigns/${campaignName}/trigger?force=${force}`, { method: 'POST' }),
  getStats: async (days: number = 30): Promise<MarketingStats> =>
    apiFetch(`/admin/marketing/stats?days=${days}`),
  recordMetric: async (metricName: string, metricValue: number, metadata?: Record<string, unknown>) =>
    apiFetch('/admin/marketing/metrics', {
      method: 'POST',
      body: JSON.stringify({ metric_name: metricName, metric_value: metricValue, metadata }),
    }),
};
