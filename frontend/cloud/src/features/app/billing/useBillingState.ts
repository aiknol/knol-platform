'use client';

import { useCallback, useEffect, useState } from 'react';
import {
  appBillingAPI,
  appAuthAPI,
  getAppAuthUser,
  SubscriptionInfo,
  UsageInfo,
  Invoice,
  UpcomingInvoice,
} from '@/features/app/api';
import type { UsageHistoryItem, AppUser } from '@/features/app/api';

/** Validate that a URL is from a trusted domain before redirecting. */
function isTrustedRedirectUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    const host = parsed.hostname.toLowerCase();
    // Only allow Stripe checkout/portal domains
    return (
      host === 'checkout.stripe.com' ||
      host === 'billing.stripe.com' ||
      host.endsWith('.stripe.com')
    );
  } catch {
    return false;
  }
}

export function useBillingState() {
  const [user] = useState<AppUser | null>(getAppAuthUser());
  const [subscription, setSubscription] = useState<SubscriptionInfo | null>(null);
  const [usage, setUsage] = useState<UsageInfo | null>(null);
  const [usageHistory, setUsageHistory] = useState<UsageHistoryItem[]>([]);
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [upcomingInvoice, setUpcomingInvoice] = useState<UpcomingInvoice | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [actionBusy, setActionBusy] = useState(false);
  const [successMessage, setSuccessMessage] = useState('');

  const canManage = user?.role === 'owner' || user?.role === 'admin';
  const isOwner = user?.role === 'owner';

  // Check URL params for Stripe redirect
  const [checkoutSuccess, setCheckoutSuccess] = useState(false);
  const [checkoutCanceled, setCheckoutCanceled] = useState(false);

  useEffect(() => {
    if (typeof window !== 'undefined') {
      const params = new URLSearchParams(window.location.search);
      if (params.get('session_id')) setCheckoutSuccess(true);
      if (params.get('canceled')) setCheckoutCanceled(true);
      // Clean up URL params
      if (params.get('session_id') || params.get('canceled')) {
        window.history.replaceState({}, '', window.location.pathname);
      }
    }
  }, []);

  const load = useCallback(async () => {
    setError('');
    setLoading(true);
    try {
      const [sub, usageData, history] = await Promise.all([
        appBillingAPI.getSubscription(),
        appBillingAPI.getUsage(),
        appBillingAPI.getUsageHistory(),
      ]);
      setSubscription(sub);
      setUsage(usageData);
      setUsageHistory(history);

      // Only load invoices if user has billing access and has a Stripe customer
      if (canManage && sub.has_stripe_customer) {
        try {
          const [inv, upcoming] = await Promise.all([
            appBillingAPI.listInvoices(),
            appBillingAPI.upcomingInvoice(),
          ]);
          setInvoices(inv.invoices);
          setUpcomingInvoice(upcoming);
        } catch {
          // Invoices may not be available for all accounts
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load billing data');
    } finally {
      setLoading(false);
    }
  }, [canManage]);

  useEffect(() => {
    load().catch(() => undefined);
  }, [load]);

  const onUpgrade = async (plan: string) => {
    setError('');
    setActionBusy(true);
    try {
      const result = await appBillingAPI.createCheckout(plan);
      if (!isTrustedRedirectUrl(result.checkout_url)) {
        setError('Unexpected checkout URL. Please try again.');
        setActionBusy(false);
        return;
      }
      window.location.href = result.checkout_url;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start checkout');
      setActionBusy(false);
    }
  };

  const onOpenPortal = async () => {
    setError('');
    setActionBusy(true);
    try {
      const result = await appBillingAPI.createPortal();
      if (!isTrustedRedirectUrl(result.portal_url)) {
        setError('Unexpected portal URL. Please try again.');
        setActionBusy(false);
        return;
      }
      window.location.href = result.portal_url;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to open billing portal');
      setActionBusy(false);
    }
  };

  const onCancel = async () => {
    if (!window.confirm('Cancel your subscription? You will retain access until the end of the current billing period.')) {
      return;
    }
    setError('');
    setActionBusy(true);
    try {
      await appBillingAPI.cancelSubscription();
      setSuccessMessage('Subscription will be canceled at the end of the billing period.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cancel subscription');
    } finally {
      setActionBusy(false);
    }
  };

  const onReactivate = async () => {
    setError('');
    setActionBusy(true);
    try {
      await appBillingAPI.reactivateSubscription();
      setSuccessMessage('Subscription reactivated successfully!');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reactivate subscription');
    } finally {
      setActionBusy(false);
    }
  };

  return {
    user,
    subscription,
    usage,
    usageHistory,
    invoices,
    upcomingInvoice,
    loading,
    error,
    actionBusy,
    successMessage,
    canManage,
    isOwner,
    checkoutSuccess,
    checkoutCanceled,
    onUpgrade,
    onOpenPortal,
    onCancel,
    onReactivate,
  };
}
