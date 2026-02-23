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
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const isLoginPage = pathname === '/login' || pathname === '/login/';

  // Close mobile menu on route change
  useEffect(() => {
    setMobileMenuOpen(false);
  }, [pathname]);

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
      {/* Mobile overlay */}
      {mobileMenuOpen && (
        <div
          className="fixed inset-0 bg-black/60 z-40 lg:hidden"
          onClick={() => setMobileMenuOpen(false)}
        />
      )}

      {/* Sidebar — hidden on mobile, slide-in drawer when mobileMenuOpen */}
      <aside
        className={`
          bg-dark-800/50 border-r border-dark-600/30 flex flex-col
          transition-all duration-300 z-50
          fixed inset-y-0 left-0 lg:static
          ${mobileMenuOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
          ${sidebarOpen ? 'w-64' : 'w-20'}
        `}
      >
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
                onClick={() => setMobileMenuOpen(false)}
              >
                <span className="text-xl">{item.icon}</span>
                {sidebarOpen && <span className="text-sm font-medium">{item.label}</span>}
              </Link>
            );
          })}
        </nav>

        <div className="p-4 border-t border-dark-600/30 hidden lg:block">
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="w-full p-2 rounded-lg hover:bg-dark-700/30 transition-colors text-dark-400 hover:text-dark-200"
            title={sidebarOpen ? 'Collapse' : 'Expand'}
          >
            {sidebarOpen ? '\u25C0' : '\u25B6'}
          </button>
        </div>
      </aside>

      <div className="flex-1 flex flex-col overflow-hidden w-full">
        <header className="bg-dark-800/30 border-b border-dark-600/30 px-4 sm:px-6 lg:px-8 py-4 flex items-center justify-between gap-3">
          {/* Mobile menu button */}
          <button
            className="lg:hidden text-dark-200 hover:text-dark-50 p-1"
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            aria-label="Toggle menu"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              {mobileMenuOpen
                ? <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                : <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />}
            </svg>
          </button>

          <div className="flex items-center gap-3 min-w-0">
            <KnolLogo className="w-7 h-7 shrink-0 hidden sm:block" label={`${SITE.name} logo`} />
            <h2 className="text-lg sm:text-xl font-semibold text-dark-100 truncate">Knol Control Plane</h2>
          </div>
          <div className="flex items-center gap-2 sm:gap-4 shrink-0">
            <div className="hidden sm:block">
              <GlobalSearch />
            </div>
            {user && (
              <>
                <div className="text-right hidden md:block">
                  <p className="text-sm text-dark-300 truncate max-w-[200px]">{user.email}</p>
                  <p className="text-xs text-dark-500 capitalize">{user.role}</p>
                </div>
                <button
                  onClick={handleLogout}
                  className="px-3 sm:px-4 py-2 rounded-lg bg-dark-700/30 hover:bg-dark-700/50 text-dark-300 hover:text-dark-100 text-sm font-medium transition-colors"
                >
                  Logout
                </button>
              </>
            )}
          </div>
        </header>

        <main className="flex-1 overflow-auto">
          <div className="p-4 sm:p-6 lg:p-8">{children}</div>
        </main>
      </div>
    </div>
  );
}
