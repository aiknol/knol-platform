'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import Link from 'next/link';
import { authAPI, getAuthToken, getAuthUser, clearAuthSession } from '@/features/admin/api';
import { ADMIN_NAV_ITEMS, SITE } from '@/config';
import { GlobalSearch } from '@/features/admin/global-search';

interface AdminUser {
  email: string;
  role: string;
}

export default function AdminLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const [user, setUser] = useState<AdminUser | null>(null);
  const [loading, setLoading] = useState(true);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const isLoginPage = pathname === '/admin/login' || pathname === '/admin/login/';

  useEffect(() => {
    // Skip auth check on login page
    if (isLoginPage) {
      setLoading(false);
      return;
    }

    const checkAuth = () => {
      const token = getAuthToken();

      if (!token) {
        router.push('/admin/login');
        return;
      }

      const userData = getAuthUser();
      if (userData) {
        setUser(userData);
      } else {
        clearAuthSession();
        router.push('/admin/login');
        return;
      }

      setLoading(false);
    };

    checkAuth();
  }, [router, isLoginPage]);

  // Login page renders without admin chrome
  if (isLoginPage) {
    return <>{children}</>;
  }

  const handleLogout = async () => {
    try {
      await authAPI.logout();
    } catch (err) {
      console.error('Logout error:', err);
    }
    clearAuthSession();
    router.push('/admin/login');
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
    <div className="flex h-[calc(100vh-4rem)]">
      {/* Sidebar */}
      <aside className={`bg-dark-800/50 border-r border-dark-600/30 ${sidebarOpen ? 'w-64' : 'w-20'} transition-all duration-300 flex flex-col`}>
        {/* Logo */}
        <div className="p-6 border-b border-dark-600/30">
          <h1 className={`font-bold bg-clip-text text-transparent bg-gradient-brand ${!sidebarOpen && 'text-center'}`}>
            {sidebarOpen ? `${SITE.name} Admin` : SITE.name.charAt(0)}
          </h1>
        </div>

        {/* Navigation */}
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

        {/* Toggle Sidebar */}
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

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="bg-dark-800/30 border-b border-dark-600/30 px-8 py-4 flex items-center justify-between">
          <h2 className="text-xl font-semibold text-dark-100">Knol Control Plane</h2>
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

        {/* Page Content */}
        <main className="flex-1 overflow-auto">
          <div className="p-8">{children}</div>
        </main>
      </div>
    </div>
  );
}
