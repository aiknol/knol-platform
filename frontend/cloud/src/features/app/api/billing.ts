import { apiFetch } from './client';

// ── Response types (match service-tenant billing.rs JSON responses) ──

export interface SubscriptionInfo {
  plan: string;
  subscription_status: string;
  billing_period_start: string | null;
  billing_period_end: string | null;
  usage_ops_month: number;
  usage_limit: number | null;
  has_stripe_customer: boolean;
  stripe_status?: string;
  cancel_at_period_end?: boolean;
}

export interface CheckoutResponse {
  checkout_url: string;
  session_id: string;
}

export interface PortalResponse {
  portal_url: string;
}

export interface CancelResponse {
  canceled: boolean;
  cancel_at_period_end: boolean;
  current_period_end: string | null;
}

export interface ReactivateResponse {
  reactivated: boolean;
  status: string;
}

export interface Invoice {
  id: string;
  status: string;
  amount_due: number;
  amount_paid: number;
  currency: string;
  hosted_invoice_url: string | null;
  created: number;
  period_start: number;
  period_end: number;
}

export interface InvoicesResponse {
  invoices: Invoice[];
  has_more: boolean;
}

export interface UpcomingInvoice {
  amount_due: number;
  currency: string;
  period_start: number;
  period_end: number;
}

export interface UsageInfo {
  plan: string;
  ops_this_month: number;
  ops_limit: number | null;
  usage_percentage: number | null;
  alerts_triggered: number[];
  month: string;
}

// ── API ──

export const appBillingAPI = {
  getSubscription: async () =>
    apiFetch<SubscriptionInfo>('/app/billing/subscription', { method: 'GET' }),

  createCheckout: async (plan: string) =>
    apiFetch<CheckoutResponse>('/app/billing/checkout', {
      method: 'POST',
      body: JSON.stringify({ plan }),
    }),

  createPortal: async () =>
    apiFetch<PortalResponse>('/app/billing/portal', { method: 'POST' }),

  cancelSubscription: async () =>
    apiFetch<CancelResponse>('/app/billing/cancel', { method: 'POST' }),

  reactivateSubscription: async () =>
    apiFetch<ReactivateResponse>('/app/billing/reactivate', { method: 'POST' }),

  listInvoices: async () =>
    apiFetch<InvoicesResponse>('/app/billing/invoices', { method: 'GET' }),

  upcomingInvoice: async () =>
    apiFetch<UpcomingInvoice>('/app/billing/invoices/upcoming', { method: 'GET' }),

  getUsage: async () =>
    apiFetch<UsageInfo>('/app/billing/usage', { method: 'GET' }),

  getUsageHistory: async () =>
    apiFetch<import('./types').UsageHistoryItem[]>('/app/billing/usage/history', { method: 'GET' }),
};
