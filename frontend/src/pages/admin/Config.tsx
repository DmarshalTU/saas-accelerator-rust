import { Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { configApi } from '../../api/client';
import { useState, useRef } from 'react';

export default function AdminConfig() {
  const queryClient = useQueryClient();
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [editValue, setEditValue] = useState('');
  const [uploadStatus, setUploadStatus] = useState<string | null>(null);
  const logoInputRef = useRef<HTMLInputElement>(null);
  const faviconInputRef = useRef<HTMLInputElement>(null);

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

  const uploadMutation = useMutation({
    mutationFn: ({ configName, base64 }: { configName: string; base64: string }) =>
      configApi.uploadFile(configName, base64),
    onSuccess: (_, { configName }) => {
      queryClient.invalidateQueries({ queryKey: ['config'] });
      setUploadStatus(`${configName} uploaded. Refresh (Ctrl+F5) to see changes.`);
      if (configName === 'LogoFile' && logoInputRef.current) logoInputRef.current.value = '';
      if (configName === 'FaviconFile' && faviconInputRef.current) faviconInputRef.current.value = '';
    },
    onError: () => setUploadStatus('Upload failed'),
  });

  const handleFileUpload = (configName: 'LogoFile' | 'FaviconFile', e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      const base64 = result.startsWith('data:') ? result.split(',')[1] : result;
      if (base64) uploadMutation.mutate({ configName, base64 });
    };
    reader.readAsDataURL(file);
  };

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
      <div style={{ marginBottom: '16px', padding: '12px 16px', backgroundColor: '#ecf0f1', borderRadius: '6px', fontSize: '14px' }}>
        <strong>What you can do:</strong> Edit any config value in the table (click Edit, change value, Save). Upload Logo (.png) or Favicon (.ico) below to change branding. <Link to="/admin/logs" style={{ color: '#3498db', marginRight: '12px' }}>View application logs</Link>
        <Link to="/admin/email-templates" style={{ color: '#3498db' }}>View email templates</Link>
      </div>
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

      <div style={{ marginTop: '32px', padding: '20px', backgroundColor: 'white', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
        <h2>Upload Logo / Favicon</h2>
        <p style={{ fontSize: '12px', color: '#7f8c8d' }}>Restart the application after uploading and refresh (Ctrl+F5) to see the new logo/favicon.</p>
        {uploadStatus && <p style={{ color: uploadStatus.startsWith('Upload failed') ? '#e74c3c' : '#27ae60' }}>{uploadStatus}</p>}
        <div style={{ display: 'flex', gap: '24px', flexWrap: 'wrap', marginTop: '12px' }}>
          <div>
            <label style={{ display: 'block', marginBottom: '4px' }}>Logo (.png)</label>
            <input ref={logoInputRef} type="file" accept=".png" onChange={(e) => handleFileUpload('LogoFile', e)} disabled={uploadMutation.isPending} />
          </div>
          <div>
            <label style={{ display: 'block', marginBottom: '4px' }}>Favicon (.ico)</label>
            <input ref={faviconInputRef} type="file" accept=".ico" onChange={(e) => handleFileUpload('FaviconFile', e)} disabled={uploadMutation.isPending} />
          </div>
        </div>
      </div>
    </div>
  );
}

