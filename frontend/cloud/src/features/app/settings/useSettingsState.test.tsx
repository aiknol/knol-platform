import { renderHook, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getAppAuthUser: vi.fn(),
  getAppTenant: vi.fn(),
  setAppProfile: vi.fn(),
  appAuthAPI: {
    me: vi.fn(),
  },
  appSettingsAPI: {
    updateTenant: vi.fn(),
    updateProfile: vi.fn(),
    changePassword: vi.fn(),
  },
}));

vi.mock('@/features/app/api', () => ({
  getAppAuthUser: apiMocks.getAppAuthUser,
  getAppTenant: apiMocks.getAppTenant,
  setAppProfile: apiMocks.setAppProfile,
  appAuthAPI: apiMocks.appAuthAPI,
  appSettingsAPI: apiMocks.appSettingsAPI,
}));

import { useSettingsState } from './useSettingsState';

const MOCK_USER = {
  id: 'user_1',
  email: 'test@example.com',
  full_name: 'Test User',
  role: 'owner',
  tenant_id: 'tenant_1',
  enabled: true,
};

const MOCK_TENANT = {
  id: 'tenant_1',
  name: 'Test Co',
  slug: 'test-co',
  plan: 'free',
};

function fakeFormEvent(): React.FormEvent {
  return { preventDefault: vi.fn() } as unknown as React.FormEvent;
}

beforeEach(() => {
  vi.clearAllMocks();
  apiMocks.getAppAuthUser.mockReturnValue(MOCK_USER);
  apiMocks.getAppTenant.mockReturnValue(MOCK_TENANT);
  apiMocks.appAuthAPI.me.mockResolvedValue({ user: MOCK_USER, tenant: MOCK_TENANT });
});

describe('useSettingsState', () => {
  it('loads user and tenant data', async () => {
    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.user?.email).toBe('test@example.com');
    expect(result.current.tenant?.name).toBe('Test Co');
    expect(result.current.workspaceName).toBe('Test Co');
    expect(result.current.fullName).toBe('Test User');
    expect(result.current.error).toBe('');
  });

  it('onSaveWorkspace updates tenant name', async () => {
    const updatedTenant = { ...MOCK_TENANT, name: 'New Name' };
    apiMocks.appSettingsAPI.updateTenant.mockResolvedValue(updatedTenant);
    apiMocks.appAuthAPI.me.mockResolvedValue({ user: MOCK_USER, tenant: updatedTenant });

    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      result.current.setWorkspaceName('New Name');
    });

    await act(async () => {
      await result.current.onSaveWorkspace(fakeFormEvent());
    });

    expect(apiMocks.appSettingsAPI.updateTenant).toHaveBeenCalledWith({ name: 'New Name' });
    expect(result.current.success).toBe('Workspace name updated.');
    expect(apiMocks.setAppProfile).toHaveBeenCalled();
  });

  it('onSaveProfile updates user name', async () => {
    const updatedUser = { ...MOCK_USER, full_name: 'New Name' };
    apiMocks.appSettingsAPI.updateProfile.mockResolvedValue(updatedUser);
    apiMocks.appAuthAPI.me.mockResolvedValue({ user: updatedUser, tenant: MOCK_TENANT });

    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      result.current.setFullName('New Name');
    });

    await act(async () => {
      await result.current.onSaveProfile(fakeFormEvent());
    });

    expect(apiMocks.appSettingsAPI.updateProfile).toHaveBeenCalledWith({ full_name: 'New Name' });
    expect(result.current.success).toBe('Profile updated.');
    expect(apiMocks.setAppProfile).toHaveBeenCalled();
  });

  it('onChangePassword validates password length (< 12 chars)', async () => {
    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      result.current.setCurrentPassword('OldPass123!@');
      result.current.setNewPassword('Short1!A');
      result.current.setConfirmPassword('Short1!A');
    });

    await act(async () => {
      await result.current.onChangePassword(fakeFormEvent());
    });

    expect(result.current.error).toBe('Password must be at least 12 characters.');
    expect(apiMocks.appSettingsAPI.changePassword).not.toHaveBeenCalled();
  });

  it('onChangePassword validates uppercase requirement', async () => {
    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      result.current.setCurrentPassword('OldPass123!@');
      result.current.setNewPassword('alllowercase1!');
      result.current.setConfirmPassword('alllowercase1!');
    });

    await act(async () => {
      await result.current.onChangePassword(fakeFormEvent());
    });

    expect(result.current.error).toBe('Password must include an uppercase letter.');
    expect(apiMocks.appSettingsAPI.changePassword).not.toHaveBeenCalled();
  });

  it('onChangePassword validates special char requirement', async () => {
    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      result.current.setCurrentPassword('OldPass123!@');
      result.current.setNewPassword('NoSpecialChar1A');
      result.current.setConfirmPassword('NoSpecialChar1A');
    });

    await act(async () => {
      await result.current.onChangePassword(fakeFormEvent());
    });

    expect(result.current.error).toBe('Password must include a special character.');
    expect(apiMocks.appSettingsAPI.changePassword).not.toHaveBeenCalled();
  });

  it('handles API failure on load', async () => {
    apiMocks.appAuthAPI.me.mockRejectedValue(new Error('Network error'));

    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe('Network error');
  });

  it('handles API failure on save workspace', async () => {
    apiMocks.appSettingsAPI.updateTenant.mockRejectedValue(new Error('Save failed'));

    const { result } = renderHook(() => useSettingsState());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.onSaveWorkspace(fakeFormEvent());
    });

    expect(result.current.error).toBe('Save failed');
  });
});
