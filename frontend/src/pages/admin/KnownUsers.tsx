import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { knownUsersApi, type KnownUser } from '../../api/client';
import { useState } from 'react';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

export default function KnownUsers() {
  const queryClient = useQueryClient();
  const [emailInput, setEmailInput] = useState('');

  const { data: users, isLoading } = useQuery({
    queryKey: ['known-users'],
    queryFn: () => knownUsersApi.getAll().then((r) => r.data),
  });

  const saveMutation = useMutation({
    mutationFn: (list: { user_email: string; role_id: number }[]) =>
      knownUsersApi.saveAll(list),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['known-users'] });
      setLocalList([]);
    },
  });

  const [localList, setLocalList] = useState<{ user_email: string; role_id: number }[]>([]);
  const displayList = (users ?? []).map((u) => ({ user_email: u.user_email, role_id: u.role_id })).concat(localList);

  const handleAdd = () => {
    if (!emailInput.trim()) return;
    setLocalList((prev) => prev.concat({ user_email: emailInput.trim(), role_id: 1 }));
    setEmailInput('');
  };

  const handleRemove = (index: number) => {
    const next = displayList.filter((_, i) => i !== index);
    if (index < (users?.length ?? 0)) {
      saveMutation.mutate(next);
    } else {
      setLocalList((prev) => prev.filter((_, i) => i !== index - (users?.length ?? 0)));
    }
  };

  const handleSaveAll = () => {
    const toSave = displayList.filter((u) => u.user_email.trim() !== '');
    saveMutation.mutate(toSave);
    setLocalList([]);
  };

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h1>Known Users</h1>
      <div style={cardStyle}>
        <p style={{ color: '#7f8c8d', marginBottom: '16px' }}>
          Add or remove users allowed to access the admin portal. Click <strong>Save All</strong> to persist.
        </p>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '2px solid #ecf0f1' }}>
              <th style={{ padding: '12px', textAlign: 'left' }}>User Email</th>
              <th style={{ padding: '12px', width: '100px' }}>Action</th>
            </tr>
          </thead>
          <tbody>
            {displayList.map((u, index) => (
              <tr key={`${u.user_email}-${index}`} style={{ borderBottom: '1px solid #ecf0f1' }}>
                <td style={{ padding: '12px' }}>{u.user_email}</td>
                <td style={{ padding: '12px' }}>
                  <button
                    type="button"
                    onClick={() => handleRemove(index)}
                    style={{
                      padding: '6px 12px',
                      backgroundColor: '#e74c3c',
                      color: 'white',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: 'pointer',
                    }}
                  >
                    Remove
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
          <tfoot>
            <tr style={{ borderTop: '2px solid #ecf0f1' }}>
              <td style={{ padding: '12px' }}>
                <input
                  type="email"
                  value={emailInput}
                  onChange={(e) => setEmailInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAdd())}
                  placeholder="user@example.com"
                  style={{ padding: '8px', width: '100%', maxWidth: '320px' }}
                />
              </td>
              <td style={{ padding: '12px' }}>
                <button
                  type="button"
                  onClick={handleAdd}
                  style={{
                    padding: '6px 12px',
                    backgroundColor: '#3498db',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    marginRight: '8px',
                  }}
                >
                  Add
                </button>
              </td>
            </tr>
          </tfoot>
        </table>
        <button
          type="button"
          onClick={handleSaveAll}
          disabled={saveMutation.isPending}
          style={{
            marginTop: '16px',
            padding: '10px 20px',
            backgroundColor: '#27ae60',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {saveMutation.isPending ? 'Saving...' : 'Save All'}
        </button>
      </div>
    </div>
  );
}
