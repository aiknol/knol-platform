'use client';

import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'next/navigation';
import { credentialsAPI, Credential, getAuthUser } from '@/features/admin/api';
import { useAdminFetch, useAdminAction } from '@/features/admin/hooks';
import { PageHeader, Loading, ErrorBanner, Button, AdminCard } from '@/features/admin/components';


interface EditingCredential {
  name: string;
  value: string;
  service?: string;
  description?: string;
}

export default function CredentialsPage() {
  const searchParams = useSearchParams();
  const initialSearch = searchParams.get('search') || '';

  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [editing, setEditing] = useState<EditingCredential | null>(null);
  const [testResults, setTestResults] = useState<Record<string, { status: string; message?: string }>>({});
  const [userRole, setUserRole] = useState<string>('');
  const [searchTerm, setSearchTerm] = useState(initialSearch);

  const { data: fetchedCredentials, loading: initialLoading, error: fetchError } = useAdminFetch(
    () => credentialsAPI.list(),
  );
  const { loading: saving, error: saveError, run: saveCredential } = useAdminAction();
  const { loading: testing, error: testError, run: testCredentialAction } = useAdminAction();
  const { error: deleteError, run: deleteCredentialAction } = useAdminAction();

  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [activeTest, setActiveTest] = useState<string | null>(null);

  // Sync fetched credentials to local state
  useEffect(() => {
    if (fetchedCredentials) setCredentials(fetchedCredentials);
  }, [fetchedCredentials]);

  // Get current user role
  useEffect(() => {
    const userData = getAuthUser();
    if (userData) {
      setUserRole(userData.role);
    }
  }, []);

  // Pick up search param from URL (e.g. from global search)
  useEffect(() => {
    const s = searchParams.get('search');
    if (s) setSearchTerm(s);
  }, [searchParams]);

  const filteredCredentials = useMemo(() => {
    if (!searchTerm.trim()) return credentials;
    const q = searchTerm.trim().toLowerCase();
    return credentials.filter((c) => {
      const searchable = `${c.name} ${c.service || ''} ${c.description || ''}`.toLowerCase();
      return searchable.includes(q);
    });
  }, [credentials, searchTerm]);

  const handleAdd = () => {
    setEditing({ name: '', value: '', service: '', description: '' });
    setShowModal(true);
    setError('');
  };

  const handleEdit = (credential: Credential) => {
    setEditing({
      name: credential.name,
      value: '',
      service: credential.service || '',
      description: credential.description || '',
    });
    setShowModal(true);
    setError('');
  };

  const handleSave = async () => {
    if (!editing || !editing.name || !editing.value) {
      setError('Name and value are required');
      return;
    }

    const success = await saveCredential(() =>
      credentialsAPI.update(editing.name, editing.value, editing.service, editing.description)
    );

    if (success) {
      setCredentials((prev) => {
        const exists = prev.find((c) => c.name === editing.name);
        if (exists) {
          return prev.map((c) =>
            c.name === editing.name
              ? { ...c, value: editing.value, service: editing.service, description: editing.description }
              : c
          );
        }
        return [...prev, { name: editing.name, value: editing.value, service: editing.service, description: editing.description }];
      });

      setMessage(`Credential "${editing.name}" saved successfully`);
      setShowModal(false);
      setEditing(null);
      setTimeout(() => setMessage(''), 3000);
    } else if (saveError) {
      setError(saveError);
    }
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`Are you sure you want to delete "${name}"?`)) return;

    const success = await deleteCredentialAction(() => credentialsAPI.delete(name));

    if (success) {
      setCredentials(credentials.filter((c) => c.name !== name));
      setMessage(`Credential "${name}" deleted successfully`);
      setTimeout(() => setMessage(''), 3000);
    } else if (deleteError) {
      setError(deleteError);
    }
  };

  const handleTest = async (name: string) => {
    setActiveTest(name);
    const result = await testCredentialAction(() => credentialsAPI.test(name));

    if (result) {
      setTestResults((prev) => ({ ...prev, [name]: result }));
    } else if (testError) {
      setTestResults((prev) => ({
        ...prev,
        [name]: { status: 'error', message: testError },
      }));
    }
    setActiveTest(null);
  };

  const displayError = fetchError || saveError || deleteError || testError;

  if (initialLoading) {
    return <Loading message="Loading credentials..." />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="Credentials"
        description="Manage service credentials and API keys"
        action={
          <Button onClick={handleAdd} variant="primary">
            Add Credential
          </Button>
        }
      />

      {displayError && <ErrorBanner message={displayError} />}

      {message && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <p className="text-green-400 text-sm">{message}</p>
        </div>
      )}

      {/* Search & Filter Bar */}
      <AdminCard>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
          <div className="flex-1">
            <label className="block text-xs text-dark-400 mb-1">Search credentials</label>
            <input
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search by name, service, or description..."
              className="w-full px-3 py-2 bg-dark-700/40 border border-dark-600/40 rounded-lg text-sm text-dark-100 focus:outline-none focus:border-brand-500/50 focus:ring-1 focus:ring-brand-500/30"
            />
          </div>
          {searchTerm && (
            <Button variant="secondary" onClick={() => setSearchTerm('')}>
              Clear
            </Button>
          )}
        </div>
        <p className="text-xs text-dark-500 mt-2">
          Showing {filteredCredentials.length} of {credentials.length} credentials
        </p>
      </AdminCard>

      {/* Modal */}
      {showModal && editing && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <AdminCard className="max-w-2xl w-full">
            <div className="p-6">
              <h2 className="text-2xl font-bold text-dark-100 mb-4">
                {credentials.find((c) => c.name === editing.name) ? 'Edit' : 'Add'} Credential
              </h2>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-dark-200 mb-2">Name</label>
                  <input
                    type="text"
                    value={editing.name}
                    onChange={(e) => setEditing({ ...editing, name: e.target.value })}
                    placeholder="e.g., gemini_api_key"
                    className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-dark-200 mb-2">Value</label>
                  <textarea
                    value={editing.value}
                    onChange={(e) => setEditing({ ...editing, value: e.target.value })}
                    placeholder="Enter the credential value (will be encrypted at rest)"
                    className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
                    rows={3}
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-dark-200 mb-2">Service</label>
                  <select
                    value={editing.service || ''}
                    onChange={(e) => setEditing({ ...editing, service: e.target.value })}
                    className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
                  >
                    <option value="">Select a service...</option>
                    <option value="gemini">Gemini</option>
                    <option value="openai">OpenAI</option>
                    <option value="anthropic">Anthropic</option>
                    <option value="twitter">Twitter</option>
                    <option value="github">GitHub</option>
                    <option value="devto">Dev.to</option>
                    <option value="reddit">Reddit</option>
                    <option value="linkedin">LinkedIn</option>
                    <option value="hashnode">Hashnode</option>
                    <option value="medium">Medium</option>
                    <option value="producthunt">Product Hunt</option>
                    <option value="email">Email</option>
                    <option value="other">Other</option>
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-dark-200 mb-2">Description</label>
                  <input
                    type="text"
                    value={editing.description || ''}
                    onChange={(e) => setEditing({ ...editing, description: e.target.value })}
                    placeholder="What is this credential for?"
                    className="w-full px-4 py-2 bg-dark-700/50 border border-dark-600/50 rounded-lg text-dark-100 focus:outline-none focus:border-brand-500/50"
                  />
                </div>

                {error && (
                  <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
                    <p className="text-red-400 text-sm">{error}</p>
                  </div>
                )}

                <div className="flex space-x-2 pt-4">
                  <Button onClick={handleSave} disabled={saving} variant="primary">
                    {saving ? 'Saving...' : 'Save'}
                  </Button>
                  <Button
                    onClick={() => {
                      setShowModal(false);
                      setEditing(null);
                    }}
                    variant="secondary"
                  >
                    Cancel
                  </Button>
                </div>
              </div>
            </div>
          </AdminCard>
        </div>
      )}

      {/* Credentials List */}
      <AdminCard>
        {filteredCredentials.length === 0 ? (
          <div className="p-12 text-center text-dark-400">
            {credentials.length === 0 ? (
              <div>
                <p className="mb-2">No credentials configured yet</p>
                <p className="text-sm text-dark-500">
                  Click &ldquo;Add Credential&rdquo; to store your first API key (e.g., <code className="text-brand-400">gemini_api_key</code>)
                </p>
              </div>
            ) : (
              <p>No credentials match &ldquo;{searchTerm}&rdquo;</p>
            )}
          </div>
        ) : (
          <div className="divide-y divide-dark-700/30">
            {filteredCredentials.map((credential) => (
              <div key={credential.name} className="p-6">
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <p className="font-mono text-brand-400 font-medium text-sm">{credential.name}</p>
                    {credential.service && (
                      <span className="inline-block mt-1 px-2 py-0.5 rounded-full text-[10px] bg-amber-500/10 border border-amber-500/30 text-amber-300">
                        {credential.service}
                      </span>
                    )}
                    {credential.description && (
                      <p className="text-sm text-dark-400 mt-1">{credential.description}</p>
                    )}
                  </div>
                  <div className="flex items-center space-x-2">
                    {credential.last_rotated && (
                      <span className="text-xs text-dark-500 whitespace-nowrap">
                        Last rotated: {new Date(credential.last_rotated).toLocaleDateString()}
                      </span>
                    )}
                  </div>
                </div>

                {/* Value */}
                <div className="mb-4">
                  <code className="block bg-dark-800/50 border border-dark-700/30 rounded p-3 text-dark-400 text-sm font-mono truncate">
                    {credential.value ? '••••••••' : '(no value set)'}
                  </code>
                </div>

                {/* Test Result */}
                {testResults[credential.name] && (
                  <div
                    className={`mb-4 p-3 rounded-lg text-sm ${
                      testResults[credential.name].status === 'success'
                        ? 'bg-green-500/10 text-green-400'
                        : 'bg-red-500/10 text-red-400'
                    }`}
                  >
                    {testResults[credential.name].message || testResults[credential.name].status}
                  </div>
                )}

                {/* Actions */}
                <div className="flex space-x-2">
                  <Button
                    onClick={() => handleTest(credential.name)}
                    disabled={activeTest === credential.name}
                    variant="secondary"
                    size="sm"
                  >
                    {activeTest === credential.name ? 'Testing...' : 'Test'}
                  </Button>
                  <Button
                    onClick={() => handleEdit(credential)}
                    variant="primary"
                    size="sm"
                  >
                    Edit
                  </Button>
                  {userRole === 'super_admin' && (
                    <Button
                      onClick={() => handleDelete(credential.name)}
                      variant="danger"
                      size="sm"
                    >
                      Delete
                    </Button>
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
