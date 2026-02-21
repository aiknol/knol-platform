export type { AppUser, TenantProfile, ApiKeyItem, TenantUser, TenantAuditItem } from './types';

export { apiFetch } from './client';
export type { FetchOptions } from './client';

export {
  getAppAuthToken,
  getAppAuthUser,
  getAppTenant,
  setAppAuthSession,
  setAppProfile,
  clearAppAuthSession,
  consumeInitialApiKey,
} from './session';

export { appAuthAPI } from './auth';
export type { SignupPayload, SignupResponse, LoginResponse, MeResponse } from './auth';

export { appTenantAPI } from './tenant';

export { appApiKeysAPI } from './api-keys';
export type {
  CreateApiKeyPayload,
  CreateApiKeyResponse,
  RevokeApiKeyResponse,
} from './api-keys';

export { appUsersAPI } from './users';
export type {
  CreateTenantUserPayload,
  CreateTenantUserResponse,
  UpdateTenantUserPayload,
  UpdateTenantUserResponse,
} from './users';

export { appAuditAPI } from './audit';
