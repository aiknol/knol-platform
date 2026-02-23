import { apiFetch } from './client';

export interface UpdateTenantPayload {
  name: string;
}

export interface UpdateTenantResponse {
  updated: boolean;
  name: string;
}

export interface UpdateProfilePayload {
  full_name: string;
}

export interface UpdateProfileResponse {
  updated: boolean;
  full_name: string;
}

export interface ChangePasswordPayload {
  current_password: string;
  new_password: string;
}

export interface ChangePasswordResponse {
  password_changed: boolean;
  token: string;
  expires_at: string;
}

export const appSettingsAPI = {
  updateTenant: async (payload: UpdateTenantPayload) =>
    apiFetch<UpdateTenantResponse>('/app/settings/tenant', {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),

  updateProfile: async (payload: UpdateProfilePayload) =>
    apiFetch<UpdateProfileResponse>('/app/settings/profile', {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),

  changePassword: async (payload: ChangePasswordPayload) =>
    apiFetch<ChangePasswordResponse>('/app/settings/change-password', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
};
