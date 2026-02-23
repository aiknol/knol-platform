'use client';

import { ReactNode } from 'react';

interface EmptyStateProps {
  message: string;
  action?: ReactNode;
}

export default function EmptyState({ message, action }: EmptyStateProps) {
  return (
    <div className="text-center py-10">
      <p className="text-dark-400 text-sm">{message}</p>
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
