# Knol API Mapping

This document maps MCP tools to the underlying Knol REST API endpoints.

## Architecture

```
AI Coding Tool (Claude Code, Cursor, Windsurf)
         ↓
    MCP Protocol
         ↓
   Knol MCP Server (this project)
         ↓
   HTTP Client (fetch)
         ↓
   Knol REST API
```

## Tool to API Endpoint Mapping

### knol_remember → POST /v1/memory

Stores a new memory in Knol.

**MCP Request:**
```json
{
  "name": "knol_remember",
  "arguments": {
    "content": "String of memory content",
    "user_id": "optional-user-id",
    "session_id": "optional-session-id",
    "metadata": {
      "key": "value"
    }
  }
}
```

**HTTP Request:**
```bash
POST /v1/memory HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
Content-Type: application/json

{
  "content": "String of memory content",
  "user_id": "user-id-or-default",
  "session_id": "optional-session-id",
  "metadata": {
    "key": "value"
  }
}
```

**Knol API Response:**
```json
{
  "id": "mem_abc123xyz",
  "content": "String of memory content",
  "user_id": "user-id",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

**MCP Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{...formatted JSON...}"
    }
  ]
}
```

---

### knol_search → POST /v1/memory/search

Searches memories with semantic matching.

**MCP Request:**
```json
{
  "name": "knol_search",
  "arguments": {
    "query": "search terms",
    "user_id": "optional-filter",
    "limit": 5,
    "kind": "optional-type",
    "graph_depth": 0
  }
}
```

**HTTP Request:**
```bash
POST /v1/memory/search HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
Content-Type: application/json

{
  "query": "search terms",
  "user_id": "optional-filter",
  "limit": 5,
  "kind": "optional-type",
  "graph_depth": 0
}
```

**Knol API Response:**
```json
{
  "memories": [
    {
      "id": "mem_123",
      "content": "matching content",
      "user_id": "user-id",
      "confidence": 0.95,
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 1,
  "query": "search terms"
}
```

**MCP Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{...formatted JSON...}"
    }
  ]
}
```

---

### knol_get → GET /v1/memory/:id

Retrieves a specific memory by ID.

**MCP Request:**
```json
{
  "name": "knol_get",
  "arguments": {
    "memory_id": "mem_abc123xyz"
  }
}
```

**HTTP Request:**
```bash
GET /v1/memory/mem_abc123xyz HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
```

**Knol API Response:**
```json
{
  "id": "mem_abc123xyz",
  "content": "memory content",
  "user_id": "user-id",
  "importance": 8,
  "status": "active",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:35:00Z"
}
```

**MCP Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{...formatted JSON...}"
    }
  ]
}
```

---

### knol_update → PUT /v1/memory/:id

Updates an existing memory.

**MCP Request:**
```json
{
  "name": "knol_update",
  "arguments": {
    "memory_id": "mem_abc123xyz",
    "content": "new content",
    "importance": 9
  }
}
```

**HTTP Request:**
```bash
PUT /v1/memory/mem_abc123xyz HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
Content-Type: application/json

{
  "content": "new content",
  "importance": 9
}
```

**Knol API Response:**
```json
{
  "id": "mem_abc123xyz",
  "content": "new content",
  "user_id": "user-id",
  "importance": 9,
  "updated_at": "2024-01-15T10:40:00Z"
}
```

**MCP Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{...formatted JSON...}"
    }
  ]
}
```

---

### knol_delete → DELETE /v1/memory/:id

Deletes a memory.

**MCP Request:**
```json
{
  "name": "knol_delete",
  "arguments": {
    "memory_id": "mem_abc123xyz"
  }
}
```

**HTTP Request:**
```bash
DELETE /v1/memory/mem_abc123xyz HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
```

**Knol API Response:**
```json
{
  "success": true,
  "id": "mem_abc123xyz",
  "deleted_at": "2024-01-15T10:45:00Z"
}
```

**MCP Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"success\": true, ...}"
    }
  ]
}
```

---

### knol_entities → GET /v1/graph/entities

Lists knowledge graph entities.

**MCP Request:**
```json
{
  "name": "knol_entities",
  "arguments": {
    "entity_type": "technology",
    "limit": 20
  }
}
```

**HTTP Request:**
```bash
GET /v1/graph/entities?entity_type=technology&limit=20 HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
```

**Knol API Response:**
```json
{
  "entities": [
    {
      "id": "ent_123",
      "name": "TypeScript",
      "type": "technology",
      "properties": {
        "category": "language",
        "ecosystem": "javascript"
      },
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 42,
  "limit": 20
}
```

**MCP Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{...formatted JSON...}"
    }
  ]
}
```

---

### knol_entity_neighbors → GET /v1/graph/entities/:id/neighbors

Gets related entities.

**MCP Request:**
```json
{
  "name": "knol_entity_neighbors",
  "arguments": {
    "entity_id": "ent_typescript",
    "limit": 10
  }
}
```

**HTTP Request:**
```bash
GET /v1/graph/entities/ent_typescript/neighbors?limit=10 HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
```

**Knol API Response:**
```json
{
  "entity_id": "ent_typescript",
  "neighbors": [
    {
      "id": "ent_nodejs",
      "name": "Node.js",
      "type": "technology",
      "relationship": "ecosystem",
      "strength": 0.95
    },
    {
      "id": "ent_react",
      "name": "React",
      "type": "framework",
      "relationship": "related",
      "strength": 0.87
    }
  ],
  "total": 15,
  "limit": 10
}
```

**MCP Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{...formatted JSON...}"
    }
  ]
}
```

---

## Resource to API Mapping

### knol://recent → POST /v1/memory/search

