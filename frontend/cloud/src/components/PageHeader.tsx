'use client';

import { ReactNode } from 'react';

interface PageHeaderProps {
  title: string;
  description?: string;
  action?: ReactNode;
}

export default function PageHeader({ title, description, action }: PageHeaderProps) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3 mb-6">
      <div className="min-w-0">
        <h2 className="text-xl sm:text-2xl font-semibold text-dark-50">{title}</h2>
        {description && <p className="text-sm text-dark-400 mt-1">{description}</p>}
      </div>
      {action}
    </div>
  );
}
