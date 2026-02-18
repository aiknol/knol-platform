/**
 * Knol Memory Platform SDK - Client
 * Main client class for interacting with the Knol API
 */

import {
  KnolClientConfig,
  KnolError,
  Memory,
  WriteMemoryRequest,
  UpdateMemoryRequest,
  SearchMemoryRequest,
  SearchMemoryResponse,
  BatchWriteMemoryRequest,
  BatchWriteMemoryResponse,
  Entity,
  EntityListResponse,
  EntityEdgesResponse,
  NeighborsResponse,
  ExpandedGraph,
  TraversalResponse,
  PathFinderResponse,
  Webhook,
  CreateWebhookRequest,
  ListWebhooksResponse,
  TenantUsage,
  AuditLogEntry,
  ListAuditLogResponse,
  ExportMemoriesRequest,
  ExportMemoriesResponse,
  ImportMemoriesRequest,
  ImportMemoriesResponse,
  SearchQueryBuilder as ISearchQueryBuilder,
} from './types.js';

// ============================================================================
// Search Query Builder Implementation
// ============================================================================

class SearchQueryBuilder implements ISearchQueryBuilder {
  private request: SearchMemoryRequest = { query: '' };

  query(q: string): SearchQueryBuilder {
    this.request.query = q;
    return this;
  }

  userId(id: string): SearchQueryBuilder {
    this.request.user_id = id;
    return this;
  }

  scope(scope: string | string[]): SearchQueryBuilder {
    this.request.scope = scope as any;
    return this;
  }

  kind(kind: string | string[]): SearchQueryBuilder {
    this.request.kind = kind as any;
    return this;
  }

  limit(n: number): SearchQueryBuilder {
    this.request.limit = n;
    return this;
  }

  minConfidence(score: number): SearchQueryBuilder {
    this.request.min_confidence = score;
    return this;
  }

  temporalFilter(filter: any): SearchQueryBuilder {
    this.request.temporal_filter = filter;
    return this;
  }

  sessionId(id: string): SearchQueryBuilder {
    this.request.session_id = id;
    return this;
  }

  agentId(id: string): SearchQueryBuilder {
    this.request.agent_id = id;
    return this;
  }

  tags(tags: string[]): SearchQueryBuilder {
    this.request.tags = tags;
    return this;
  }

  entityTypes(types: string[]): SearchQueryBuilder {
    this.request.entity_types = types;
    return this;
  }

  minImportance(score: number): SearchQueryBuilder {
    this.request.min_importance = score;
    return this;
  }

  applyDecay(apply: boolean): SearchQueryBuilder {
    this.request.apply_decay = apply;
    return this;
  }

  graphDepth(depth: number): SearchQueryBuilder {
    this.request.graph_depth = depth;
    return this;
  }

  build(): SearchMemoryRequest {
    if (!this.request.query) {
      throw new KnolError('Query is required', 400);
    }
    return { ...this.request };
  }
}

// ============================================================================
// Knol Client
// ============================================================================

export class KnolClient {
  private readonly apiKey: string;
  private readonly baseUrl: string;
  private readonly timeout: number;
  private readonly retryAttempts: number;
  private readonly retryDelayMs: number;

  constructor(config: KnolClientConfig) {
    if (!config.apiKey) {
      throw new KnolError('apiKey is required', 400);
    }

    this.apiKey = config.apiKey;
    this.baseUrl = config.baseUrl || 'https://api.knol.ai';
    this.timeout = config.timeout || 30000;
    this.retryAttempts = config.retryAttempts || 3;
    this.retryDelayMs = config.retryDelayMs || 1000;
  }

  /**
   * Creates a new search query builder for fluent API
   */
  searchBuilder(): ISearchQueryBuilder {
    return new SearchQueryBuilder();
  }

  // ========================================================================
  // Memory Operations
  // ========================================================================

  /**
   * Write a single memory
   */
  async writeMemory(memory: WriteMemoryRequest): Promise<Memory> {
    return this.post<Memory>('/v1/memory', memory);
  }

