'use client';

const COLORS: Record<string, string> = {
  active: 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300',
  trialing: 'border-blue-500/30 bg-blue-500/10 text-blue-300',
  canceled: 'border-amber-500/30 bg-amber-500/10 text-amber-300',
  past_due: 'border-red-500/30 bg-red-500/10 text-red-300',
  unpaid: 'border-red-500/30 bg-red-500/10 text-red-300',
  pending: 'border-blue-500/30 bg-blue-500/10 text-blue-300',
  accepted: 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300',
  revoked: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  expired: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  none: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  enabled: 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300',
  disabled: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  owner: 'border-brand-500/30 bg-brand-500/10 text-brand-300',
  admin: 'border-amber-500/30 bg-amber-500/10 text-amber-300',
  developer: 'border-blue-500/30 bg-blue-500/10 text-blue-300',
  read_only: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  viewer: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  // Subscription plan names
  free: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  builder: 'border-blue-500/30 bg-blue-500/10 text-blue-300',
  growth: 'border-brand-500/30 bg-brand-500/10 text-brand-300',
  enterprise: 'border-amber-500/30 bg-amber-500/10 text-amber-300',
  // Invoice / payment statuses
  paid: 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300',
  open: 'border-blue-500/30 bg-blue-500/10 text-blue-300',
  draft: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
  void: 'border-dark-500/30 bg-dark-500/10 text-dark-400',
};

const DEFAULT = 'border-dark-500/30 bg-dark-500/10 text-dark-400';

export default function StatusBadge({ status, label }: { status: string; label?: string }) {
  const display = label ?? status.replace(/_/g, ' ');
  return (
    <span className={`inline-flex items-center px-2 py-0.5 text-xs font-medium rounded-full border ${COLORS[status] ?? DEFAULT}`}>
      {display}
    </span>
  );
}