The `knol://recent` resource uses the search endpoint with wildcard query.

**Resource Read Request:**
```json
{
  "method": "resources/read",
  "params": {
    "uri": "knol://recent"
  }
}
```

**HTTP Request (Behind the Scenes):**
```bash
POST /v1/memory/search HTTP/1.1
Authorization: Bearer {KNOL_API_KEY}
Content-Type: application/json

{
  "query": "*",
  "user_id": "{KNOL_USER_ID}",
  "limit": 10
}
```

**Knol API Response:**
```json
{
  "memories": [
    {
      "id": "mem_latest",
      "content": "most recent",
      "created_at": "2024-01-15T10:50:00Z"
    },
    // ... 9 more entries
  ],
  "total": 247
}
```

**MCP Resource Response:**
```json
{
  "contents": [
    {
      "uri": "knol://recent",
      "mimeType": "application/json",
      "text": "{...formatted memories...}"
    }
  ]
}
```

---

## Authentication Headers

All requests include Bearer token authentication:

```http
Authorization: Bearer {KNOL_API_KEY}
Content-Type: application/json
```

The `KNOL_API_KEY` is set via the `KNOL_API_KEY` environment variable.

---

## Error Handling

### HTTP Status Codes

| Status | Meaning | Handling |
|--------|---------|----------|
| 200 | Success | Return parsed JSON |
| 201 | Created | Return parsed JSON |
| 400 | Bad Request | Return error message |
| 401 | Unauthorized | Return "Invalid API key" error |
| 403 | Forbidden | Return "Access denied" error |
| 404 | Not Found | Return "Resource not found" error |
| 429 | Rate Limited | Return "Rate limit exceeded" error |
| 500 | Server Error | Return "API server error" message |

### Error Response Format

**HTTP Error Response:**
```json
{
  "error": "Unauthorized",
  "message": "Invalid bearer token",
  "code": "INVALID_AUTH"
}
```

**MCP Error Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Error: Knol API error (401): Unauthorized"
    }
  ],
  "isError": true
}
```

---

## Request/Response Flow Example

### Complete Memory Lifecycle

1. **Store Memory**
   - MCP Tool: `knol_remember`
   - HTTP: `POST /v1/memory`
   - Result: Memory ID returned

2. **Retrieve Memory**
   - MCP Tool: `knol_get`
   - HTTP: `GET /v1/memory/{id}`
   - Result: Full memory object

3. **Update Memory**
   - MCP Tool: `knol_update`
   - HTTP: `PUT /v1/memory/{id}`
   - Result: Updated memory object

4. **Search Memories**
   - MCP Tool: `knol_search`
   - HTTP: `POST /v1/memory/search`
   - Result: Array of matching memories

5. **Delete Memory**
   - MCP Tool: `knol_delete`
   - HTTP: `DELETE /v1/memory/{id}`
   - Result: Deletion confirmation

---

## API Rate Limiting

The Knol API may implement rate limiting. Key considerations:

- **Rate Limit Header**: `X-RateLimit-Remaining`
- **Reset Time**: `X-RateLimit-Reset`
- **Status Code**: 429 (Too Many Requests)

The server will return errors if rate limits are exceeded.

---

## Pagination

For large result sets, use `limit` parameter:

- **knol_search**: `limit` parameter (default: 5)
- **knol_entities**: `limit` parameter (default: 20)
- **knol_entity_neighbors**: `limit` parameter (default: 10)

There's currently no offset-based pagination; use semantic search with refined queries instead.

---

## Field Mapping

### Memory Fields

| Knol API Field | MCP Parameter | Type | Required |
|---|---|---|---|
| `content` | `content` | string | Yes (for storage) |
| `user_id` | `user_id` | string | No (uses default) |
| `session_id` | `session_id` | string | No |
| `agent_id` | (not exposed) | string | No |
| `role` | (not exposed) | string | No |
| `metadata` | `metadata` | object | No |
| `importance` | `importance` | number | No |
| `status` | (not exposed) | string | No |
| `confidence` | (search only) | number | No |
| `created_at` | (read-only) | ISO-8601 | - |
| `updated_at` | (read-only) | ISO-8601 | - |

### Entity Fields

| Knol API Field | MCP Parameter | Type |
|---|---|---|
| `id` | `entity_id` | string |
| `name` | (read-only) | string |
| `type` | `entity_type` | string |
| `properties` | (read-only) | object |
| `created_at` | (read-only) | ISO-8601 |

---

## Implementation Details

The server uses:

- **HTTP Client**: Native Node.js `fetch` API
- **Protocol**: HTTP/1.1
- **Authentication**: Bearer Token (RFC 6750)
- **Content-Type**: `application/json`
- **Transport**: MCP over stdio

---

## Backwards Compatibility

This implementation maps to Knol API v1:

- Endpoint prefix: `/v1/`
- Version stability: Stable
- Deprecation policy: 6-month notice period

If the Knol API updates (e.g., to v2), the MCP server will need updates to the endpoint paths.

---

## Performance Considerations

### Request Overhead

- HTTP overhead: ~50-100ms per request
- MCP serialization: ~1-5ms
- Knol API processing: Varies (typically 100-500ms)

Total latency per operation: Typically 200-600ms

### Optimization Strategies

1. **Batch Operations**: Group related memories
2. **Limit Results**: Use appropriate `limit` values
3. **Scope Searches**: Use `kind` and `user_id` filters
4. **Cache Results**: Consider client-side caching of recent results

### Large Responses

- Responses over 10MB may timeout
- Keep `limit` reasonable (under 1000)
- Use pagination/search refinement for large datasets
