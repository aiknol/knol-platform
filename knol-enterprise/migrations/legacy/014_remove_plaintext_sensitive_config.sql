-- Remove sensitive plaintext config keys from system_config.
-- Secrets must be provided via environment variables or encrypted system_credentials.

DELETE FROM system_config
WHERE key IN (
  'gateway.jwt_secret',
  'storage.minio_access_key',
  'storage.minio_secret_key'
);
