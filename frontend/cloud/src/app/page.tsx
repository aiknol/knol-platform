'use client';

import { FormEvent, useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { appAuthAPI, getAppAuthUser } from '@/features/app/api';

function CrownIcon({ className = 'w-10 h-10' }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M6 30L12 14L20 20L28 14L34 30H6Z" fill="#F0B429" opacity="0.9" />
      <circle cx="6" cy="14" r="2.5" fill="#F7CC5F" />
      <circle cx="20" cy="9" r="2.5" fill="#F7CC5F" />
      <circle cx="34" cy="14" r="2.5" fill="#F7CC5F" />
      <rect x="5" y="30" width="30" height="3.5" rx="1.5" fill="#F0B429" />
    </svg>
  );
}

function PipelineStep({ icon, label, delay }: { icon: React.ReactNode; label: string; delay: string }) {
  return (
    <div className="flex flex-col items-center gap-2">
      <div
        className="w-12 h-12 rounded-xl border border-brand-400/30 bg-brand-400/5 flex items-center justify-center"
        style={{ animationDelay: delay }}
      >
        {icon}
      </div>
      <span className="text-xs text-dark-300 font-medium tracking-wide">{label}</span>
    </div>
  );
}

function PipelineConnector({ delay }: { delay: string }) {
  return (
    <div className="flex items-center gap-1 mt-[-20px]">
      <div className="relative w-8 h-[2px] bg-dark-600/50 overflow-hidden">
        <div
          className="absolute top-0 left-0 h-full w-3 bg-gradient-to-r from-transparent via-brand-400 to-transparent rounded-full"
          style={{
            animation: 'dot-travel 2s ease-in-out infinite',
            animationDelay: delay,
          }}
        />
      </div>
      <svg width="6" height="8" viewBox="0 0 6 8" fill="none" className="text-dark-500 mt-[0.5px]">
        <path d="M1 1L5 4L1 7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </div>
  );
}

function RobotMascot() {
  return (
    <svg width="100" height="120" viewBox="0 0 100 120" fill="none" xmlns="http://www.w3.org/2000/svg" className="opacity-80">
      <line x1="50" y1="8" x2="50" y2="22" stroke="#F0B429" strokeWidth="2.5" strokeLinecap="round" />
      <circle cx="50" cy="6" r="4" fill="#F7CC5F" />
      <rect x="22" y="22" width="56" height="40" rx="10" fill="#1A1A1D" stroke="#F0B429" strokeWidth="1.5" />
      <circle cx="38" cy="42" r="7" fill="#F0B429" opacity="0.9" />
      <circle cx="62" cy="42" r="7" fill="#F0B429" opacity="0.9" />
      <circle cx="38" cy="42" r="3.5" fill="#0A0A0B" />
      <circle cx="62" cy="42" r="3.5" fill="#0A0A0B" />
      <rect x="42" y="50" width="16" height="3" rx="1.5" fill="#F0B429" opacity="0.4" />
      <rect x="18" y="66" width="64" height="42" rx="8" fill="#1A1A1D" stroke="#F0B429" strokeWidth="1.5" />
      <rect x="35" y="74" width="30" height="16" rx="3" fill="#0A0A0B" stroke="#F7CC5F" strokeWidth="1" />
      <circle cx="43" cy="82" r="2.5" fill="#F0B429" />
      <circle cx="50" cy="82" r="2.5" fill="#F7CC5F" />
      <circle cx="57" cy="82" r="2.5" fill="#F0B429" />
      <rect x="4" y="72" width="10" height="30" rx="5" fill="#1A1A1D" stroke="#F0B429" strokeWidth="1.5" />
      <rect x="86" y="72" width="10" height="30" rx="5" fill="#1A1A1D" stroke="#F0B429" strokeWidth="1.5" />
      <rect x="30" y="110" width="14" height="8" rx="3" fill="#1A1A1D" stroke="#F0B429" strokeWidth="1.5" />
      <rect x="56" y="110" width="14" height="8" rx="3" fill="#1A1A1D" stroke="#F0B429" strokeWidth="1.5" />
    </svg>
  );
}

const DOAIDE_PRODUCTS = [
  { name: '409', href: 'https://409.doaide.com', desc: 'Status Pages' },
  { name: 'Cortex', href: 'https://cortex.doaide.com', desc: 'Agent Memory' },
  { name: 'GoSumo', href: 'https://gosumo.doaide.com', desc: 'Networking' },
];

