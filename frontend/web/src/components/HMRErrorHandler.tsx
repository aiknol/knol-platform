'use client';

import { useEffect } from 'react';

export default function HMRErrorHandler() {
  useEffect(() => {
    // Suppress HMR/extension messaging errors
    const handleError = (event: ErrorEvent) => {
      if (
        event.message &&
        event.message.includes('A listener indicated an asynchronous response')
      ) {
        event.preventDefault();
        return true;
      }
    };

    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      if (
        event.reason &&
        (typeof event.reason === 'string'
          ? event.reason.includes('A listener indicated an asynchronous response')
          : event.reason?.message?.includes(
              'A listener indicated an asynchronous response'
            ))
      ) {
        event.preventDefault();
        return true;
      }
    };

    window.addEventListener('error', handleError);
    window.addEventListener('unhandledrejection', handleUnhandledRejection);

    return () => {
      window.removeEventListener('error', handleError);
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
    };
  }, []);

  return null;
}
