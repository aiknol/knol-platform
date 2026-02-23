'use client';

import type { UsageHistoryItem } from '@/features/app/api/types';

interface UsageChartProps {
  data: UsageHistoryItem[];
  className?: string;
}

const MONTH_LABELS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

function formatMonth(month: string): string {
  const parts = month.split('-');
  if (parts.length === 2) {
    const idx = parseInt(parts[1], 10) - 1;
    if (idx >= 0 && idx < 12) return MONTH_LABELS[idx];
  }
  return month;
}

function formatOps(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}K`;
  return n.toString();
}

export default function UsageChart({ data, className = '' }: UsageChartProps) {
  if (data.length === 0) {
    return (
      <div className={`text-center py-8 text-dark-400 text-sm ${className}`}>
        No usage history yet
      </div>
    );
  }

  // Sort ascending by month
  const sorted = [...data].sort((a, b) => a.month.localeCompare(b.month));
  const maxOps = Math.max(...sorted.map((d) => d.ops_count), 1);
  const maxLimit = sorted.reduce<number | null>((acc, d) => {
    if (d.usage_limit !== null) {
      return acc === null ? d.usage_limit : Math.max(acc, d.usage_limit);
    }
    return acc;
  }, null);
  const ceiling = maxLimit !== null ? Math.max(maxOps, maxLimit) : maxOps;

  const barWidth = 32;
  const gap = 8;
  const chartHeight = 140;
  const labelHeight = 24;
  const topPadding = 20;
  const totalWidth = sorted.length * (barWidth + gap) - gap + 40;
  const totalHeight = chartHeight + labelHeight + topPadding;

  return (
    <div className={`overflow-x-auto ${className}`}>
      <svg
        viewBox={`0 0 ${totalWidth} ${totalHeight}`}
        className="w-full min-w-[300px]"
        preserveAspectRatio="xMidYMid meet"
      >
        {/* Limit line */}
        {maxLimit !== null && (
          <>
            <line
              x1={16}
              y1={topPadding + chartHeight - (maxLimit / ceiling) * chartHeight}
              x2={totalWidth - 16}
              y2={topPadding + chartHeight - (maxLimit / ceiling) * chartHeight}
              stroke="#71717A"
              strokeWidth={1}
              strokeDasharray="4 3"
            />
            <text
              x={totalWidth - 14}
              y={topPadding + chartHeight - (maxLimit / ceiling) * chartHeight - 4}
              fill="#71717A"
              fontSize={9}
              textAnchor="end"
            >
              limit
            </text>
          </>
        )}

        {sorted.map((item, i) => {
          const barHeight = ceiling > 0 ? (item.ops_count / ceiling) * chartHeight : 0;
          const x = 20 + i * (barWidth + gap);
          const y = topPadding + chartHeight - barHeight;

          return (
            <g key={item.month}>
              {/* Bar */}
              <rect
                x={x}
                y={y}
                width={barWidth}
                height={Math.max(barHeight, 1)}
                rx={4}
                fill="url(#barGradient)"
                className="opacity-80 hover:opacity-100 transition-opacity"
              />
              {/* Value label on hover area */}
              <text
                x={x + barWidth / 2}
                y={y - 4}
                fill="#A1A1AA"
                fontSize={9}
                textAnchor="middle"
              >
                {formatOps(item.ops_count)}
              </text>
              {/* Month label */}
              <text
                x={x + barWidth / 2}
                y={topPadding + chartHeight + 16}
                fill="#71717A"
                fontSize={10}
                textAnchor="middle"
              >
                {formatMonth(item.month)}
              </text>
            </g>
          );
        })}

        <defs>
          <linearGradient id="barGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#8B73E6" />
            <stop offset="100%" stopColor="#6E56CF" />
          </linearGradient>
        </defs>
      </svg>
    </div>
  );
}
