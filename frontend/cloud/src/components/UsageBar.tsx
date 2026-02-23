'use client';

interface UsageBarProps {
  used: number;
  limit: number | null;
  className?: string;
  showLabel?: boolean;
}

export default function UsageBar({ used, limit, className = '', showLabel = true }: UsageBarProps) {
  if (limit === null || limit === 0) {
    return (
      <div className={className}>
        {showLabel && <p className="text-xs text-dark-400 mb-1">Unlimited plan</p>}
        <div className="h-2 rounded-full bg-dark-700 overflow-hidden">
          <div className="h-full rounded-full bg-brand-500/40" style={{ width: '10%' }} />
        </div>
      </div>
    );
  }

  const pct = Math.min((used / limit) * 100, 100);
  const color =
    pct >= 80 ? 'bg-red-500' : pct >= 50 ? 'bg-amber-500' : 'bg-emerald-500';

  return (
    <div className={className}>
      {showLabel && (
        <div className="flex items-center justify-between text-xs text-dark-400 mb-1">
          <span>{used.toLocaleString()} / {limit.toLocaleString()} ops</span>
          <span>{pct.toFixed(0)}%</span>
        </div>
      )}
      <div className="h-2 rounded-full bg-dark-700 overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-500 ${color}`}
          style={{ width: `${Math.max(pct, 1)}%` }}
        />
      </div>
    </div>
  );
}
