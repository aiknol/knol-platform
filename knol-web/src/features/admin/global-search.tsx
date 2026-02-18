'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { useRouter } from 'next/navigation';
import { configAPI, Config, credentialsAPI, Credential } from '@/features/admin/api';

interface SearchResult {
  type: 'config' | 'credential';
  key: string;
  description?: string;
  category?: string;
  service?: string;
}

export function GlobalSearch() {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [configs, setConfigs] = useState<Config[]>([]);
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Load data when search opens
  const loadData = useCallback(async () => {
    if (configs.length > 0 && credentials.length > 0) return;
    setLoading(true);
    try {
      const [c, cr] = await Promise.all([configAPI.getAll(), credentialsAPI.list()]);
      setConfigs(c || []);
      setCredentials(cr || []);
    } catch {
      // Silently fail — user may not be authed yet
    } finally {
      setLoading(false);
    }
  }, [configs.length, credentials.length]);

  // Keyboard shortcut: Cmd/Ctrl + K
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setOpen((prev) => !prev);
      }
      if (e.key === 'Escape') {
        setOpen(false);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  // Focus input when opened
  useEffect(() => {
    if (open) {
      loadData();
      setTimeout(() => inputRef.current?.focus(), 50);
    } else {
      setQuery('');
      setResults([]);
      setSelectedIdx(0);
    }
  }, [open, loadData]);

  // Filter results
  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      setSelectedIdx(0);
      return;
    }

    const q = query.toLowerCase();

    const configResults: SearchResult[] = configs
      .filter((c) => {
        const searchable = `${c.key} ${c.description || ''} ${c.category || ''} ${c.env_override || ''}`.toLowerCase();
        return searchable.includes(q);
      })
      .slice(0, 8)
      .map((c) => ({
        type: 'config' as const,
        key: c.key,
        description: c.description,
        category: c.category,
      }));

    const credResults: SearchResult[] = credentials
      .filter((c) => {
        const searchable = `${c.name} ${c.service || ''} ${c.description || ''}`.toLowerCase();
        return searchable.includes(q);
      })
      .slice(0, 5)
      .map((c) => ({
        type: 'credential' as const,
        key: c.name,
        description: c.description,
        service: c.service,
      }));

    setResults([...configResults, ...credResults]);
    setSelectedIdx(0);
  }, [query, configs, credentials]);

  // Click outside to close
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  const navigate = (result: SearchResult) => {
    setOpen(false);
    if (result.type === 'config') {
      router.push(`/admin/config?search=${encodeURIComponent(result.key)}`);
    } else {
      router.push(`/admin/credentials?search=${encodeURIComponent(result.key)}`);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIdx((i) => Math.min(i + 1, results.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIdx((i) => Math.max(i - 1, 0));
    } else if (e.key === 'Enter' && results[selectedIdx]) {
      e.preventDefault();
      navigate(results[selectedIdx]);
    }
  };

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        className="flex items-center gap-2 px-3 py-1.5 bg-dark-700/30 border border-dark-600/40 rounded-lg text-dark-400 hover:text-dark-200 hover:bg-dark-700/50 transition-colors text-sm"
      >
        <span>Search settings...</span>
        <kbd className="hidden sm:inline-block px-1.5 py-0.5 text-[10px] bg-dark-600/50 border border-dark-600/40 rounded text-dark-500">
          {typeof navigator !== 'undefined' && /Mac/.test(navigator.userAgent) ? '⌘' : 'Ctrl'}+K
        </kbd>
      </button>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-start justify-center pt-[15vh] z-50">
      <div ref={containerRef} className="w-full max-w-xl mx-4 bg-dark-800 border border-dark-600/50 rounded-xl shadow-2xl overflow-hidden">
        {/* Search input */}
        <div className="flex items-center border-b border-dark-600/40 px-4">
          <svg className="w-5 h-5 text-dark-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search config keys, credentials, env vars..."
            className="flex-1 px-3 py-4 bg-transparent text-dark-100 text-sm placeholder:text-dark-500 focus:outline-none"
          />
          <kbd
            onClick={() => setOpen(false)}
            className="px-1.5 py-0.5 text-[10px] bg-dark-700/50 border border-dark-600/40 rounded text-dark-500 cursor-pointer hover:text-dark-300"
          >
            ESC
          </kbd>
        </div>

        {/* Results */}
        <div className="max-h-[50vh] overflow-y-auto">
          {loading && (
            <div className="px-4 py-8 text-center text-dark-400 text-sm">Loading...</div>
          )}

          {!loading && query && results.length === 0 && (
            <div className="px-4 py-8 text-center text-dark-500 text-sm">
              No results for &ldquo;{query}&rdquo;
            </div>
          )}

          {!loading && results.length > 0 && (
            <div className="py-2">
              {results.map((result, idx) => (
                <button
                  key={`${result.type}-${result.key}`}
                  onClick={() => navigate(result)}
                  onMouseEnter={() => setSelectedIdx(idx)}
                  className={`w-full text-left px-4 py-3 flex items-center gap-3 transition-colors ${
                    idx === selectedIdx ? 'bg-brand-500/15' : 'hover:bg-dark-700/30'
                  }`}
                >
                  <span className={`shrink-0 w-6 h-6 flex items-center justify-center rounded text-xs font-medium ${
                    result.type === 'config'
                      ? 'bg-blue-500/20 text-blue-400'
                      : 'bg-amber-500/20 text-amber-400'
                  }`}>
                    {result.type === 'config' ? 'C' : 'K'}
                  </span>
                  <div className="flex-1 min-w-0">
                    <p className="font-mono text-sm text-dark-100 truncate">{result.key}</p>
                    {result.description && (
                      <p className="text-xs text-dark-500 truncate mt-0.5">{result.description}</p>
                    )}
                  </div>
                  <div className="shrink-0 flex items-center gap-2">
                    {result.category && (
                      <span className="px-2 py-0.5 rounded-full text-[10px] bg-dark-700/60 border border-dark-600/40 text-dark-400">
                        {result.category}
                      </span>
                    )}
                    {result.service && (
                      <span className="px-2 py-0.5 rounded-full text-[10px] bg-amber-500/10 border border-amber-500/30 text-amber-300">
                        {result.service}
                      </span>
                    )}
                    <span className="text-[10px] text-dark-500">
                      {result.type === 'config' ? 'Config' : 'Credential'}
                    </span>
                  </div>
                </button>
              ))}
            </div>
          )}

          {!loading && !query && (
            <div className="px-4 py-6 text-center text-dark-500 text-sm space-y-2">
              <p>Search across all config keys and credentials</p>
              <p className="text-xs text-dark-600">
                Try: <span className="text-dark-400">gemini</span>, <span className="text-dark-400">api_key</span>, <span className="text-dark-400">port</span>, <span className="text-dark-400">cors</span>
              </p>
            </div>
          )}
        </div>

        {/* Footer hints */}
        {results.length > 0 && (
          <div className="border-t border-dark-600/40 px-4 py-2 flex items-center gap-4 text-[10px] text-dark-500">
            <span><kbd className="px-1 py-0.5 bg-dark-700/50 border border-dark-600/40 rounded">↑↓</kbd> Navigate</span>
            <span><kbd className="px-1 py-0.5 bg-dark-700/50 border border-dark-600/40 rounded">↵</kbd> Open</span>
            <span><kbd className="px-1 py-0.5 bg-dark-700/50 border border-dark-600/40 rounded">Esc</kbd> Close</span>
          </div>
        )}
      </div>
    </div>
  );
}
