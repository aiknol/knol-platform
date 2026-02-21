'use client';

import Link from 'next/link';
import { FormEvent, useState } from 'react';
import { useRouter } from 'next/navigation';
import { appAuthAPI } from '@/features/app/api';
import KnolLogo from '@/components/KnolLogo';

export default function AppLoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const onSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      await appAuthAPI.login(email, password);
      router.push('/dashboard');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center px-4 py-10">
      <div className="w-full max-w-md card">
        <div className="mb-6 flex items-center gap-3">
          <KnolLogo className="w-11 h-11" label="Knol logo" />
          <div>
            <p className="text-xs uppercase tracking-[0.12em] text-dark-500">Knol Cloud</p>
            <p className="text-sm text-dark-300">Tenant Workspace</p>
          </div>
        </div>
        <h2 className="text-2xl font-semibold text-dark-50 mb-2">Sign in to Knol Cloud</h2>
        <p className="text-sm text-dark-400 mb-6">Use your workspace credentials to access memory services.</p>

        {error && (
          <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
            {error}
          </div>
        )}

        <form onSubmit={onSubmit} className="space-y-4">
          <div>
            <label className="block text-sm text-dark-300 mb-2" htmlFor="email">Email</label>
            <input
              id="email"
              type="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              placeholder="you@company.com"
            />
          </div>
          <div>
            <label className="block text-sm text-dark-300 mb-2" htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              placeholder="Your password"
            />
          </div>
          <button disabled={loading} type="submit" className="w-full btn-primary !py-2.5 disabled:opacity-60">
            {loading ? 'Signing in...' : 'Sign In'}
          </button>
        </form>

        <p className="mt-6 text-sm text-dark-400">
          New company?{' '}
          <Link href="/signup" className="text-brand-400 hover:text-brand-300">
            Create a free workspace
          </Link>
        </p>
      </div>
    </div>
  );
}
