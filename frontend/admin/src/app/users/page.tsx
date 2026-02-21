'use client';

import { useState, useEffect } from 'react';
import { usersAPI, AdminUser, getAuthUser } from '@/features/admin/api';
import { useAdminFetch, useAdminAction, useEditMode } from '@/features/admin/hooks';
import {
  PageHeader,
  Loading,
  ErrorBanner,
  Button,
  AdminCard,
  StatusBadge,
} from '@/features/admin/components';

interface NewUser {
  email: string;
  password: string;
  role: 'admin' | 'super_admin';
}

interface MessageState {
  show: boolean;
  text: string;
  type: 'success' | 'error';
}

export default function UsersPage() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [message, setMessage] = useState<MessageState>({ show: false, text: '', type: 'success' });
  const [showAddForm, setShowAddForm] = useState(false);
  const [newUser, setNewUser] = useState<NewUser>({
    email: '',
    password: '',
    role: 'admin',
  });
  const [editingRole, setEditingRole] = useState<'admin' | 'super_admin'>('admin');
  const [userRole, setUserRole] = useState<string>('');

  // Fetch users
  const { data: fetchedUsers, loading, error: fetchError, refetch } = useAdminFetch(
    () => usersAPI.list(),
  );

  // Sync fetched users to local state
  useEffect(() => {
    if (fetchedUsers) {
      setUsers(fetchedUsers);
    }
  }, [fetchedUsers]);

  // Get current user role
  useEffect(() => {
    const userData = getAuthUser();
    if (userData) {
      setUserRole(userData.role);
    }
  }, []);

  // Hooks
  const editMode = useEditMode<string>();
  const addAction = useAdminAction();
  const updateRoleAction = useAdminAction();
  const toggleEnabledAction = useAdminAction();
  const deleteAction = useAdminAction();

  const showNotification = (text: string, type: 'success' | 'error' = 'success') => {
    setMessage({ show: true, text, type });
    setTimeout(() => setMessage({ show: false, text: '', type: 'success' }), 3000);
  };

  const handleAddUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newUser.email || !newUser.password) {
      showNotification('Email and password are required', 'error');
      return;
    }

    const result = await addAction.run(() =>
      usersAPI.create(newUser.email, newUser.password, newUser.role),
    );

    if (result) {
      setUsers([...users, result]);
      showNotification(`User "${newUser.email}" created successfully`);
      setNewUser({ email: '', password: '', role: 'admin' });
      setShowAddForm(false);
    } else {
      showNotification(addAction.error || 'Failed to create user', 'error');
    }
  };

  const handleUpdateRole = async (userId: string) => {
    const result = await updateRoleAction.run(() => usersAPI.update(userId, editingRole));

    if (result) {
      setUsers(users.map((u) => (u.id === userId ? { ...u, role: editingRole } : u)));
      showNotification('User role updated successfully');
      editMode.stopEdit();
    } else {
      showNotification(updateRoleAction.error || 'Failed to update user', 'error');
    }
  };

  const handleToggleEnabled = async (userId: string, currentEnabled: boolean) => {
    const result = await toggleEnabledAction.run(() =>
      usersAPI.update(userId, undefined, !currentEnabled),
    );

    if (result) {
      setUsers(users.map((u) => (u.id === userId ? { ...u, enabled: !currentEnabled } : u)));
      showNotification(`User ${!currentEnabled ? 'enabled' : 'disabled'} successfully`);
    } else {
      showNotification(toggleEnabledAction.error || 'Failed to update user', 'error');
    }
  };

  const handleDeleteUser = async (userId: string, email: string) => {
    if (!confirm(`Are you sure you want to delete "${email}"?`)) return;

    const result = await deleteAction.run(() => usersAPI.delete(userId));

    if (result) {
      setUsers(users.filter((u) => u.id !== userId));
      showNotification(`User "${email}" deleted successfully`);
    } else {
      showNotification(deleteAction.error || 'Failed to delete user', 'error');
    }
  };

  if (loading) {
    return <Loading message="Loading admin users..." />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Admin Users"
        description="Manage admin user accounts and permissions"
        action={
          userRole === 'super_admin' && (
            <Button
              onClick={() => setShowAddForm(!showAddForm)}
              variant={showAddForm ? 'secondary' : 'primary'}
            >
              {showAddForm ? 'Cancel' : 'Add User'}
            </Button>
          )
        }
      />

      {fetchError && <ErrorBanner message={fetchError} onRetry={refetch} />}

      {message.show && (
        <div
          className={`p-4 rounded-lg border ${
            message.type === 'success'
              ? 'bg-green-500/10 border-green-500/20'
              : 'bg-red-500/10 border-red-500/20'
          }`}
        >
          <p
            className={`text-sm ${
              message.type === 'success' ? 'text-green-400' : 'text-red-400'
            }`}
          >
            {message.text}
          </p>
        </div>
      )}

      {/* Add User Form */}
      {showAddForm && (
        <AdminCard>
          <h2 className="text-xl font-semibold text-dark-50 mb-4">Add New Admin User</h2>
          <form onSubmit={handleAddUser} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-dark-200 mb-2">Email</label>
              <input
                type="email"
                value={newUser.email}
                onChange={(e) => setNewUser({ ...newUser, email: e.target.value })}
                placeholder="admin@example.com"
                required
                className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-dark-200 mb-2">Password</label>
              <input
                type="password"
                value={newUser.password}
                onChange={(e) => setNewUser({ ...newUser, password: e.target.value })}
                placeholder="Enter a strong password"
                required
                className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-dark-200 mb-2">Role</label>
              <select
                value={newUser.role}
                onChange={(e) =>
                  setNewUser({ ...newUser, role: e.target.value as 'admin' | 'super_admin' })
                }
                className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-50 focus:outline-none focus:border-brand-500/50"
              >
                <option value="admin">Admin</option>
                <option value="super_admin">Super Admin</option>
              </select>
            </div>

            <div className="flex space-x-2 pt-2">
              <Button
                type="submit"
                disabled={addAction.loading}
                variant="primary"
                size="md"
              >
                {addAction.loading ? 'Creating...' : 'Create User'}
              </Button>
              <Button
                type="button"
                onClick={() => setShowAddForm(false)}
                variant="secondary"
                size="md"
              >
                Cancel
              </Button>
            </div>
          </form>
        </AdminCard>
      )}

      {/* Users List */}
      <AdminCard>
        {users.length === 0 ? (
          <div className="p-12 text-center text-dark-400">
            <p>No admin users found</p>
          </div>
        ) : (
          <div className="divide-y divide-dark-600/30">
            {users.map((user) => (
              <div key={user.id} className="p-6">
                <div className="flex items-center justify-between mb-3">
                  <div>
                    <p className="font-semibold text-dark-50">{user.email}</p>
                    <p className="text-sm text-dark-400 mt-1">
                      ID: {user.id}
                      {user.created_at &&
                        ` • Created: ${new Date(user.created_at).toLocaleDateString()}`}
                    </p>
                  </div>
                  <div className="flex items-center space-x-3">
                    <StatusBadge status={user.enabled ? 'enabled' : 'disabled'} />
                    <span className="px-3 py-1 rounded-full text-xs font-semibold bg-brand-500/20 text-brand-400 capitalize">
                      {user.role.replace('_', ' ')}
                    </span>
                  </div>
                </div>

                <div className="flex flex-wrap gap-2 pt-3 border-t border-dark-600/30">
                  {editMode.isEditing(user.id) ? (
                    <>
                      <select
                        value={editingRole}
                        onChange={(e) =>
                          setEditingRole(e.target.value as 'admin' | 'super_admin')
                        }
                        className="px-3 py-1 bg-dark-700/50 border border-dark-600/50 rounded text-dark-50 text-sm focus:outline-none focus:border-brand-500/50"
                      >
                        <option value="admin">Admin</option>
                        <option value="super_admin">Super Admin</option>
                      </select>
                      <Button
                        onClick={() => handleUpdateRole(user.id)}
                        disabled={updateRoleAction.loading}
                        variant="primary"
                        size="sm"
                      >
                        {updateRoleAction.loading ? 'Saving...' : 'Save'}
                      </Button>
                      <Button
                        onClick={() => editMode.stopEdit()}
                        variant="secondary"
                        size="sm"
                      >
                        Cancel
                      </Button>
                    </>
                  ) : (
                    <>
                      {userRole === 'super_admin' && (
                        <>
                          <Button
                            onClick={() => {
                              editMode.startEdit(user.id);
                              setEditingRole(user.role);
                            }}
                            variant="ghost"
                            size="sm"
                            className="border border-brand-500/30 bg-brand-500/10 hover:bg-brand-500/20 text-brand-400"
                          >
                            Edit Role
                          </Button>
                          <button
                            onClick={() => handleToggleEnabled(user.id, user.enabled)}
                            className={`px-3 py-1 rounded text-sm font-medium border transition-colors ${
                              user.enabled
                                ? 'bg-red-600/20 hover:bg-red-600/30 text-red-400 border-red-500/30'
                                : 'bg-green-600/20 hover:bg-green-600/30 text-green-400 border-green-500/30'
                            }`}
                          >
                            {user.enabled ? 'Disable' : 'Enable'}
                          </button>
                          <Button
                            onClick={() => handleDeleteUser(user.id, user.email)}
                            variant="danger"
                            size="sm"
                          >
                            Delete
                          </Button>
                        </>
                      )}
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </AdminCard>
    </div>
  );
}