export default function LandingPage() {
  const router = useRouter();
  const [tab, setTab] = useState<'signin' | 'signup'>('signin');
  const [ready, setReady] = useState(false);

  const [loginEmail, setLoginEmail] = useState('');
  const [loginPassword, setLoginPassword] = useState('');

  const [companyName, setCompanyName] = useState('');
  const [fullName, setFullName] = useState('');
  const [signupEmail, setSignupEmail] = useState('');
  const [signupPassword, setSignupPassword] = useState('');

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    const user = getAppAuthUser();
    if (user) {
      router.push('/playground');
    } else {
      setReady(true);
    }
  }, [router]);

  const validatePassword = (pw: string): string | null => {
    if (pw.length < 12) return 'Password must be at least 12 characters.';
    if (!/[A-Z]/.test(pw)) return 'Must include an uppercase letter.';
    if (!/[a-z]/.test(pw)) return 'Must include a lowercase letter.';
    if (!/[0-9]/.test(pw)) return 'Must include a digit.';
    if (!/[^A-Za-z0-9]/.test(pw)) return 'Must include a special character.';
    return null;
  };

  const onLogin = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      await appAuthAPI.login(loginEmail, loginPassword);
      router.push('/playground');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  const onSignup = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    const pwError = validatePassword(signupPassword);
    if (pwError) {
      setError(pwError);
      return;
    }
    setLoading(true);
    try {
      await appAuthAPI.signup({
        company_name: companyName,
        full_name: fullName,
        email: signupEmail,
        password: signupPassword,
      });
      router.push('/api-keys');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Signup failed');
    } finally {
      setLoading(false);
    }
  };

  if (!ready) {
    return <div className="min-h-screen bg-[#0A0A0B]" />;
  }

  return (
    <div className="min-h-screen bg-[#0A0A0B] flex flex-col">
      <style>{`
        @keyframes dot-travel {
          0% { transform: translateX(-12px); opacity: 0; }
          20% { opacity: 1; }
          80% { opacity: 1; }
          100% { transform: translateX(32px); opacity: 0; }
        }
        @keyframes fade-up {
          from { opacity: 0; transform: translateY(12px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .animate-fade-up {
          animation: fade-up 0.6s ease-out both;
        }
      `}</style>

      {/* Main split layout */}
      <div className="flex-1 flex flex-col md:flex-row">

        {/* Left side — info (55%) */}
        <div className="md:w-[55%] px-6 md:px-12 lg:px-20 py-10 md:py-0 flex flex-col justify-center relative overflow-hidden">
          {/* Background glow */}
          <div className="absolute top-1/3 left-1/4 w-[400px] h-[400px] bg-brand-400/5 rounded-full blur-[120px] pointer-events-none" />

          <div className="relative z-10 max-w-lg">
            {/* Logo */}
            <div className="flex items-center gap-3 mb-10 md:mb-14 animate-fade-up">
              <CrownIcon className="w-10 h-10" />
              <h1 className="text-2xl tracking-tight">
                <span className="text-white font-medium">DoAide</span>{' '}
                <span className="text-brand-400 font-serif italic">Cortex</span>
              </h1>
            </div>

            {/* Headline */}
            <h2
              className="text-4xl md:text-5xl lg:text-[3.5rem] font-serif text-white leading-[1.15] tracking-tight mb-5 animate-fade-up"
              style={{ animationDelay: '0.1s' }}
            >
              Agent memory,
              <br />
              organized.
            </h2>

            <p
              className="text-base md:text-lg text-dark-400 leading-relaxed mb-10 md:mb-14 max-w-md animate-fade-up"
              style={{ animationDelay: '0.2s' }}
            >
              <span className="font-mono">Knowledge management for AI agents</span> &mdash; store, retrieve, connect, evolve.
            </p>

            {/* Pipeline — hidden on mobile */}
            <div
              className="hidden md:flex items-start gap-2 mb-14 animate-fade-up"
              style={{ animationDelay: '0.3s' }}
            >
              <PipelineStep
                delay="0s"
                label="Store"
                icon={
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                    <ellipse cx="10" cy="5" rx="7" ry="3" stroke="#F0B429" strokeWidth="1.5" />
                    <path d="M3 5v10c0 1.66 3.13 3 7 3s7-1.34 7-3V5" stroke="#F0B429" strokeWidth="1.5" />
                    <path d="M3 10c0 1.66 3.13 3 7 3s7-1.34 7-3" stroke="#F0B429" strokeWidth="1.5" opacity="0.5" />
                  </svg>
                }
              />
              <PipelineConnector delay="0s" />
              <PipelineStep
                delay="0.1s"
                label="Retrieve"
                icon={
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                    <circle cx="9" cy="9" r="5.5" stroke="#F0B429" strokeWidth="1.5" />
                    <path d="M13 13l4 4" stroke="#F0B429" strokeWidth="1.5" strokeLinecap="round" />
                  </svg>
                }
              />
              <PipelineConnector delay="0.5s" />
              <PipelineStep
                delay="0.2s"
                label="Connect"
                icon={
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                    <circle cx="5" cy="10" r="2.5" stroke="#F0B429" strokeWidth="1.5" />
                    <circle cx="15" cy="5" r="2.5" stroke="#F0B429" strokeWidth="1.5" />
                    <circle cx="15" cy="15" r="2.5" stroke="#F0B429" strokeWidth="1.5" />
                    <path d="M7.2 9L12.5 6M7.2 11L12.5 14" stroke="#F0B429" strokeWidth="1" opacity="0.6" />
                  </svg>
                }
              />
              <PipelineConnector delay="1s" />
              <PipelineStep
                delay="0.3s"
                label="Evolve"
                icon={
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                    <path d="M10 16V6" stroke="#F0B429" strokeWidth="1.5" strokeLinecap="round" />
                    <path d="M6 10l4-4 4 4" stroke="#F0B429" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                    <path d="M4 16h12" stroke="#F0B429" strokeWidth="1.5" strokeLinecap="round" opacity="0.5" />
                  </svg>
                }
              />
            </div>

            {/* Robot mascot — hidden on mobile */}
            <div className="hidden lg:block animate-fade-up" style={{ animationDelay: '0.4s' }}>
              <RobotMascot />
            </div>
          </div>
        </div>

        {/* Right side — auth card (45%) */}
        <div className="md:w-[45%] flex items-center justify-center px-6 py-10 md:py-0 md:pr-12 lg:pr-20">
          <div className="w-full max-w-md animate-fade-up" style={{ animationDelay: '0.15s' }}>
            <div className="bg-dark-800/60 backdrop-blur-sm border border-dark-600/30 rounded-2xl p-6 md:p-8">
              {/* Tabs */}
              <div className="flex mb-6 border-b border-dark-600/40">
                <button
                  type="button"
                  onClick={() => { setTab('signin'); setError(''); }}
                  className={`flex-1 pb-3 text-sm font-medium transition-colors border-b-2 ${
                    tab === 'signin'
                      ? 'border-brand-400 text-brand-300'
                      : 'border-transparent text-dark-400 hover:text-dark-200'
                  }`}
                >
                  Sign in
                </button>
                <button
                  type="button"
                  onClick={() => { setTab('signup'); setError(''); }}
                  className={`flex-1 pb-3 text-sm font-medium transition-colors border-b-2 ${
                    tab === 'signup'
                      ? 'border-brand-400 text-brand-300'
                      : 'border-transparent text-dark-400 hover:text-dark-200'
                  }`}
                >
                  Create account
                </button>
              </div>

              {error && (
                <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
                  {error}
                </div>
              )}

              {/* Sign in form */}
              {tab === 'signin' && (
                <form onSubmit={onLogin} className="space-y-4">
                  <div>
                    <label className="block text-sm text-dark-300 mb-2" htmlFor="login-email">Email</label>
                    <input
                      id="login-email"
                      type="email"
                      required
                      maxLength={255}
                      autoComplete="email"
                      value={loginEmail}
                      onChange={(e) => setLoginEmail(e.target.value)}
                      className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2.5 text-dark-100 placeholder:text-dark-500 focus:outline-none focus:border-brand-400/50 transition-colors"
                      placeholder="you@company.com"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-dark-300 mb-2" htmlFor="login-password">Password</label>
                    <input
                      id="login-password"
                      type="password"
                      required
                      maxLength={128}
                      autoComplete="current-password"
                      value={loginPassword}
                      onChange={(e) => setLoginPassword(e.target.value)}
                      className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2.5 text-dark-100 placeholder:text-dark-500 focus:outline-none focus:border-brand-400/50 transition-colors"
                      placeholder="Your password"
                    />
                  </div>
                  <button
                    disabled={loading}
                    type="submit"
                    className="w-full py-2.5 rounded-lg font-semibold text-dark-900 bg-gradient-to-r from-brand-400 to-brand-300 hover:opacity-90 transition-opacity disabled:opacity-60"
                  >
                    {loading ? 'Signing in...' : 'Sign In'}
                  </button>
                </form>
              )}

              {/* Signup form */}
              {tab === 'signup' && (
                <form onSubmit={onSignup} className="space-y-4">
                  <div>
                    <label className="block text-sm text-dark-300 mb-2" htmlFor="signup-company">Company name</label>
                    <input
                      id="signup-company"
                      type="text"
                      required
                      maxLength={100}
                      autoComplete="organization"
                      value={companyName}
                      onChange={(e) => setCompanyName(e.target.value)}
                      className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2.5 text-dark-100 placeholder:text-dark-500 focus:outline-none focus:border-brand-400/50 transition-colors"
                      placeholder="Acme Inc"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-dark-300 mb-2" htmlFor="signup-name">Full name</label>
                    <input
                      id="signup-name"
                      type="text"
                      required
                      maxLength={100}
                      autoComplete="name"
                      value={fullName}
                      onChange={(e) => setFullName(e.target.value)}
                      className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2.5 text-dark-100 placeholder:text-dark-500 focus:outline-none focus:border-brand-400/50 transition-colors"
                      placeholder="Jane Doe"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-dark-300 mb-2" htmlFor="signup-email">Work email</label>
                    <input
                      id="signup-email"
                      type="email"
                      required
                      maxLength={255}
                      autoComplete="email"
                      value={signupEmail}
                      onChange={(e) => setSignupEmail(e.target.value)}
                      className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2.5 text-dark-100 placeholder:text-dark-500 focus:outline-none focus:border-brand-400/50 transition-colors"
                      placeholder="jane@acme.com"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-dark-300 mb-2" htmlFor="signup-password">Password</label>
                    <input
                      id="signup-password"
                      type="password"
                      minLength={12}
                      maxLength={128}
                      required
                      autoComplete="new-password"
                      value={signupPassword}
                      onChange={(e) => setSignupPassword(e.target.value)}
                      className="w-full rounded-lg border border-dark-600/40 bg-dark-900/70 px-3 py-2.5 text-dark-100 placeholder:text-dark-500 focus:outline-none focus:border-brand-400/50 transition-colors"
                      placeholder="At least 12 characters"
                    />
                    <p className="text-xs text-dark-500 mt-1">
                      Upper, lower, digit, and special character required.
                    </p>
                  </div>
                  <button
                    disabled={loading}
                    type="submit"
                    className="w-full py-2.5 rounded-lg font-semibold text-dark-900 bg-gradient-to-r from-brand-400 to-brand-300 hover:opacity-90 transition-opacity disabled:opacity-60"
                  >
                    {loading ? 'Creating workspace...' : 'Create Free Workspace'}
                  </button>
                </form>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Footer */}
      <footer className="border-t border-dark-600/20 px-6 py-8">
        <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-6">
          <div className="flex items-center gap-2">
            <CrownIcon className="w-5 h-5" />
            <span className="text-sm text-dark-400">
              &copy; {new Date().getFullYear()} DoAide
            </span>
          </div>

          <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
            {DOAIDE_PRODUCTS.map((p) => (
              <a
                key={p.name}
                href={p.href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-dark-400 hover:text-brand-300 transition-colors"
              >
                {p.name}
              </a>
            ))}
            <Link href="/playground" className="text-sm text-dark-400 hover:text-brand-300 transition-colors">
              Playground
            </Link>
            <a
              href="https://docs.cortex.doaide.com"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-dark-400 hover:text-brand-300 transition-colors"
            >
              Docs
            </a>
            <a
              href="https://github.com/doaide/cortex"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-dark-400 hover:text-brand-300 transition-colors"
            >
              GitHub
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}
