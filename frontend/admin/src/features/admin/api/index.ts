export { apiFetch } from './client';
export type { FetchOptions } from './client';

export {
  getAuthToken,
  getAuthUser,
  setAuthSession,
  setAuthUser,
  clearAuthSession,
} from './session';

export { authAPI } from './auth';
export type { AdminAuthUser } from './auth';

export { configAPI } from './config';
export type { Config } from './config';

export { credentialsAPI } from './credentials';
export type { Credential } from './credentials';

export { campaignsAPI } from './campaigns';
export type { Campaign, CampaignLog, MarketingStats } from './campaigns';

export { tenantsAPI } from './tenants';
export type { Tenant } from './tenants';

export { statusAPI } from './status';
export type { ServiceStatus, DatabaseStatus, SystemStatus } from './status';

export { usersAPI } from './users';
export type { AdminUser } from './users';

export { auditAPI } from './audit';
export type { AuditLog } from './audit';
