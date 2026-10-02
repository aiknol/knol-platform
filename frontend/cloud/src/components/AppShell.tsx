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
import CortexLogo from '@/components/CortexLogo';
import { APP_NAV_ITEMS } from '@/config/site';

export default function AppShell({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const [user, setUser] = useState<AppUser | null>(null);
  const [tenant, setTenant] = useState<TenantProfile | null>(null);
  const [loading, setLoading] = useState(true);

  const isAuthPage =
    pathname === '/' ||
    pathname === '/login' ||
    pathname === '/login/' ||
    pathname === '/signup' ||
    pathname === '/signup/';

  // Playground is accessible without login, but should show the nav when logged in.
  const isPlayground =
    pathname === '/playground' || pathname === '/playground/';

  const canManage = user?.role === 'owner' || user?.role === 'admin';

  useEffect(() => {
    if (isAuthPage) {
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
        if (isPlayground) {
          // Playground is accessible without login — just clear the user and stop loading.
          setUser(null);
          setTenant(null);
          setLoading(false);
        } else {
          router.push('/login');
        }
      }
    };

    check().catch(() => {
      clearAppAuthSession();
      if (isPlayground) {
        setUser(null);
        setTenant(null);
        setLoading(false);
      } else {
        router.push('/login');
      }
    });
  }, [isAuthPage, isPlayground, router]);

  const onLogout = async () => {
    try {
      await appAuthAPI.logout();
    } catch {
      // Keep logout idempotent client-side.
    }
    clearAppAuthSession();
    router.push('/login');
  };

  // Auth pages (login/signup) never show the nav shell.
  if (isAuthPage) {
    return <>{children}</>;
  }

  // Playground: show the full shell if the user is logged in, otherwise render standalone.
  if (isPlayground && !user) {
    return <>{children}</>;
  }

  // For the playground, don't block rendering while verifying auth — show the
  // shell immediately with whatever cached user data we have. For all other
  // protected pages, keep the loading gate so we don't flash content.
  if (loading && !isPlayground) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-dark-300">Loading workspace...</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="border-b border-dark-600/40 bg-dark-900/70 backdrop-blur-sm">
        <div className="max-w-6xl mx-auto px-4 py-3 sm:py-4 flex items-center justify-between gap-3 sm:gap-4">
          <div className="flex items-center gap-2 sm:gap-3 min-w-0">
            <CortexLogo className="w-8 h-8 sm:w-9 sm:h-9 shrink-0" label="Cortex logo" />
            <div className="min-w-0">
              <p className="text-xs sm:text-sm text-dark-400">DoAide Cortex</p>
              <h1 className="text-base sm:text-lg font-semibold text-dark-100 truncate">{tenant?.name || 'Workspace'}</h1>
            </div>
          </div>
          <div className="flex items-center gap-2 sm:gap-3 shrink-0">
            <div className="text-right hidden md:block">
              <p className="text-sm text-dark-200 truncate max-w-[180px]">{user?.email}</p>
              <p className="text-xs text-dark-500 capitalize">{user?.role}</p>
            </div>
            <button
              onClick={onLogout}
              className="px-2.5 sm:px-3 py-1.5 sm:py-2 text-sm rounded-lg border border-dark-600/50 text-dark-200 hover:bg-dark-800/60"
            >
              Logout
            </button>
          </div>
        </div>
      </header>

      {/* Tab navigation */}
      <nav className="border-b border-dark-600/30 bg-dark-900/50">
        <div className="max-w-6xl mx-auto px-4 flex gap-0 overflow-x-auto scrollbar-none">
          {APP_NAV_ITEMS
            .filter((item) => !item.adminOnly || canManage)
            .map((item) => {
              const active =
                pathname === item.href ||
                pathname === `${item.href}/` ||
                (item.href !== '/dashboard' && pathname.startsWith(item.href));
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`px-4 py-3 text-sm whitespace-nowrap border-b-2 transition-colors ${
                    active
                      ? 'border-brand-500 text-brand-300'
                      : 'border-transparent text-dark-400 hover:text-dark-200'
                  }`}
                >
                  {item.label}
                </Link>
              );
            })}
        </div>
      </nav>

      <main className="max-w-6xl mx-auto px-4 py-6 sm:py-8">{children}</main>
    </div>
  );
}