  /**
   * Batch write memories
   */
  async batchWriteMemory(
    memories: WriteMemoryRequest[],
    parallel = true
  ): Promise<BatchWriteMemoryResponse> {
    const request: BatchWriteMemoryRequest = { memories, parallel };
    return this.post<BatchWriteMemoryResponse>('/v1/memory/batch', request);
  }

  /**
   * Search memories with flexible query parameters
   */
  async searchMemory(query: SearchMemoryRequest | ISearchQueryBuilder): Promise<SearchMemoryResponse> {
    const searchQuery = query instanceof SearchQueryBuilder ? query.build() : query;
    return this.post<SearchMemoryResponse>('/v1/memory/search', searchQuery);
  }

  /**
   * Get a specific memory by ID
   */
  async getMemory(id: string): Promise<Memory> {
    return this.get<Memory>(`/v1/memory/${this.encodeId(id)}`);
  }

  /**
   * Update a memory
   */
  async updateMemory(id: string, updates: UpdateMemoryRequest): Promise<Memory> {
    return this.put<Memory>(`/v1/memory/${this.encodeId(id)}`, updates);
  }

  /**
   * Delete a memory
   */
  async deleteMemory(id: string): Promise<void> {
    await this.delete(`/v1/memory/${this.encodeId(id)}`);
  }

  /**
   * Export memories
   */
  async exportMemories(request: ExportMemoriesRequest): Promise<ExportMemoriesResponse> {
    return this.post<ExportMemoriesResponse>('/v1/memory/export', request);
  }

  /**
   * Import memories
   */
  async importMemories(request: ImportMemoriesRequest): Promise<ImportMemoriesResponse> {
    return this.post<ImportMemoriesResponse>('/v1/memory/import', request);
  }

  // ========================================================================
  // Graph Operations
  // ========================================================================

  /**
   * List entities with optional filtering
   */
  async listEntities(entityType?: string, limit = 100): Promise<EntityListResponse> {
    const params = new URLSearchParams();
    if (entityType) params.append('entity_type', entityType);
    params.append('limit', limit.toString());

    return this.get<EntityListResponse>(`/v1/graph/entities?${params}`);
  }

  /**
   * Get a specific entity
   */
  async getEntity(id: string): Promise<Entity> {
    return this.get<Entity>(`/v1/graph/entities/${this.encodeId(id)}`);
  }

  /**
   * Get edges for an entity
   */
  async getEntityEdges(id: string): Promise<EntityEdgesResponse> {
    return this.get<EntityEdgesResponse>(`/v1/graph/entities/${this.encodeId(id)}/edges`);
  }

  /**
   * Get neighbors of an entity
   */
  async getEntityNeighbors(
    id: string,
    relationType?: string,
    limit = 100
  ): Promise<NeighborsResponse> {
    const params = new URLSearchParams();
    if (relationType) params.append('rel_type', relationType);
    params.append('limit', limit.toString());

    return this.get<NeighborsResponse>(
      `/v1/graph/entities/${this.encodeId(id)}/neighbors?${params}`
    );
  }

  /**
   * Perform 2-hop expansion around an entity
   */
  async expandEntity(id: string): Promise<ExpandedGraph> {
    return this.get<ExpandedGraph>(`/v1/graph/entities/${this.encodeId(id)}/expand`);
  }

  /**
   * Traverse the graph N hops from an entity
   */
  async traverseGraph(
    id: string,
    depth = 2,
    limit = 100
  ): Promise<TraversalResponse> {
    const params = new URLSearchParams();
    params.append('depth', depth.toString());
    params.append('limit', limit.toString());

    return this.get<TraversalResponse>(
      `/v1/graph/entities/${this.encodeId(id)}/traverse?${params}`
    );
  }

