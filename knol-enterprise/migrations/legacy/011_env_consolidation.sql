-- Migration 011: Environment Consolidation
-- Moves service ports, MINIO settings, CORS config, and other env-only vars
-- into admin-configurable system_config with env var fallback.

-- ── Service Ports ───────────────────────────────────────────────────────────
INSERT INTO system_config (key, value, value_type, category, description, env_override)
VALUES
  ('services.gateway_port',       '8080', 'number', 'services',
   'Gateway service listen port',                          'GATEWAY_PORT'),
  ('services.write_port',         '8081', 'number', 'services',
   'Write service listen port',                            'WRITE_SERVICE_PORT'),
  ('services.retrieve_port',      '8082', 'number', 'services',
   'Retrieve service listen port',                         'RETRIEVE_SERVICE_PORT'),
  ('services.graph_port',         '8083', 'number', 'services',
   'Graph service listen port',                            'GRAPH_SERVICE_PORT'),
  ('services.admin_port',         '8084', 'number', 'services',
   'Admin service listen port',                            'ADMIN_SERVICE_PORT'),
  ('services.admin_panel_port',   '8084', 'number', 'services',
   'Admin panel proxy service listen port',                'ADMIN_PANEL_SERVICE_PORT'),
  ('services.billing_port',       '8086', 'number', 'services',
   'Billing service listen port',                          'BILLING_SERVICE_PORT'),
  ('services.ingest_port',        '8087', 'number', 'services',
   'Ingest service listen port',                           'INGEST_SERVICE_PORT'),

  -- CORS
  ('services.admin_cors_origin',  '"http://localhost:3006,http://localhost:3005,http://localhost:8080"', 'string', 'services',
   'Allowed CORS origins for admin panel and local demo UI', 'ADMIN_CORS_ORIGIN'),

  -- MinIO / S3
  ('storage.minio_endpoint',      '"http://localhost:9000"', 'string', 'storage',
   'MinIO/S3 endpoint URL',                                'MINIO_ENDPOINT'),
  ('storage.minio_bucket',        '"memorylayer"',           'string', 'storage',
   'MinIO/S3 bucket name',                                 'MINIO_BUCKET'),

  -- Database pool
  ('database.max_connections',    '20',   'number', 'database',
   'Maximum database pool connections',                    'DATABASE_MAX_CONNECTIONS'),
  ('database.min_connections',    '2',    'number', 'database',
   'Minimum database pool connections',                    'DATABASE_MIN_CONNECTIONS')
ON CONFLICT (key) DO NOTHING;
