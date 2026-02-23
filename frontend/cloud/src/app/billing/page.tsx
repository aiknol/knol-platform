'use client';

import { useBillingState } from '@/features/app/billing/useBillingState';
import PageHeader from '@/components/PageHeader';
import StatusBadge from '@/components/StatusBadge';
import UsageBar from '@/components/UsageBar';
import UsageChart from '@/components/UsageChart';
import EmptyState from '@/components/EmptyState';

const PLANS = [
  {
    id: 'free',
    name: 'Free',
    price: '$0',
    period: '',
    ops: 'Rate limited',
    features: ['10 RPS', 'Community support', 'Single user'],
  },
  {
    id: 'builder',
    name: 'Builder',
    price: '$49',
    period: '/mo',
    ops: '100K ops/mo',
    features: ['100,000 ops/month', '100 RPS', 'Email support', 'Team access'],
  },
  {
    id: 'growth',
    name: 'Growth',
    price: '$199',
    period: '/mo',
    ops: '500K ops/mo',
    features: ['500,000 ops/month', '500 RPS', 'Priority support', 'Team access', 'Custom webhooks'],
  },
];

function formatCents(cents: number, currency: string): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency.toUpperCase(),
  }).format(cents / 100);
}

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

export default function BillingPage() {
  const {
    subscription,
    usage,
    usageHistory,
    invoices,
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
  } = useBillingState();

  if (loading) {
    return <p className="text-dark-300 py-8">Loading billing...</p>;
  }

  const currentPlan = subscription?.plan || 'free';
  const isCanceled = subscription?.cancel_at_period_end === true;

  return (
    <div className="space-y-8">
      <PageHeader title="Billing" description="Manage your subscription, usage, and invoices." />

      {error && <div className="alert-error">{error}</div>}
      {successMessage && <div className="alert-success">{successMessage}</div>}
      {checkoutSuccess && <div className="alert-success">Payment successful! Your subscription is now active.</div>}
      {checkoutCanceled && <div className="alert-warning">Checkout was canceled. No changes were made.</div>}

      {/* Subscription Status */}
      <section className="card">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">Current Plan</p>
            <div className="flex items-center gap-3">
              <h3 className="text-xl font-semibold text-dark-50 capitalize">{currentPlan}</h3>
              <StatusBadge status={subscription?.subscription_status || 'none'} />
            </div>
            {subscription?.billing_period_end && (
              <p className="text-sm text-dark-400 mt-1">
                {isCanceled ? 'Access until' : 'Renews'}: {new Date(subscription.billing_period_end).toLocaleDateString()}
              </p>
            )}
            {isCanceled && (
              <p className="text-sm text-amber-400 mt-1">
                Subscription will end at the current billing period.
              </p>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            {subscription?.has_stripe_customer && canManage && (
              <button
                onClick={onOpenPortal}
                disabled={actionBusy}
                className="btn-secondary !px-4 !py-2 text-sm disabled:opacity-50"
              >
                Manage Payment
              </button>
            )}
            {isCanceled && isOwner && (
              <button
                onClick={onReactivate}
                disabled={actionBusy}
                className="btn-primary !px-4 !py-2 text-sm disabled:opacity-50"
              >
                Reactivate
              </button>
            )}
            {!isCanceled && subscription?.subscription_status === 'active' && isOwner && (
              <button
                onClick={onCancel}
                disabled={actionBusy}
                className="px-4 py-2 text-sm rounded-lg border border-red-500/30 text-red-400 hover:bg-red-500/10 disabled:opacity-50"
              >
                Cancel Subscription
              </button>
            )}
          </div>
        </div>
      </section>

      {/* Usage */}
      <section className="card">
        <h3 className="text-lg font-semibold text-dark-50 mb-4">Usage</h3>
        <div className="grid gap-6 md:grid-cols-2">
          <div>
            <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">This Month</p>
            <p className="text-3xl font-bold text-dark-50">{(usage?.ops_this_month ?? 0).toLocaleString()}</p>
            <p className="text-sm text-dark-400 mt-1">operations</p>
            <UsageBar
              used={usage?.ops_this_month ?? 0}
              limit={usage?.ops_limit ?? null}
              className="mt-3"
            />
          </div>
          <div>
            <p className="text-xs uppercase tracking-wide text-dark-400 mb-2">12-Month History</p>
            <UsageChart data={usageHistory} />
          </div>
        </div>
      </section>

      {/* Plan Cards */}
      {canManage && (
        <section>
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Plans</h3>
          <div className="grid gap-4 sm:grid-cols-3">
            {PLANS.map((plan) => {
              const isCurrent = currentPlan === plan.id;
              return (
                <div
                  key={plan.id}
                  className={`rounded-xl border p-5 transition-colors ${
                    isCurrent
                      ? 'border-brand-500/50 bg-brand-500/5'
                      : 'border-dark-600/30 bg-dark-800/30 hover:border-dark-500/40'
                  }`}
                >
                  <h4 className="text-lg font-semibold text-dark-50">{plan.name}</h4>
                  <div className="mt-2">
                    <span className="text-2xl font-bold text-dark-50">{plan.price}</span>
                    {plan.period && <span className="text-dark-400 text-sm">{plan.period}</span>}
                  </div>
                  <p className="text-sm text-brand-400 mt-1">{plan.ops}</p>
                  <ul className="mt-4 space-y-1.5">
                    {plan.features.map((f) => (
                      <li key={f} className="text-sm text-dark-300 flex items-start gap-2">
                        <span className="text-emerald-400 mt-0.5">&#10003;</span>
                        {f}
                      </li>
                    ))}
                  </ul>
                  <div className="mt-5">
                    {isCurrent ? (
                      <span className="text-sm text-brand-400 font-medium">Current plan</span>
                    ) : plan.id !== 'free' ? (
                      <button
                        onClick={() => onUpgrade(plan.id)}
                        disabled={actionBusy}
                        className="btn-primary !px-4 !py-2 text-sm w-full disabled:opacity-50"
                      >
                        {actionBusy ? 'Redirecting...' : 'Upgrade'}
                      </button>
                    ) : null}
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      )}

      {/* Invoices */}
      {invoices.length > 0 && (
        <section className="card">
          <h3 className="text-lg font-semibold text-dark-50 mb-4">Invoices</h3>
          <div className="space-y-2">
            {invoices.map((inv) => (
              <div
                key={inv.id}
                className="rounded-lg border border-dark-600/40 bg-dark-900/40 p-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2"
              >
                <div>
                  <p className="text-sm text-dark-100">
                    {formatDate(inv.created)} &mdash; {formatCents(inv.amount_due, inv.currency)}
                  </p>
                  <p className="text-xs text-dark-400">
                    Period: {formatDate(inv.period_start)} &ndash; {formatDate(inv.period_end)}
                  </p>
                </div>
                <div className="flex items-center gap-3">
                  <StatusBadge status={inv.status === 'paid' ? 'active' : inv.status} label={inv.status} />
                  {inv.hosted_invoice_url && (
                    <a
                      href={inv.hosted_invoice_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-sm text-brand-400 hover:text-brand-300"
                    >
                      View
                    </a>
                  )}
                </div>
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
