'use client';

import Link from 'next/link';
import { FormEvent, useState } from 'react';
import { useRouter } from 'next/navigation';
import { appAuthAPI } from '@/features/app/api';
import KnolLogo from '@/components/KnolLogo';

export default function AppSignupPage() {
  const router = useRouter();
  const [companyName, setCompanyName] = useState('');
  const [fullName, setFullName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const validatePassword = (pw: string): string | null => {
    if (pw.length < 12) return 'Password must be at least 12 characters.';
    if (!/[A-Z]/.test(pw)) return 'Password must include an uppercase letter.';
    if (!/[a-z]/.test(pw)) return 'Password must include a lowercase letter.';
    if (!/[0-9]/.test(pw)) return 'Password must include a digit.';
    if (!/[^A-Za-z0-9]/.test(pw)) return 'Password must include a special character.';
    return null;
  };

  const onSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');

    const pwError = validatePassword(password);
    if (pwError) {
      setError(pwError);
      return;
    }

    setLoading(true);

    try {
      await appAuthAPI.signup({
        company_name: companyName,
        full_name: fullName,
        email,
        password,
      });
      router.push('/dashboard');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Signup failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center px-4 py-10">
      <div className="w-full max-w-lg card">
        <div className="mb-6 flex items-center gap-3">
          <KnolLogo className="w-11 h-11" label="Knol logo" />
          <div>
            <p className="text-xs uppercase tracking-[0.12em] text-dark-500">Knol Cloud</p>
            <p className="text-sm text-dark-300">Tenant Workspace</p>
          </div>
        </div>
        <h2 className="text-2xl font-semibold text-dark-50 mb-2">Create Your Free Workspace</h2>
        <p className="text-sm text-dark-400 mb-6">
          Start with one tenant and API access in under a minute.
        </p>

        {error && (
          <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
            {error}
          </div>
        )}

        <form onSubmit={onSubmit} className="space-y-4">
          <div>
            <label className="block text-sm text-dark-300 mb-2" htmlFor="company">Company name</label>
            <input
              id="company"
              type="text"
              required
              maxLength={100}
              autoComplete="organization"
              value={companyName}
              onChange={(e) => setCompanyName(e.target.value)}
              className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              placeholder="Acme Inc"
            />
          </div>

          <div>
            <label className="block text-sm text-dark-300 mb-2" htmlFor="full_name">Full name</label>
            <input
              id="full_name"
              type="text"
              required
              maxLength={100}
              autoComplete="name"
              value={fullName}
              onChange={(e) => setFullName(e.target.value)}
              className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              placeholder="Jane Doe"
            />
          </div>

          <div>
            <label className="block text-sm text-dark-300 mb-2" htmlFor="email">Work email</label>
            <input
              id="email"
              type="email"
              required
              maxLength={255}
              autoComplete="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              placeholder="jane@acme.com"
            />
          </div>

          <div>
            <label className="block text-sm text-dark-300 mb-2" htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              minLength={12}
              maxLength={128}
              required
              autoComplete="new-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2 text-dark-100"
              placeholder="At least 12 characters"
            />
            <p className="text-xs text-dark-500 mt-1">
              Must include uppercase, lowercase, digit, and special character.
            </p>
          </div>

          <button disabled={loading} type="submit" className="w-full btn-primary !py-2.5 disabled:opacity-60">
            {loading ? 'Creating workspace...' : 'Create Free Workspace'}
          </button>
        </form>

        <p className="mt-6 text-sm text-dark-400">
          Already registered?{' '}
          <Link href="/login" className="text-brand-400 hover:text-brand-300">
            Sign in
          </Link>
        </p>
      </div>
    </div>
  );
}
