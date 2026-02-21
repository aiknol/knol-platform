'use client';

import { ReactNode } from 'react';

// ── Page shell (title + optional action button) ─────────────────

interface PageHeaderProps {
  title: string;
  description?: string;
  action?: ReactNode;
}

export function PageHeader({ title, description, action }: PageHeaderProps) {
  return (
    <div className="flex items-start justify-between mb-8">
      <div>
        <h1 className="text-2xl font-bold text-dark-50">{title}</h1>
        {description && <p className="text-dark-400 text-sm mt-1">{description}</p>}
      </div>
      {action}
    </div>
  );
}

// ── Loading state ───────────────────────────────────────────────

export function Loading({ message = 'Loading...' }: { message?: string }) {
  return (
    <div className="flex items-center justify-center py-20">
      <div className="text-dark-400">{message}</div>
    </div>
  );
}

// ── Error banner ────────────────────────────────────────────────

export function ErrorBanner({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg flex items-center justify-between">
      <p className="text-red-400 text-sm">{message}</p>
      {onRetry && (
        <button onClick={onRetry} className="text-red-400 hover:text-red-300 text-sm underline ml-4">
          Retry
        </button>
      )}
    </div>
  );
}

// ── Empty state ─────────────────────────────────────────────────

export function EmptyState({ message = 'No data found' }: { message?: string }) {
  return (
    <div className="text-center py-12 text-dark-400">
      <p>{message}</p>
    </div>
  );
}

// ── Styled table ────────────────────────────────────────────────

interface TableColumn<T> {
  header: string;
  accessor: keyof T | ((row: T) => ReactNode);
  className?: string;
}

interface DataTableProps<T> {
  columns: TableColumn<T>[];
  data: T[];
  rowKey: (row: T) => string;
  emptyMessage?: string;
}

export function DataTable<T>({ columns, data, rowKey, emptyMessage }: DataTableProps<T>) {
  if (data.length === 0) return <EmptyState message={emptyMessage} />;

  return (
    <div className="overflow-x-auto border border-dark-600/30 rounded-xl">
      <table className="w-full">
        <thead>
          <tr className="border-b border-dark-600/30 bg-dark-800/50">
            {columns.map((col) => (
              <th key={col.header} className={`text-left px-4 py-3 text-xs font-semibold text-dark-400 uppercase tracking-wider ${col.className || ''}`}>
                {col.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.map((row) => (
            <tr key={rowKey(row)} className="border-b border-dark-600/20 hover:bg-dark-700/20 transition-colors">
              {columns.map((col) => (
                <td key={col.header} className={`px-4 py-3 text-sm text-dark-200 ${col.className || ''}`}>
                  {typeof col.accessor === 'function' ? col.accessor(row) : (row[col.accessor] as ReactNode)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// ── Action button variants ──────────────────────────────────────

interface ButtonProps {
  onClick?: () => void;
  disabled?: boolean;
  children: ReactNode;
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md';
  type?: 'button' | 'submit';
  className?: string;
}

const BUTTON_STYLES = {
  primary: 'bg-brand-500 hover:bg-brand-600 text-white',
  secondary: 'bg-dark-700/30 hover:bg-dark-700/50 text-dark-300 hover:text-dark-100',
  danger: 'bg-red-500/10 hover:bg-red-500/20 text-red-400 hover:text-red-300 border border-red-500/20',
  ghost: 'text-dark-400 hover:text-dark-200 hover:bg-dark-700/30',
} as const;

const SIZE_STYLES = {
  sm: 'px-3 py-1.5 text-xs',
  md: 'px-4 py-2 text-sm',
} as const;

export function Button({ onClick, disabled, children, variant = 'primary', size = 'md', type = 'button', className = '' }: ButtonProps) {
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={`rounded-lg font-medium transition-colors disabled:opacity-50 ${BUTTON_STYLES[variant]} ${SIZE_STYLES[size]} ${className}`}
    >
      {children}
    </button>
  );
}

// ── Status badge ────────────────────────────────────────────────

export function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    up: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    healthy: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    down: 'bg-red-500/10 text-red-400 border-red-500/20',
    unreachable: 'bg-red-500/10 text-red-400 border-red-500/20',
    degraded: 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20',
    unhealthy: 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20',
    enabled: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    disabled: 'bg-dark-600/30 text-dark-400 border-dark-600/30',
  };
  return (
    <span className={`inline-block px-2 py-0.5 text-xs font-medium rounded-full border ${colors[status] || colors.disabled}`}>
      {status}
    </span>
  );
}

// ── Card wrapper ────────────────────────────────────────────────

export function AdminCard({ children, className = '' }: { children: ReactNode; className?: string }) {
  return (
    <div className={`bg-dark-800/30 border border-dark-600/30 rounded-xl p-6 ${className}`}>
      {children}
    </div>
  );
}

// ── Stat card (dashboard) ───────────────────────────────────────

export function StatCard({ label, value, icon }: { label: string; value: string | number; icon?: string }) {
  return (
    <AdminCard>
      <div className="flex items-center justify-between">
        <div>
          <p className="text-dark-400 text-sm">{label}</p>
          <p className="text-2xl font-bold text-dark-50 mt-1">{value}</p>
        </div>
        {icon && <span className="text-3xl">{icon}</span>}
      </div>
    </AdminCard>
  );
}
