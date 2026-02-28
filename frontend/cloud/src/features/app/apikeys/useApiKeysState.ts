'use client';

import { FormEvent, useCallback, useEffect, useState } from 'react';
import {
  ApiKeyItem,
  AppUser,
  appApiKeysAPI,
  appAuthAPI,
  consumeInitialApiKey,
  getAppAuthUser,
  storeSessionApiKey,
} from '@/features/app/api';

export function useApiKeysState() {
  const [user] = useState<AppUser | null>(getAppAuthUser());
  const [gatewayBaseUrl, setGatewayBaseUrl] = useState('');
  const [keys, setKeys] = useState<ApiKeyItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  // Create form state
  const [newName, setNewName] = useState('integration-key');
  const [newRole, setNewRole] = useState<'admin' | 'developer' | 'read_only'>('developer');
  const [newExpiryDays, setNewExpiryDays] = useState('30');
  const [createBusy, setCreateBusy] = useState(false);
  const [newlyCreatedApiKey, setNewlyCreatedApiKey] = useState<string | null>(consumeInitialApiKey());
  const [keyVisible, setKeyVisible] = useState(false);

  // Per-key visibility in the list (set of key IDs whose prefix is revealed)
  const [visibleKeyIds, setVisibleKeyIds] = useState<Set<string>>(new Set());

  const load = useCallback(async () => {
    setError('');
    setLoading(true);
    try {
      const [me, keyList] = await Promise.all([
        appAuthAPI.me(),
        appApiKeysAPI.list(),
      ]);
      setGatewayBaseUrl(me.gateway_base_url || '');
      setKeys(keyList);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load API keys');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load().catch(() => undefined);
  }, [load]);

  const onCreateKey = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setCreateBusy(true);
    setError('');
    try {
      const expires = Number(newExpiryDays);
      const payload: { name: string; role: 'admin' | 'developer' | 'read_only'; expires_in_days?: number } = {
        name: newName,
        role: newRole,
      };
      if (Number.isFinite(expires) && expires > 0) {
        payload.expires_in_days = expires;
      }
      const created = await appApiKeysAPI.create(payload);
      setNewlyCreatedApiKey(created.api_key);
      setKeyVisible(false);

      // Store full key in session vault so Playground can use it
      storeSessionApiKey({
        id: created.id,
        name: created.name || newName,
        role: created.role || newRole,
        api_key: created.api_key,
      });

      const keyList = await appApiKeysAPI.list();
      setKeys(keyList);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create API key');
    } finally {
      setCreateBusy(false);
    }
  };

  const onRevoke = async (id: string) => {
    if (!window.confirm('Revoke this API key? Existing integrations using it will stop working.')) {
      return;
    }
    setError('');
    try {
      await appApiKeysAPI.revoke(id);
      const keyList = await appApiKeysAPI.list();
      setKeys(keyList);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke API key');
    }
  };

  const toggleKeyVisibility = () => setKeyVisible((v) => !v);

  const toggleListKeyVisibility = (id: string) => {
    setVisibleKeyIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const isListKeyVisible = (id: string) => visibleKeyIds.has(id);

  const copyToClipboard = async (value: string) => {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      // Ignore clipboard errors.
    }
  };

  return {
    user,
    gatewayBaseUrl,
    keys,
    loading,
    error,
    newName,
    setNewName,
    newRole,
    setNewRole,
    newExpiryDays,
    setNewExpiryDays,
    createBusy,
    newlyCreatedApiKey,
    keyVisible,
    toggleKeyVisibility,
    visibleKeyIds,
    toggleListKeyVisibility,
    isListKeyVisible,
    onCreateKey,
    onRevoke,
    copyToClipboard,
  };
}
