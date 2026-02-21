'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import {
  AppUser,
  TenantProfile,
  appAuthAPI,
  clearAppAuthSession,
  getAppAuthUser,
  getAppTenant,
} from '@/features/app/api';
import KnolLogo from '@/components/KnolLogo';

export default function AppShell({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const [user, setUser] = useState<AppUser | null>(null);
  const [tenant, setTenant] = useState<TenantProfile | null>(null);
  const [loading, setLoading] = useState(true);

  const isPublicPage =
    pathname === '/login' ||
    pathname === '/login/' ||
    pathname === '/signup' ||
    pathname === '/signup/';

  useEffect(() => {
    if (isPublicPage) {
      setLoading(false);
      return;
    }

    const cachedUser = getAppAuthUser();
    const cachedTenant = getAppTenant();
    if (cachedUser) setUser(cachedUser);
    if (cachedTenant) setTenant(cachedTenant);

    const check = async () => {
      try {
        const me = await appAuthAPI.me();
        setUser(me.user);
        setTenant(me.tenant);
        setLoading(false);
      } catch {
        clearAppAuthSession();
        router.push('/login');
      }
    };

    check().catch(() => {
      clearAppAuthSession();
      router.push('/login');
    });
  }, [isPublicPage, router]);

  const onLogout = async () => {
    try {
      await appAuthAPI.logout();
    } catch {
      // Keep logout idempotent client-side.
    }
    clearAppAuthSession();
    router.push('/login');
  };

  if (isPublicPage) {
    return <>{children}</>;
  }

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-dark-300">Loading workspace...</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen">
      <header className="border-b border-dark-600/40 bg-dark-900/70 backdrop-blur-sm">
        <div className="max-w-6xl mx-auto px-4 py-4 flex items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <KnolLogo className="w-9 h-9" label="Knol logo" />
            <div>
              <p className="text-sm text-dark-400">Knol Cloud</p>
              <h1 className="text-lg font-semibold text-dark-100">{tenant?.name || 'Workspace'}</h1>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Link href="/dashboard" className="text-sm text-brand-400 hover:text-brand-300 transition-colors">
              Dashboard
            </Link>
            <div className="text-right">
              <p className="text-sm text-dark-200">{user?.email}</p>
              <p className="text-xs text-dark-500 capitalize">{user?.role}</p>
            </div>
            <button
              onClick={onLogout}
              className="px-3 py-2 text-sm rounded-lg border border-dark-600/50 text-dark-200 hover:bg-dark-800/60"
            >
              Logout
            </button>
          </div>
        </div>
      </header>
      <main className="max-w-6xl mx-auto px-4 py-8">{children}</main>
    </div>
  );
}
