import type { InviteItem } from './types';
import { apiFetch } from './client';

export interface CreateInvitePayload {
  email: string;
  role?: 'admin' | 'developer' | 'viewer';
}

export interface CreateInviteResponse {
  id: string;
  email: string;
  role: string;
  token: string;
  expires_at: string;
}

export interface RevokeInviteResponse {
  id: string;
  revoked: boolean;
}

export const appInvitesAPI = {
  list: async (): Promise<InviteItem[]> => {
    const res = await apiFetch<{ data: InviteItem[] }>('/app/invites', { method: 'GET' });
    return res.data;
  },

  create: async (payload: CreateInvitePayload) =>
    apiFetch<CreateInviteResponse>('/app/invites', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  revoke: async (id: string) =>
    apiFetch<RevokeInviteResponse>(`/app/invites/${id}`, {
      method: 'DELETE',
    }),
};
