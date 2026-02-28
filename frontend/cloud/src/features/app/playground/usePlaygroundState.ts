'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import {
  ApiKeyItem,
  appAuthAPI,
  appApiKeysAPI,
  clearAppAuthSession,
  getAppAuthUser,
  getInitialApiKey,
  getSessionApiKeys,
  getSessionApiKeyValue,
  storeSessionApiKey,
} from '@/features/app/api';
import { OPERATIONS, OperationDef, OperationField } from './operations';

/** An item from the gateway with an ID and descriptive label for dropdowns. */
export interface SampleItem {
  id: string;
  label: string;
}

function normalizeGatewayBaseUrl(value: string): string {
  const trimmed = value.trim();
  return trimmed.replace(/\/+$/, '');
}

function inferDefaultGatewayBaseUrl(): string {
  if (typeof window === 'undefined') return '';
  const host = window.location.hostname;
  if (host === 'localhost' || host === '127.0.0.1') {
    return 'http://localhost:3000';
  }
  return 'https://api.aiknol.com';
}

function isLocalFrontend(): boolean {
  if (typeof window === 'undefined') return false;
  const host = window.location.hostname;
  return host === 'localhost' || host === '127.0.0.1';
}

function isDefaultPublicGateway(url: string): boolean {
  try {
    return new URL(url).hostname === 'api.aiknol.com';
  } catch {
    return false;
  }
}

function isUuid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
}

/** Sample data fetched from the gateway to populate field defaults with real IDs. */
export interface SampleData {
  memoryIds: string[];
  entityIds: string[];
  userIds: string[];
  /** Full memory items for dropdown display (id + content preview). */
  memoryItems: SampleItem[];
  /** Full entity items for dropdown display (id + name). */
  entityItems: SampleItem[];
}

/**
 * Build a fieldValues record pre-filled with any defaultValue entries.
 * When `sample` is provided, overrides static placeholders with real IDs.
 */
function buildDefaults(op: OperationDef, sample?: SampleData | null): Record<string, string> {
  const defaults: Record<string, string> = {};
  const allFields: OperationField[] = [...op.pathParams, ...op.bodyFields];
  for (const f of allFields) {
    if (f.defaultValue) {
      defaults[f.name] = f.defaultValue;
    }
  }

  if (!sample) return defaults;

  // Override static placeholders with real data based on operation group
  for (const f of allFields) {
    if (f.name === 'user_id' && sample.userIds[0]) {
      defaults['user_id'] = sample.userIds[0];
    } else if (f.name === 'id' && op.group === 'Memory' && sample.memoryIds[0]) {
      defaults['id'] = sample.memoryIds[0];
    } else if (f.name === 'id' && op.group === 'Graph' && sample.entityIds[0]) {
      defaults['id'] = sample.entityIds[0];
    } else if (f.name === 'from' && sample.entityIds[0]) {
      defaults['from'] = sample.entityIds[0];
    } else if (f.name === 'to' && sample.entityIds.length > 1) {
      defaults['to'] = sample.entityIds[1];
    }
  }

  return defaults;
}

/** A selectable key entry shown in the Playground dropdown. */
export interface PlaygroundKeyOption {
  id: string;
  name: string;
  role: string;
  /** Whether the full key value is available (created this session / signup key). */
  hasValue: boolean;
  key_prefix?: string;
}

function extractUpstreamError(json: any, fallback: string): string {
  const msg =
    json?.error ||
    json?.message ||
    json?.detail ||
    json?.errors?.[0]?.message ||
    json?.errors?.[0]?.detail;
  if (typeof msg === 'string' && msg.trim()) return msg.trim();
  return fallback;
}

