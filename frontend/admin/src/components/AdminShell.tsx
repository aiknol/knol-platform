'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import Link from 'next/link';
import { authAPI, getAuthUser, clearAuthSession, setAuthUser } from '@/features/admin/api';
import { ADMIN_NAV_ITEMS, SITE } from '@/config';
import { GlobalSearch } from '@/features/admin/global-search';
import KnolLogo from '@/components/KnolLogo';

interface AdminUser {
  email: string;
  role: string;
}

export default function AdminShell({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const [user, setUser] = useState<AdminUser | null>(null);
  const [loading, setLoading] = useState(true);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const isLoginPage = pathname === '/login' || pathname === '/login/';

  useEffect(() => {
    if (isLoginPage) {
      setLoading(false);
      return;
    }

    const checkAuth = async () => {
      const cached = getAuthUser();
      if (cached) {
        setUser(cached);
      }

      try {
        const me = await authAPI.me();
        if (!me?.admin) {
          throw new Error('Missing admin profile');
        }
        setAuthUser(me.admin);
        setUser(me.admin);
        setLoading(false);
      } catch {
        clearAuthSession();
        router.push('/login');
      }
    };

    checkAuth().catch(() => {
      clearAuthSession();
      router.push('/login');
    });
  }, [router, isLoginPage]);

  if (isLoginPage) {
    return <>{children}</>;
  }

  const handleLogout = async () => {
    try {
      await authAPI.logout();
    } catch {
      // Keep logout idempotent client-side.
    }
    clearAuthSession();
    router.push('/login');
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-dark-900 flex items-center justify-center">
        <div className="text-dark-400">Loading...</div>
      </div>
    );
  }

  const navItems = ADMIN_NAV_ITEMS;

  return (
    <div className="flex h-screen">
      <aside className={`bg-dark-800/50 border-r border-dark-600/30 ${sidebarOpen ? 'w-64' : 'w-20'} transition-all duration-300 flex flex-col`}>
        <div className={`p-6 border-b border-dark-600/30 ${sidebarOpen ? '' : 'flex justify-center'}`}>
          {sidebarOpen ? (
            <div className="flex items-center gap-3">
              <KnolLogo className="w-8 h-8" label={`${SITE.name} logo`} />
              <h1 className="font-bold bg-clip-text text-transparent bg-gradient-brand">
                {`${SITE.name} Admin`}
              </h1>
            </div>
          ) : (
            <KnolLogo className="w-8 h-8" label={`${SITE.name} logo`} />
          )}
        </div>

        <nav className="flex-1 p-4 space-y-2 overflow-y-auto">
          {navItems.map((item) => {
            const isActive = pathname === item.href || pathname === item.href + '/';
            return (
              <Link
                key={item.href}
                href={item.href}
                className={`flex items-center space-x-3 px-4 py-2 rounded-lg transition-colors ${
                  isActive
                    ? 'bg-brand-500/20 text-brand-400 border border-brand-500/30'
                    : 'text-dark-300 hover:bg-dark-700/30 hover:text-dark-100'
                }`}
                title={!sidebarOpen ? item.label : ''}
              >
                <span className="text-xl">{item.icon}</span>
                {sidebarOpen && <span className="text-sm font-medium">{item.label}</span>}
              </Link>
            );
          })}
        </nav>

        <div className="p-4 border-t border-dark-600/30">
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="w-full p-2 rounded-lg hover:bg-dark-700/30 transition-colors text-dark-400 hover:text-dark-200"
            title={sidebarOpen ? 'Collapse' : 'Expand'}
          >
            {sidebarOpen ? '◀' : '▶'}
          </button>
        </div>
      </aside>

      <div className="flex-1 flex flex-col overflow-hidden">
        <header className="bg-dark-800/30 border-b border-dark-600/30 px-8 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <KnolLogo className="w-7 h-7" label={`${SITE.name} logo`} />
            <h2 className="text-xl font-semibold text-dark-100">Knol Control Plane</h2>
          </div>
          <div className="flex items-center space-x-4">
            <GlobalSearch />
            {user && (
              <>
                <div className="text-right">
                  <p className="text-sm text-dark-300">{user.email}</p>
                  <p className="text-xs text-dark-500 capitalize">{user.role}</p>
                </div>
                <button
                  onClick={handleLogout}
                  className="px-4 py-2 rounded-lg bg-dark-700/30 hover:bg-dark-700/50 text-dark-300 hover:text-dark-100 text-sm font-medium transition-colors"
                >
                  Logout
                </button>
              </>
            )}
          </div>
        </header>

        <main className="flex-1 overflow-auto">
          <div className="p-8">{children}</div>
        </main>
      </div>
    </div>
  );
}
