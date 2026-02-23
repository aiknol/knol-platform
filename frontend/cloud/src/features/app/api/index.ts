export type { AppUser, TenantProfile, ApiKeyItem, TenantUser, TenantAuditItem, InviteItem, UsageHistoryItem } from './types';

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

export { appBillingAPI } from './billing';
export type {
  SubscriptionInfo,
  CheckoutResponse,
  PortalResponse,
  CancelResponse,
  ReactivateResponse,
  Invoice,
  InvoicesResponse,
  UpcomingInvoice,
  UsageInfo,
} from './billing';

export { appInvitesAPI } from './invites';
export type {
  CreateInvitePayload,
  CreateInviteResponse,
  RevokeInviteResponse,
} from './invites';

export { appSettingsAPI } from './settings';
export type {
  UpdateTenantPayload,
  UpdateTenantResponse,
  UpdateProfilePayload,
  UpdateProfileResponse,
  ChangePasswordPayload,
  ChangePasswordResponse,
} from './settings';
