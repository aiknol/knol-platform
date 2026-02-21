'use client';

import { useEffect } from 'react';

export default function AppError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error('App error boundary caught:', error);
  }, [error]);

  return (
    <div className="min-h-screen flex items-center justify-center px-4">
      <div className="max-w-md w-full text-center space-y-6">
        <div className="rounded-xl border border-red-500/30 bg-red-500/10 p-8">
          <h2 className="text-xl font-semibold text-dark-50 mb-2">
            Something went wrong
          </h2>
          <p className="text-sm text-dark-300 mb-6">
            An unexpected error occurred. You can try again or return to the
            dashboard.
          </p>
          <button
            onClick={reset}
            className="btn-primary !py-2.5"
          >
            Try again
          </button>
        </div>
      </div>
    </div>
  );
}
