import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { knownUsersApi, type KnownUser } from '../../api/client';
import { useState } from 'react';

const ROLES: Record<number, string> = { 1: 'Admin', 2: 'Customer' };

const cardStyle = { backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' };

type PendingUser = { user_email: string; role_id: number };

function isValidEmail(s: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s.trim());
}

export default function KnownUsers() {
  const queryClient = useQueryClient();
  const [emailInput, setEmailInput]   = useState('');
  const [roleInput, setRoleInput]     = useState(1);
  const [emailError, setEmailError]   = useState('');
  const [localAdded, setLocalAdded]   = useState<PendingUser[]>([]);
  const [feedback, setFeedback]       = useState<{ ok?: string; err?: string }>({});

  const { data: users, isLoading } = useQuery({
    queryKey: ['known-users'],
    queryFn: () => knownUsersApi.getAll().then((r) => r.data),
  });

  const saveMutation = useMutation({
    mutationFn: (list: PendingUser[]) => knownUsersApi.saveAll(list),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['known-users'] });
      setLocalAdded([]);
      setFeedback({ ok: 'Saved successfully.' });
    },
    onError: (e: unknown) => setFeedback({ err: `Save failed: ${(e as Error).message}` }),
  });

  const dbUsers: PendingUser[] = (users ?? []).map((u: KnownUser) => ({ user_email: u.user_email, role_id: u.role_id }));
  const displayList = [...dbUsers, ...localAdded];

  const handleAdd = () => {
    const email = emailInput.trim();
    if (!isValidEmail(email)) { setEmailError('Enter a valid email address.'); return; }
    if (displayList.some((u) => u.user_email.toLowerCase() === email.toLowerCase())) {
      setEmailError('This email is already in the list.'); return;
    }
    setLocalAdded((prev) => [...prev, { user_email: email, role_id: roleInput }]);
    setEmailInput('');
    setEmailError('');
    setFeedback({});
  };

  const handleRemove = (index: number) => {
    const next = displayList.filter((_, i) => i !== index);
    saveMutation.mutate(next);
  };

  const handleSaveAll = () => {
    setFeedback({});
    saveMutation.mutate(displayList.filter((u) => u.user_email.trim() !== ''));
  };

  if (isLoading) return <div>Loading…</div>;

  return (
    <div>
      <h1>Known Users (Admin access)</h1>
      <div style={cardStyle}>
        <p style={{ color: '#7f8c8d', marginBottom: '16px' }}>
          Only users listed here can access the admin portal (when Azure AD authentication is enabled).
          Role <strong>Admin (1)</strong> grants full access.
        </p>

        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '2px solid #ecf0f1', backgroundColor: '#f8f9fa' }}>
              <th style={{ padding: '10px 12px', textAlign: 'left' }}>Email</th>
              <th style={{ padding: '10px 12px', textAlign: 'left', width: '120px' }}>Role</th>
              <th style={{ padding: '10px 12px', width: '90px' }}></th>
            </tr>
          </thead>
          <tbody>
            {displayList.length === 0 && (
              <tr><td colSpan={3} style={{ padding: '12px', color: '#7f8c8d', textAlign: 'center' }}>No users. Add one below.</td></tr>
            )}
            {displayList.map((u, index) => {
              const isPending = index >= dbUsers.length;
              return (
                <tr key={`${u.user_email}-${index}`} style={{ borderBottom: '1px solid #ecf0f1', backgroundColor: isPending ? '#f0f9ff' : undefined }}>
                  <td style={{ padding: '10px 12px' }}>
                    {u.user_email}
                    {isPending && <span style={{ marginLeft: '6px', fontSize: '11px', color: '#3498db' }}>unsaved</span>}
                  </td>
                  <td style={{ padding: '10px 12px', color: '#555' }}>
                    {ROLES[u.role_id] ?? `Role ${u.role_id}`}
                  </td>
                  <td style={{ padding: '10px 12px', textAlign: 'center' }}>
                    <button
                      type="button"
                      onClick={() => handleRemove(index)}
                      disabled={saveMutation.isPending}
                      style={{ padding: '5px 10px', backgroundColor: '#e74c3c', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer', fontSize: '13px' }}
                    >
                      Remove
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
          <tfoot>
            <tr style={{ borderTop: '2px solid #ecf0f1' }}>
              <td style={{ padding: '10px 12px' }}>
                <input
                  type="email"
                  value={emailInput}
                  onChange={(e) => { setEmailInput(e.target.value); setEmailError(''); }}
                  onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAdd())}
                  placeholder="user@example.com"
                  style={{ padding: '7px', width: '100%', maxWidth: '300px', borderRadius: '4px', border: emailError ? '1px solid #e74c3c' : '1px solid #ccc' }}
                />
                {emailError && <div style={{ color: '#e74c3c', fontSize: '12px', marginTop: '4px' }}>{emailError}</div>}
              </td>
              <td style={{ padding: '10px 12px' }}>
                <select
                  value={roleInput}
                  onChange={(e) => setRoleInput(Number(e.target.value))}
                  style={{ padding: '7px', borderRadius: '4px', border: '1px solid #ccc' }}
                >
                  {Object.entries(ROLES).map(([id, name]) => (
                    <option key={id} value={id}>{name}</option>
                  ))}
                </select>
              </td>
              <td style={{ padding: '10px 12px' }}>
                <button
                  type="button"
                  onClick={handleAdd}
                  style={{ padding: '7px 14px', backgroundColor: '#3498db', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
                >
                  Add
                </button>
              </td>
            </tr>
          </tfoot>
        </table>

        <div style={{ marginTop: '16px', display: 'flex', alignItems: 'center', gap: '12px' }}>
          <button
            type="button"
            onClick={handleSaveAll}
            disabled={saveMutation.isPending || localAdded.length === 0}
            style={{ padding: '10px 20px', backgroundColor: '#27ae60', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
          >
            {saveMutation.isPending ? 'Saving…' : `Save All${localAdded.length > 0 ? ` (${localAdded.length} pending)` : ''}`}
          </button>
          {localAdded.length > 0 && (
            <span style={{ color: '#e67e22', fontSize: '13px' }}>You have unsaved changes.</span>
          )}
        </div>

        {feedback.ok  && <div style={{ marginTop: '10px', padding: '8px 12px', backgroundColor: '#d4edda', color: '#155724', borderRadius: '4px' }}>{feedback.ok}</div>}
        {feedback.err && <div style={{ marginTop: '10px', padding: '8px 12px', backgroundColor: '#f8d7da', color: '#721c24', borderRadius: '4px' }}>{feedback.err}</div>}
      </div>
    </div>
  );
}
