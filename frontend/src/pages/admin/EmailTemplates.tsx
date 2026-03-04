import { Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { emailTemplatesApi, type EmailTemplate } from '../../api/client';
import { useState } from 'react';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

export default function EmailTemplates() {
  const queryClient = useQueryClient();
  const [editingStatus, setEditingStatus] = useState<string | null>(null);
  const [form, setForm] = useState<Partial<EmailTemplate>>({});

  const { data: templates, isLoading } = useQuery({
    queryKey: ['email-templates'],
    queryFn: () => emailTemplatesApi.getAll().then((r) => r.data),
  });

  const saveMutation = useMutation({
    mutationFn: ({ status, body }: { status: string; body: Partial<EmailTemplate> }) =>
      emailTemplatesApi.save(status, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['email-templates'] });
      setEditingStatus(null);
    },
  });

  const startEdit = (t: EmailTemplate) => {
    setEditingStatus(t.status ?? null);
    setForm({
      description: t.description ?? undefined,
      template_body: t.template_body ?? undefined,
      subject: t.subject ?? undefined,
      to_recipients: t.to_recipients ?? undefined,
      cc: t.cc ?? undefined,
      bcc: t.bcc ?? undefined,
      is_active: t.is_active,
    });
  };

  const handleSave = () => {
    if (!editingStatus) return;
    saveMutation.mutate({ status: editingStatus, body: form });
  };

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/admin/config" style={{ color: '#3498db' }}>← Back to Settings</Link>
      </div>
      <h1>Email Templates</h1>
      <div style={cardStyle}>
        <p style={{ color: '#7f8c8d', marginBottom: '16px' }}>
          Edit templates by status. Click <strong>Edit</strong> to change body, subject, and recipients.
        </p>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead style={{ backgroundColor: '#34495e', color: 'white' }}>
            <tr>
              <th style={{ padding: '12px', textAlign: 'left' }}>ID</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Status</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Description</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Active</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {templates?.map((t) => (
              <tr key={t.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
                <td style={{ padding: '12px' }}>{t.id}</td>
                <td style={{ padding: '12px' }}>{t.status ?? '-'}</td>
                <td style={{ padding: '12px' }}>{t.description ?? '-'}</td>
                <td style={{ padding: '12px' }}>{t.is_active ? 'Yes' : 'No'}</td>
                <td style={{ padding: '12px' }}>
                  <button
                    type="button"
                    onClick={() => startEdit(t)}
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
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {(!templates || templates.length === 0) && (
          <p style={{ color: '#7f8c8d', marginTop: '20px' }}>No email templates.</p>
        )}
      </div>

      {editingStatus && (
        <div style={{ ...cardStyle, marginTop: '24px' }}>
          <h3>Edit template: {editingStatus}</h3>
          <div style={{ display: 'grid', gap: '12px', maxWidth: '600px' }}>
            <label>
              Description
              <input
                value={form.description ?? ''}
                onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
                style={{ display: 'block', padding: '8px', width: '100%' }}
              />
            </label>
            <label>
              Subject
              <input
                value={form.subject ?? ''}
                onChange={(e) => setForm((f) => ({ ...f, subject: e.target.value }))}
                style={{ display: 'block', padding: '8px', width: '100%' }}
              />
            </label>
            <label>
              To recipients
              <input
                value={form.to_recipients ?? ''}
                onChange={(e) => setForm((f) => ({ ...f, to_recipients: e.target.value }))}
                style={{ display: 'block', padding: '8px', width: '100%' }}
              />
            </label>
            <label>
              CC
              <input
                value={form.cc ?? ''}
                onChange={(e) => setForm((f) => ({ ...f, cc: e.target.value }))}
                style={{ display: 'block', padding: '8px', width: '100%' }}
              />
            </label>
            <label>
              BCC
              <input
                value={form.bcc ?? ''}
                onChange={(e) => setForm((f) => ({ ...f, bcc: e.target.value }))}
                style={{ display: 'block', padding: '8px', width: '100%' }}
              />
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <input
                type="checkbox"
                checked={form.is_active ?? false}
                onChange={(e) => setForm((f) => ({ ...f, is_active: e.target.checked }))}
              />
              Active
            </label>
            <label>
              Template body
              <textarea
                value={form.template_body ?? ''}
                onChange={(e) => setForm((f) => ({ ...f, template_body: e.target.value }))}
                rows={6}
                style={{ display: 'block', padding: '8px', width: '100%', fontFamily: 'inherit' }}
              />
            </label>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                type="button"
                onClick={handleSave}
                disabled={saveMutation.isPending}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#27ae60',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                {saveMutation.isPending ? 'Saving...' : 'Save'}
              </button>
              <button
                type="button"
                onClick={() => setEditingStatus(null)}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#95a5a6',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
