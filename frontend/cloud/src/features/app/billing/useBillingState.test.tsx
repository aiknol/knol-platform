import { renderHook, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getAppAuthUser: vi.fn(),
  appBillingAPI: {
    getSubscription: vi.fn(),
    getUsage: vi.fn(),
    getUsageHistory: vi.fn(),
    listInvoices: vi.fn(),
    upcomingInvoice: vi.fn(),
    createCheckout: vi.fn(),
    createPortal: vi.fn(),
    cancelSubscription: vi.fn(),
    reactivateSubscription: vi.fn(),
  },
}));

vi.mock('@/features/app/api', () => ({
  getAppAuthUser: apiMocks.getAppAuthUser,
  appBillingAPI: apiMocks.appBillingAPI,
}));

import { useBillingState } from './useBillingState';

const MOCK_SUBSCRIPTION = {
  plan: 'free',
  status: 'active',
  has_stripe_customer: false,
  cancel_at_period_end: false,
};

const MOCK_USAGE = {
  plan: 'free',
  ops_this_month: 10,
  ops_limit: 1000,
  usage_percentage: 1.0,
  month: '2026-02',
};

beforeEach(() => {
  vi.clearAllMocks();
  apiMocks.getAppAuthUser.mockReturnValue({ id: 'u1', role: 'owner', email: 'test@test.com' });
  apiMocks.appBillingAPI.getSubscription.mockResolvedValue(MOCK_SUBSCRIPTION);
  apiMocks.appBillingAPI.getUsage.mockResolvedValue(MOCK_USAGE);
  apiMocks.appBillingAPI.getUsageHistory.mockResolvedValue([]);
  apiMocks.appBillingAPI.listInvoices.mockResolvedValue({ invoices: [] });
  apiMocks.appBillingAPI.upcomingInvoice.mockResolvedValue(null);

  // Clean URL params
  window.history.replaceState({}, '', '/');
});

describe('useBillingState', () => {
  it('loads subscription and usage', async () => {
    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.subscription?.plan).toBe('free');
    expect(result.current.usage?.ops_this_month).toBe(10);
    expect(result.current.error).toBe('');
  });

  it('loads invoices for owner with stripe customer', async () => {
    apiMocks.appBillingAPI.getSubscription.mockResolvedValue({
      ...MOCK_SUBSCRIPTION,
      has_stripe_customer: true,
    });
    apiMocks.appBillingAPI.listInvoices.mockResolvedValue({
      invoices: [{ id: 'inv_1', amount_due: 100 }],
    });

    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.invoices).toHaveLength(1);
  });

  it('skips invoices for non-admin', async () => {
    apiMocks.getAppAuthUser.mockReturnValue({ id: 'u1', role: 'developer', email: 'dev@test.com' });

    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(apiMocks.appBillingAPI.listInvoices).not.toHaveBeenCalled();
  });

  it('detects checkout success URL param', async () => {
    window.history.replaceState({}, '', '/?session_id=cs_test_123');
    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.checkoutSuccess).toBe(true);
  });

  it('detects checkout canceled URL param', async () => {
    window.history.replaceState({}, '', '/?canceled=1');
    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.checkoutCanceled).toBe(true);
  });

  it('onUpgrade validates checkout URL', async () => {
    apiMocks.appBillingAPI.createCheckout.mockResolvedValue({
      checkout_url: 'https://evil.com/steal',
    });
    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.onUpgrade('builder');
    });
    expect(result.current.error).toContain('Unexpected checkout URL');
  });

  it('onOpenPortal validates portal URL', async () => {
    apiMocks.appBillingAPI.createPortal.mockResolvedValue({
      portal_url: 'https://evil.com/phish',
    });
    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.onOpenPortal();
    });
    expect(result.current.error).toContain('Unexpected portal URL');
  });

  it('sets error on API failure', async () => {
    apiMocks.appBillingAPI.getSubscription.mockRejectedValue(new Error('Network error'));
    const { result } = renderHook(() => useBillingState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe('Network error');
  });
});
