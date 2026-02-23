import { renderHook, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getAppAuthUser: vi.fn(),
  appUsersAPI: {
    list: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
  },
  appInvitesAPI: {
    list: vi.fn(),
    create: vi.fn(),
    revoke: vi.fn(),
  },
  appAuditAPI: {
    list: vi.fn(),
  },
  appAuthAPI: {
    me: vi.fn(),
  },
}));

vi.mock('@/features/app/api', () => ({
  getAppAuthUser: apiMocks.getAppAuthUser,
  appUsersAPI: apiMocks.appUsersAPI,
  appInvitesAPI: apiMocks.appInvitesAPI,
  appAuditAPI: apiMocks.appAuditAPI,
  appAuthAPI: apiMocks.appAuthAPI,
}));

import { useTeamState } from './useTeamState';

const OWNER_USER = { id: 'u1', email: 'owner@test.com', role: 'owner', tenant_id: 't1' };
const ADMIN_USER = { id: 'u2', email: 'admin@test.com', role: 'admin', tenant_id: 't1' };
const DEV_USER = { id: 'u3', email: 'dev@test.com', role: 'developer', tenant_id: 't1' };

const MOCK_USERS = [
  { id: 'm1', email: 'member@test.com', full_name: 'Member', role: 'developer', enabled: true, created_at: '2026-01-01', updated_at: '2026-01-01' },
];
const MOCK_INVITES = [
  { id: 'inv1', email: 'invite@test.com', role: 'developer', status: 'pending', expires_at: '2026-03-01', created_at: '2026-02-01' },
];
const MOCK_AUDIT = [
  { id: 'a1', action: 'create', resource_type: 'user', created_at: '2026-02-01' },
];

beforeEach(() => {
  vi.clearAllMocks();
  apiMocks.getAppAuthUser.mockReturnValue(OWNER_USER);
  apiMocks.appUsersAPI.list.mockResolvedValue(MOCK_USERS);
  apiMocks.appInvitesAPI.list.mockResolvedValue(MOCK_INVITES);
  apiMocks.appAuditAPI.list.mockResolvedValue(MOCK_AUDIT);
  apiMocks.appInvitesAPI.create.mockResolvedValue({ token: 'tok_abc' });
  apiMocks.appUsersAPI.create.mockResolvedValue({ id: 'new1' });
  apiMocks.appUsersAPI.update.mockResolvedValue({});
  apiMocks.appInvitesAPI.revoke.mockResolvedValue({});

  // Mock window.confirm to always return true
  vi.spyOn(window, 'confirm').mockReturnValue(true);
});

describe('useTeamState', () => {
  it('loads users, invites, audit for owner', async () => {
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe('');
    expect(result.current.canManage).toBe(true);
    expect(result.current.users).toHaveLength(1);
    expect(result.current.invites).toHaveLength(1);
    expect(result.current.auditLogs).toHaveLength(1);
    expect(apiMocks.appUsersAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appInvitesAPI.list).toHaveBeenCalledTimes(1);
    expect(apiMocks.appAuditAPI.list).toHaveBeenCalledTimes(1);
  });

  it('loads team data for admin', async () => {
    apiMocks.getAppAuthUser.mockReturnValue(ADMIN_USER);
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.canManage).toBe(true);
    expect(apiMocks.appUsersAPI.list).toHaveBeenCalledTimes(1);
  });

  it('skips team data for developer', async () => {
    apiMocks.getAppAuthUser.mockReturnValue(DEV_USER);
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.canManage).toBe(false);
    expect(apiMocks.appUsersAPI.list).not.toHaveBeenCalled();
    expect(apiMocks.appInvitesAPI.list).not.toHaveBeenCalled();
    expect(apiMocks.appAuditAPI.list).not.toHaveBeenCalled();
  });

  it('onInvite creates invite and refreshes', async () => {
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Set invite form fields
    act(() => {
      result.current.setInviteEmail('new@test.com');
      result.current.setInviteRole('admin');
    });

    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onInvite(mockEvent);
    });

    expect(mockEvent.preventDefault).toHaveBeenCalled();
    expect(apiMocks.appInvitesAPI.create).toHaveBeenCalledWith({ email: 'new@test.com', role: 'admin' });
    expect(result.current.newInviteToken).toBe('tok_abc');
  });

  it('onRevokeInvite calls revoke and reloads', async () => {
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.onRevokeInvite('inv1');
    });

    expect(apiMocks.appInvitesAPI.revoke).toHaveBeenCalledWith('inv1');
    // Reloads invites + audit
    expect(apiMocks.appInvitesAPI.list).toHaveBeenCalledTimes(2); // initial + after revoke
    expect(apiMocks.appAuditAPI.list).toHaveBeenCalledTimes(2);
  });

  it('onCreateUser creates user and reloads', async () => {
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => {
      result.current.setNewUserName('New User');
      result.current.setNewUserEmail('newuser@test.com');
      result.current.setNewUserPassword('StrongPass123!@');
      result.current.setNewUserRole('developer');
    });

    const mockEvent = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    await act(async () => {
      await result.current.onCreateUser(mockEvent);
    });

    expect(apiMocks.appUsersAPI.create).toHaveBeenCalledWith({
      full_name: 'New User',
      email: 'newuser@test.com',
      password: 'StrongPass123!@',
      role: 'developer',
    });
  });

  it('onToggleUser toggles enabled status', async () => {
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    const member = { id: 'm1', email: 'member@test.com', full_name: 'Member', role: 'developer', enabled: true, created_at: '2026-01-01', updated_at: '2026-01-01' };
    await act(async () => {
      await result.current.onToggleUser(member);
    });

    expect(apiMocks.appUsersAPI.update).toHaveBeenCalledWith('m1', { enabled: false });
  });

  it('handles invite list API failure gracefully', async () => {
    apiMocks.appInvitesAPI.list.mockRejectedValue(new Error('Invite service unavailable'));
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    // Users and audit should still load; invites falls back to []
    expect(result.current.error).toBe('');
    expect(result.current.users).toHaveLength(1);
    expect(result.current.invites).toHaveLength(0);
    expect(result.current.auditLogs).toHaveLength(1);
  });

  it('surfaces load error', async () => {
    apiMocks.appUsersAPI.list.mockRejectedValue(new Error('Database unavailable'));
    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe('Database unavailable');
  });

  it('copyToClipboard writes to clipboard', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });

    const { result } = renderHook(() => useTeamState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.copyToClipboard('test-value');
    });

    expect(writeText).toHaveBeenCalledWith('test-value');
  });
});