  /**
   * Find path between two entities
   */
  async findPath(fromId: string, toId: string, maxDepth = 5): Promise<PathFinderResponse> {
    const params = new URLSearchParams();
    params.append('max_depth', maxDepth.toString());

    return this.get<PathFinderResponse>(
      `/v1/graph/path/${this.encodeId(fromId)}/${this.encodeId(toId)}?${params}`
    );
  }

  // ========================================================================
  // Webhook Operations
  // ========================================================================

  /**
   * List all webhooks
   */
  async listWebhooks(): Promise<ListWebhooksResponse> {
    return this.get<ListWebhooksResponse>('/v1/webhooks');
  }

  /**
   * Create a new webhook
   */
  async createWebhook(webhook: CreateWebhookRequest): Promise<Webhook> {
    return this.post<Webhook>('/v1/webhooks', webhook);
  }

  /**
   * Delete a webhook
   */
  async deleteWebhook(id: string): Promise<void> {
    await this.delete(`/v1/webhooks/${this.encodeId(id)}`);
  }

  // ========================================================================
  // Admin Operations
  // ========================================================================

  /**
   * Get tenant usage information
   */
  async getTenantUsage(): Promise<TenantUsage> {
    return this.get<TenantUsage>('/v1/admin/tenants');
  }

  /**
   * List audit log entries
   */
  async listAuditLog(limit = 100, offset = 0): Promise<ListAuditLogResponse> {
    const params = new URLSearchParams();
    params.append('limit', limit.toString());
    params.append('offset', offset.toString());

    return this.get<ListAuditLogResponse>(`/v1/admin/audit?${params}`);
  }

  // ========================================================================
  // HTTP Methods
  // ========================================================================

  private async get<T>(path: string): Promise<T> {
    return this.request<T>(path, { method: 'GET' });
  }

  private async post<T>(path: string, body: any): Promise<T> {
    return this.request<T>(path, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  private async put<T>(path: string, body: any): Promise<T> {
    return this.request<T>(path, {
      method: 'PUT',
      body: JSON.stringify(body),
    });
  }

  private async delete(path: string): Promise<void> {
    await this.request<void>(path, { method: 'DELETE' });
  }

  private async request<T>(path: string, init: RequestInit): Promise<T> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt < this.retryAttempts; attempt++) {
      try {
        return await this.executeRequest<T>(path, init);
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        // Don't retry on client errors (4xx)
        if (error instanceof KnolError && error.statusCode >= 400 && error.statusCode < 500) {
          throw error;
        }

        // Wait before retrying
        if (attempt < this.retryAttempts - 1) {
          await this.delay(this.retryDelayMs * Math.pow(2, attempt));
        }
      }
    }

    throw lastError || new KnolError('Request failed after retries', 500);
  }

  private async executeRequest<T>(path: string, init: RequestInit): Promise<T> {
    const url = `${this.baseUrl}${path}`;

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${this.apiKey}`,
      'User-Agent': `knol-sdk/typescript/0.1.0`,
    };

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        ...init,
        headers: { ...headers, ...(init.headers || {}) },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      // Parse response
      const contentType = response.headers.get('content-type');
      let data: any = null;

      if (contentType?.includes('application/json')) {
        data = await response.json();
      } else if (response.status !== 204) {
        data = await response.text();
      }

      // Handle errors
      if (!response.ok) {
        const errorMessage = data?.message || data?.error || response.statusText;
        const requestId = response.headers.get('x-request-id') || undefined;

        throw new KnolError(
          errorMessage || `HTTP ${response.status}`,
          response.status,
          requestId,
          data?.details
        );
      }

      return data as T;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof KnolError) {
        throw error;
      }

      if (error instanceof TypeError && error.message === 'Failed to fetch') {
        throw new KnolError('Network error: Failed to connect to API', 0);
      }

      if (error instanceof DOMException && error.name === 'AbortError') {
        throw new KnolError(`Request timeout after ${this.timeout}ms`, 408);
      }

      throw new KnolError(
        error instanceof Error ? error.message : 'Unknown error',
        500
      );
    }
  }

  private encodeId(id: string): string {
    return encodeURIComponent(id);
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
