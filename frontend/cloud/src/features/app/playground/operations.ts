export interface OperationField {
  name: string;
  label: string;
  type: 'string' | 'number' | 'textarea' | 'json';
  required?: boolean;
  placeholder?: string;
  /** Pre-filled default so users can test immediately. */
  defaultValue?: string;
}

export interface OperationDef {
  id: string;
  label: string;
  group: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  pathTemplate: string;
  pathParams: OperationField[];
  bodyFields: OperationField[];
  description: string;
}

export const OPERATIONS: OperationDef[] = [
  // --- Memory ---
  {
    id: 'search-memory',
    label: 'Search Memories',
    group: 'Memory',
    method: 'POST',
    pathTemplate: '/v1/memory/search',
    pathParams: [],
    bodyFields: [
      { name: 'query', label: 'Query', type: 'string', required: true, placeholder: 'What does the user prefer?', defaultValue: 'What does the user prefer?' },
      { name: 'user_id', label: 'User ID', type: 'string', placeholder: '550e8400-e29b-41d4-a716-446655440000' },
      { name: 'limit', label: 'Limit', type: 'number', placeholder: '10', defaultValue: '10' },
    ],
    description: 'Semantic search across stored memories.',
  },
  {
    id: 'get-memory',
    label: 'Get Memory',
    group: 'Memory',
    method: 'GET',
    pathTemplate: '/v1/memory/:id',
    pathParams: [
      { name: 'id', label: 'Memory ID', type: 'string', required: true, placeholder: '550e8400-e29b-41d4-a716-446655440000' },
    ],
    bodyFields: [],
    description: 'Retrieve a single memory by ID.',
  },
  {
    id: 'export-memories',
    label: 'Export Memories',
    group: 'Memory',
    method: 'POST',
    pathTemplate: '/v1/memory/export',
    pathParams: [],
    bodyFields: [
      { name: 'limit', label: 'Limit', type: 'number', placeholder: '20', defaultValue: '20' },
      { name: 'format', label: 'Format', type: 'string', placeholder: 'json', defaultValue: 'json' },
      { name: 'include_graph', label: 'Include Graph (true/false)', type: 'json', placeholder: 'false', defaultValue: 'false' },
      { name: 'include_episodes', label: 'Include Episodes (true/false)', type: 'json', placeholder: 'false', defaultValue: 'false' },
    ],
    description: 'Export recent memories to quickly inspect real data and IDs.',
  },
  {
    id: 'write-memory',
    label: 'Write Memory',
    group: 'Memory',
    method: 'POST',
    pathTemplate: '/v1/memory',
    pathParams: [],
    bodyFields: [
      { name: 'user_id', label: 'User ID', type: 'string', placeholder: '550e8400-e29b-41d4-a716-446655440000' },
      { name: 'content', label: 'Content', type: 'textarea', required: true, placeholder: 'User likes concise replies', defaultValue: 'User likes concise replies' },
    ],
    description: 'Store a new memory.',
  },
  {
    id: 'batch-write',
    label: 'Batch Write',
    group: 'Memory',
    method: 'POST',
    pathTemplate: '/v1/memory/batch',
    pathParams: [],
    bodyFields: [
      {
        name: '_rootBody',
        label: 'Memories (JSON array)',
        type: 'json',
        required: true,
        placeholder: '[{"content":"Likes TypeScript"}]',
        defaultValue: '[{"content":"Likes TypeScript"},{"content":"Prefers dark mode"}]',
      },
    ],
    description: 'Store multiple memories in one request.',
  },
  {
    id: 'update-memory',
    label: 'Update Memory',
    group: 'Memory',
    method: 'PUT',
    pathTemplate: '/v1/memory/:id',
    pathParams: [
      { name: 'id', label: 'Memory ID', type: 'string', required: true, placeholder: '550e8400-e29b-41d4-a716-446655440000' },
    ],
    bodyFields: [
      { name: 'content', label: 'Content', type: 'textarea', required: true, placeholder: 'Updated content', defaultValue: 'User prefers detailed explanations' },
    ],
    description: 'Update an existing memory.',
  },
  {
    id: 'delete-memory',
    label: 'Delete Memory',
    group: 'Memory',
    method: 'DELETE',
    pathTemplate: '/v1/memory/:id',
    pathParams: [
      { name: 'id', label: 'Memory ID', type: 'string', required: true, placeholder: '550e8400-e29b-41d4-a716-446655440000' },
    ],
    bodyFields: [],
    description: 'Delete a memory by ID.',
  },
  // --- Graph ---
  {
    id: 'list-entities',
    label: 'List Entities',
    group: 'Graph',
    method: 'GET',
    pathTemplate: '/v1/graph/entities',
    pathParams: [],
    bodyFields: [],
    description: 'List all graph entities.',
  },
  {
    id: 'get-entity',
    label: 'Get Entity',
    group: 'Graph',
    method: 'GET',
    pathTemplate: '/v1/graph/entities/:id',
    pathParams: [
      { name: 'id', label: 'Entity ID', type: 'string', required: true, placeholder: '550e8400-e29b-41d4-a716-446655440000' },
    ],
    bodyFields: [],
    description: 'Get a single entity by ID.',
  },
  {
    id: 'get-edges',
    label: 'Get Edges',
    group: 'Graph',
    method: 'GET',
    pathTemplate: '/v1/graph/entities/:id/edges',
    pathParams: [
      { name: 'id', label: 'Entity ID', type: 'string', required: true, placeholder: '550e8400-e29b-41d4-a716-446655440000' },
    ],
    bodyFields: [],
    description: 'Get edges for an entity.',
  },
  {
    id: 'shortest-path',
    label: 'Shortest Path',
    group: 'Graph',
    method: 'GET',
    pathTemplate: '/v1/graph/path/:from/:to',
    pathParams: [
      { name: 'from', label: 'From Entity ID', type: 'string', required: true, placeholder: '550e8400-e29b-41d4-a716-446655440001' },
      { name: 'to', label: 'To Entity ID', type: 'string', required: true, placeholder: '550e8400-e29b-41d4-a716-446655440002' },
    ],
    bodyFields: [],
    description: 'Find shortest path between two entities.',
  },
  // --- Admin ---
  {
    id: 'create-webhook',
    label: 'Create Webhook',
    group: 'Admin',
    method: 'POST',
    pathTemplate: '/v1/webhooks',
    pathParams: [],
    bodyFields: [
      { name: 'url', label: 'Webhook URL', type: 'string', required: true, placeholder: 'https://example.com/hook', defaultValue: 'https://example.com/hook' },
      { name: 'events', label: 'Events (JSON array)', type: 'json', placeholder: '["memory.created"]', defaultValue: '["memory.created","memory.updated"]' },
    ],
    description: 'Register a webhook for events.',
  },
  {
    id: 'audit-log',
    label: 'Audit Log',
    group: 'Admin',
    method: 'GET',
    pathTemplate: '/v1/admin/audit',
    pathParams: [],
    bodyFields: [],
    description: 'View the audit log.',
  },
];
