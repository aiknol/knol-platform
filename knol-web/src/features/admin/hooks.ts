'use client';

import { useState, useEffect, useCallback } from 'react';

// ── useAdminFetch: generic data fetcher with loading/error ──────

interface FetchState<T> {
  data: T | null;
  loading: boolean;
  error: string;
  refetch: () => void;
}

export function useAdminFetch<T>(
  fetchFn: () => Promise<T>,
  deps: unknown[] = [],
): FetchState<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const refetch = useCallback(() => {
    setLoading(true);
    setError('');
    fetchFn()
      .then(setData)
      .catch((err) => setError(err instanceof Error ? err.message : 'Failed to load'))
      .finally(() => setLoading(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  useEffect(() => {
    refetch();
  }, [refetch]);

  return { data, loading, error, refetch };
}

// ── useAdminAction: wrap async mutations with loading/error ─────

interface ActionState {
  loading: boolean;
  error: string;
}

export function useAdminAction() {
  const [state, setState] = useState<ActionState>({ loading: false, error: '' });

  const run = useCallback(async <T>(fn: () => Promise<T>): Promise<T | null> => {
    setState({ loading: true, error: '' });
    try {
      const result = await fn();
      setState({ loading: false, error: '' });
      return result;
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Action failed';
      setState({ loading: false, error: msg });
      return null;
    }
  }, []);

  const clearError = useCallback(() => setState((s) => ({ ...s, error: '' })), []);

  return { ...state, run, clearError };
}

// ── useEditMode: toggle edit state for a keyed item ─────────────

export function useEditMode<K extends string | number = string>() {
  const [editingKey, setEditingKey] = useState<K | null>(null);

  const startEdit = useCallback((key: K) => setEditingKey(key), []);
  const stopEdit = useCallback(() => setEditingKey(null), []);
  const isEditing = useCallback((key: K) => editingKey === key, [editingKey]);

  return { editingKey, startEdit, stopEdit, isEditing };
}
