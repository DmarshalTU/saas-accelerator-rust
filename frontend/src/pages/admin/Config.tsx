import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { configApi } from '../../api/client';
import { useState } from 'react';

export default function AdminConfig() {
  const queryClient = useQueryClient();
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [editValue, setEditValue] = useState('');

  const { data: configs, isLoading } = useQuery({
    queryKey: ['config'],
    queryFn: () => configApi.getAll().then(res => res.data),
  });

  const updateMutation = useMutation({
    mutationFn: ({ name, value }: { name: string; value: string }) =>
      configApi.update(name, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['config'] });
      setEditingKey(null);
    },
  });

  const handleEdit = (name: string, currentValue: string) => {
    setEditingKey(name);
    setEditValue(currentValue || '');
  };

  const handleSave = (name: string) => {
    updateMutation.mutate({ name, value: editValue });
  };

  if (isLoading) {
    return <div>Loading configuration...</div>;
  }

  return (
    <div>
      <h1>Application Configuration</h1>
      <table style={{
        width: '100%',
        backgroundColor: 'white',
        borderRadius: '8px',
        overflow: 'hidden',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        marginTop: '20px',
      }}>
        <thead style={{ backgroundColor: '#34495e', color: 'white' }}>
          <tr>
            <th style={{ padding: '12px', textAlign: 'left' }}>Name</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Value</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Description</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {configs?.map((config) => (
            <tr key={config.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
              <td style={{ padding: '12px' }}>{config.name}</td>
              <td style={{ padding: '12px' }}>
                {editingKey === config.name ? (
                  <input
                    type="text"
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    style={{
                      padding: '6px',
                      border: '1px solid #bdc3c7',
                      borderRadius: '4px',
                      width: '100%',
                    }}
                  />
                ) : (
                  <span>{config.value || '-'}</span>
                )}
              </td>
              <td style={{ padding: '12px', color: '#7f8c8d' }}>
                {config.description || '-'}
              </td>
              <td style={{ padding: '12px' }}>
                {editingKey === config.name ? (
                  <>
                    <button
                      onClick={() => handleSave(config.name)}
                      style={{
                        padding: '6px 12px',
                        backgroundColor: '#27ae60',
                        color: 'white',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        marginRight: '8px',
                      }}
                    >
                      Save
                    </button>
                    <button
                      onClick={() => setEditingKey(null)}
                      style={{
                        padding: '6px 12px',
                        backgroundColor: '#95a5a6',
                        color: 'white',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                      }}
                    >
                      Cancel
                    </button>
                  </>
                ) : (
                  <button
                    onClick={() => handleEdit(config.name, config.value || '')}
                    style={{
                      padding: '6px 12px',
                      backgroundColor: '#3498db',
                      color: 'white',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: 'pointer',
                    }}
                  >
                    Edit
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