export function usePlaygroundState() {
  const [gatewayBaseUrl, setGatewayBaseUrlState] = useState(() => inferDefaultGatewayBaseUrl());
  const [workspaceMode, setWorkspaceMode] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const [selectedOperationId, setSelectedOperationId] = useState(OPERATIONS[0].id);
  const [apiKey, setApiKey] = useState('');
  const [apiKeyVisible, setApiKeyVisible] = useState(false);
  const [fieldValues, setFieldValues] = useState<Record<string, string>>(buildDefaults(OPERATIONS[0]));
  const [executing, setExecuting] = useState(false);
  const [response, setResponse] = useState<{
    status: number;
    body: string;
    duration: number;
  } | null>(null);
  const [responseError, setResponseError] = useState('');

  // Available API keys from the backend list
  const [availableKeys, setAvailableKeys] = useState<ApiKeyItem[]>([]);
  const [selectedKeyId, setSelectedKeyId] = useState<string>('manual');
  const [creatingKey, setCreatingKey] = useState(false);
  const [sampleData, setSampleData] = useState<SampleData | null>(null);
  const [sampleLoading, setSampleLoading] = useState(false);
  const [sampleError, setSampleError] = useState('');

  const selectedOperation: OperationDef =
    OPERATIONS.find((op) => op.id === selectedOperationId) || OPERATIONS[0];

  /**
   * Build the list of key options combining backend keys with session knowledge.
   * Keys created during this session have their full value available.
   */
  const buildKeyOptions = useCallback((): PlaygroundKeyOption[] => {
    const sessionKeys = getSessionApiKeys();
    const sessionKeyIds = new Set(sessionKeys.map((sk) => sk.id));

    return availableKeys
      .filter((k) => k.active)
      .map((k) => ({
        id: k.id,
        name: k.name,
        role: k.role,
        hasValue: sessionKeyIds.has(k.id),
        key_prefix: k.key_prefix,
      }));
  }, [availableKeys]);

  const keyOptions = buildKeyOptions();

  useEffect(() => {
    (async () => {
      try {
        // Avoid calling protected endpoints (and spamming 401s in the console)
        // unless the browser already has a cached app profile.
        const cachedUser = getAppAuthUser();
        if (!cachedUser) return;

        const [me, keyList] = await Promise.all([appAuthAPI.me(), appApiKeysAPI.list()]);
        setWorkspaceMode(true);
        // Always use the local gateway in local dev.
        if (!isLocalFrontend() && me?.gateway_base_url) {
          setGatewayBaseUrlState(normalizeGatewayBaseUrl(me.gateway_base_url));
        }
        setAvailableKeys(keyList);

        // Auto-populate with the initial signup key if available
        const initialKey = getInitialApiKey();
        if (initialKey) {
          setApiKey(initialKey);
          setSelectedKeyId('initial');
        } else {
          // Check if any existing key has a session value available
          const sessionKeys = getSessionApiKeys();
          const sessionKeyIds = new Set(sessionKeys.map((sk) => sk.id));
          const usable = keyList.filter((k) => k.active && sessionKeyIds.has(k.id));

          if (usable.length > 0) {
            // Auto-select the first usable key
            const first = usable[0];
            const value = getSessionApiKeyValue(first.id);
            if (value) {
              setApiKey(value);
              setSelectedKeyId(first.id);
            }
          } else {
            // No usable keys at all — auto-create one
            try {
              setCreatingKey(true);
              const created = await appApiKeysAPI.create({
                name: 'playground-key',
                role: 'developer',
              });
              storeSessionApiKey({
                id: created.id,
                name: created.name || 'playground-key',
                role: created.role || 'developer',
                api_key: created.api_key,
              });
              const refreshed = await appApiKeysAPI.list();
              setAvailableKeys(refreshed);
              setApiKey(created.api_key);
              setSelectedKeyId(created.id);
            } catch {
              // Auto-creation failed; user can still create manually
            } finally {
              setCreatingKey(false);
            }
          }
        }
      } catch (err) {
        if (err instanceof Error && err.message === 'Unauthorized') {
          // Auth expired / cookies cleared — fall back to manual mode and avoid
          // retrying on the next mount by clearing the cached profile.
          clearAppAuthSession();
          setAvailableKeys([]);
          setWorkspaceMode(false);
          setError('');
        } else {
          setError(err instanceof Error ? err.message : 'Failed to load playground');
        }
      } finally {
        setLoading(false);
      }
    })();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const onSelectKey = useCallback((keyId: string) => {
    setSelectedKeyId(keyId);
    if (keyId === 'manual') {
      // Switch to manual entry - clear the field
      setApiKey('');
    } else if (keyId === 'initial') {
      const initialKey = getInitialApiKey();
      if (initialKey) setApiKey(initialKey);
    } else {
      // Look up full key value from session vault
      const value = getSessionApiKeyValue(keyId);
      if (value) {
        setApiKey(value);
      }
      // If value not found, keep the existing apiKey so the user can paste manually
    }
  }, []);

  const onSelectOperation = useCallback((id: string) => {
    setSelectedOperationId(id);
    const op = OPERATIONS.find((o) => o.id === id) || OPERATIONS[0];
    setFieldValues(buildDefaults(op, sampleData));
    setResponse(null);
    setResponseError('');
  }, [sampleData]);

  const setFieldValue = useCallback((name: string, value: string) => {
    setFieldValues((prev) => ({ ...prev, [name]: value }));
  }, []);

  const toggleApiKeyVisibility = () => setApiKeyVisible((v) => !v);

  const buildUrl = (): string => {
    const base = normalizeGatewayBaseUrl(gatewayBaseUrl);
    let path = selectedOperation.pathTemplate;
    for (const p of selectedOperation.pathParams) {
      path = path.replace(`:${p.name}`, encodeURIComponent(fieldValues[p.name] || ''));
    }
    return `${base}${path}`;
  };

  const buildBody = (): string | undefined => {
    if (selectedOperation.bodyFields.length === 0) return undefined;

    // Special case: _rootBody sends its JSON value directly as the request body
    // (e.g. batch-write expects a root-level array, not wrapped in an object).
    const rootBodyField = selectedOperation.bodyFields.find((f) => f.name === '_rootBody');
    if (rootBodyField) {
      const val = fieldValues['_rootBody'];
      if (!val) return undefined;
      // Validate it's valid JSON, then send as-is
      try {
        JSON.parse(val);
        return val;
      } catch {
        return val;
      }
    }

    const obj: Record<string, unknown> = {};
    for (const f of selectedOperation.bodyFields) {
      const val = fieldValues[f.name];
      if (val === undefined || val === '') continue;
      if (f.type === 'json') {
        try {
          obj[f.name] = JSON.parse(val);
        } catch {
          obj[f.name] = val;
        }
      } else if (f.type === 'number') {
        obj[f.name] = Number(val);
      } else {
        obj[f.name] = val;
      }
    }
    return JSON.stringify(obj);
  };

  const onExecute = async () => {
    setResponseError('');
    setResponse(null);

    if (!apiKey.trim()) {
      setResponseError('Please enter or select an API key.');
      return;
    }
    if (!gatewayBaseUrl) {
      setResponseError('Gateway URL not available.');
      return;
    }

    // Basic required-field validation (since we don't submit a real <form>).
    const requiredFields = [...selectedOperation.pathParams, ...selectedOperation.bodyFields].filter((f) => f.required);
    for (const f of requiredFields) {
      const val = (fieldValues[f.name] || '').trim();
      if (!val) {
        setResponseError(`${f.label} is required.`);
        return;
      }
    }

    // Validate UUID-typed inputs where the backend expects UUIDs.
    // We only enforce for user_id because it's optional and a common pitfall.
    const userId = (fieldValues['user_id'] || '').trim();
    if (userId && !isUuid(userId)) {
      setResponseError('User ID must be a UUID (leave blank to search all users).');
      return;
    }

    setExecuting(true);
    const start = performance.now();
    try {
      const url = buildUrl();
      const body = buildBody();

      // Route through the server-side proxy to avoid CORS issues.
      // The proxy makes the gateway request server-side and returns { status, body }.
      const proxyRes = await fetch('/api/playground/proxy', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url,
          method: selectedOperation.method,
          headers: {
            'Authorization': `Bearer ${apiKey}`,
            'Content-Type': 'application/json',
          },
          body,
        }),
      });

      const duration = Math.round(performance.now() - start);
      let proxyData: any = null;
      try {
        proxyData = await proxyRes.json();
      } catch {
        const text = await proxyRes.text().catch(() => '');
        setResponseError(text || `Proxy error (HTTP ${proxyRes.status})`);
        return;
      }

      if (proxyData.error) {
        setResponseError(proxyData.error);
        return;
      }

      let formatted = proxyData.body || '';
      const MAX_BODY_CHARS = 200_000;
      if (formatted.length > MAX_BODY_CHARS) {
        formatted = `${formatted.slice(0, MAX_BODY_CHARS)}\n\n[truncated: response too large to display]`;
      } else {
        try {
          formatted = JSON.stringify(JSON.parse(formatted), null, 2);
        } catch {
          // not JSON, keep as-is
        }
      }
      if (proxyData.status >= 400) {
        // Try to surface a useful upstream error message.
        try {
          const parsed = JSON.parse(proxyData.body || '{}');
          const msg =
            parsed?.error ||
            parsed?.message ||
            parsed?.detail ||
            parsed?.errors?.[0]?.message;
          if (msg && typeof msg === 'string') {
            setResponseError(msg);
          } else {
            setResponseError(`Request failed (HTTP ${proxyData.status}).`);
          }
        } catch {
          setResponseError(`Request failed (HTTP ${proxyData.status}).`);
        }
      }
      setResponse({ status: proxyData.status, body: formatted, duration });
    } catch (err) {
      setResponseError(err instanceof Error ? err.message : 'Request failed');
    } finally {
      setExecuting(false);
    }
  };

  const onCreateQuickKey = useCallback(async () => {
    setCreatingKey(true);
    setError('');
    try {
      const created = await appApiKeysAPI.create({
        name: 'playground-key',
        role: 'developer',
      });

      // Store in session vault so it appears in the dropdown
      storeSessionApiKey({
        id: created.id,
        name: created.name || 'playground-key',
        role: created.role || 'developer',
        api_key: created.api_key,
      });

      // Refresh the key list and auto-select the newly created key
      const keyList = await appApiKeysAPI.list();
      setAvailableKeys(keyList);
      setApiKey(created.api_key);
      setSelectedKeyId(created.id);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create key');
    } finally {
      setCreatingKey(false);
    }
  }, []);

  // Fetch sample data from the gateway to pre-populate fields with real IDs.
  const fetchSampleData = useCallback(async () => {
    if (!apiKey.trim() || !gatewayBaseUrl) return;
    setSampleLoading(true);
    setSampleError('');

    try {
      const errors: string[] = [];
      const proxyCall = async (path: string, method: string, body?: string) => {
        const res = await fetch('/api/playground/proxy', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            url: `${normalizeGatewayBaseUrl(gatewayBaseUrl)}${path}`,
            method,
            headers: {
              Authorization: `Bearer ${apiKey}`,
              'Content-Type': 'application/json',
            },
            body,
          }),
        });

        let data: any = null;
        try {
          data = await res.json();
        } catch {
          return { status: res.status, error: `Proxy did not return JSON (HTTP ${res.status})` } as const;
        }

        if (data?.error) return { status: res.status, error: data.error } as const;
        const upstreamStatus = Number(data?.status ?? 0);
        const upstreamBody = String(data?.body ?? '');

        let parsed: any = null;
        try {
          parsed = JSON.parse(upstreamBody);
        } catch {
          parsed = null;
        }

        if (upstreamStatus >= 400) {
          return {
            status: upstreamStatus,
            error: extractUpstreamError(parsed, `Upstream error (HTTP ${upstreamStatus}).`),
            bodyText: upstreamBody,
            json: parsed,
          } as const;
        }

        return { status: upstreamStatus, bodyText: upstreamBody, json: parsed } as const;
      };

      // Use /v1/memory/export to get all memories (works even without vectors),
      // /v1/graph/entities for entity data.
      const [exportRes, entityRes] = await Promise.allSettled([
        proxyCall(
          '/v1/memory/export',
          'POST',
          JSON.stringify({ limit: 50, include_graph: false, include_episodes: false }),
        ),
        proxyCall('/v1/graph/entities?limit=20', 'GET'),
      ]);

      const result: SampleData = {
        memoryIds: [], entityIds: [], userIds: [],
        memoryItems: [], entityItems: [],
      };

      const exportVal = exportRes.status === 'fulfilled' ? exportRes.value : null;
      if (exportVal && (exportVal as any).error) {
        errors.push(String((exportVal as any).error));
      } else if (exportVal && (exportVal as any).json) {
        const exportJson = (exportVal as any).json;
        const memories = exportJson.memories || exportJson.data || [];
        const memArr = Array.isArray(memories) ? memories : [];
        result.memoryIds = memArr
          .map((m: Record<string, unknown>) => m.id as string)
          .filter(Boolean)
          .slice(0, 20);
        result.memoryItems = memArr
          .filter((m: Record<string, unknown>) => m.id)
          .slice(0, 20)
          .map((m: Record<string, unknown>) => ({
            id: m.id as string,
            label: ((m.content as string) || '').slice(0, 60) || (m.kind as string) || 'memory',
          }));
        // Extract user_ids from memories
        for (const m of memArr) {
          if (m.user_id && !result.userIds.includes(m.user_id as string)) {
            result.userIds.push(m.user_id as string);
          }
        }
      } else if (exportVal && (exportVal as any).status >= 400) {
        errors.push(`Failed to load memories (${(exportVal as any).status}).`);
      }

      const entityVal = entityRes.status === 'fulfilled' ? entityRes.value : null;
      if (entityVal && (entityVal as any).error) {
        // Entity list isn't critical for memory ops; only show if nothing else failed.
        if (errors.length === 0) errors.push(String((entityVal as any).error));
      } else if (entityVal && (entityVal as any).json) {
        const entityJson = (entityVal as any).json;
        const entities = entityJson.entities || entityJson.data || entityJson;
        const entArr = Array.isArray(entities) ? entities : [];
        result.entityIds = entArr
          .map((e: Record<string, unknown>) => e.id as string)
          .filter(Boolean)
          .slice(0, 20);
        result.entityItems = entArr
          .filter((e: Record<string, unknown>) => e.id)
          .slice(0, 20)
          .map((e: Record<string, unknown>) => ({
            id: e.id as string,
            label: (e.name as string) || (e.entity_type as string) || 'entity',
          }));
      }

      if (errors.length > 0) {
        setSampleError(errors[0]);
      }
      setSampleData(result);
    } catch {
      setSampleError('Failed to load sample data.');
    } finally {
      setSampleLoading(false);
    }
  }, [apiKey, gatewayBaseUrl]);

  const prevInputsRef = useRef<{ apiKey: string; gatewayBaseUrl: string }>({ apiKey: '', gatewayBaseUrl: '' });

  // Clear + refetch sample data when connection inputs change.
  useEffect(() => {
    const apiKeyTrimmed = apiKey.trim();
    const gw = normalizeGatewayBaseUrl(gatewayBaseUrl);
    const prev = prevInputsRef.current;

    if (prev.apiKey !== apiKeyTrimmed || prev.gatewayBaseUrl !== gw) {
      prevInputsRef.current = { apiKey: apiKeyTrimmed, gatewayBaseUrl: gw };
      setSampleData(null);
      setSampleError('');
      if (apiKeyTrimmed && gw) {
        fetchSampleData();
      }
    }
  }, [apiKey, gatewayBaseUrl, fetchSampleData]);

  // Re-apply defaults when sample data arrives
  useEffect(() => {
    if (sampleData) {
      setFieldValues(buildDefaults(selectedOperation, sampleData));
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sampleData]);

  return {
    gatewayBaseUrl,
    workspaceMode,
    loading,
    error,
    selectedOperation,
    selectedOperationId,
    onSelectOperation,
    apiKey,
    setApiKey,
    apiKeyVisible,
    toggleApiKeyVisibility,
    fieldValues,
    setFieldValue,
    executing,
    response,
    responseError,
    onExecute,
    keyOptions,
    availableKeys,
    selectedKeyId,
    setSelectedKeyId,
    onSelectKey,
    creatingKey,
    onCreateQuickKey,
    sampleData,
    fetchSampleData,
    sampleLoading,
    sampleError,
  };
}
