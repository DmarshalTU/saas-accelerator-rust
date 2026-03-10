import { Link, useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { offersApi, type OfferAttribute } from '../../api/client';
import { useState, useEffect } from 'react';

const cardStyle = { backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' };

type EditRow = OfferAttribute & { _new?: boolean };

const emptyAttr = (offerId: number): EditRow => ({
  id: 0, offer_id: offerId, parameter_id: '', display_name: '', description: '', type: 'input', values_list: '', create_date: null, _new: true,
});

function Feedback({ ok, err }: { ok: string | null; err: string | null }) {
  if (ok)  return <div style={{ padding: '8px 12px', backgroundColor: '#d4edda', color: '#155724', borderRadius: '4px', marginTop: '8px' }}>{ok}</div>;
  if (err) return <div style={{ padding: '8px 12px', backgroundColor: '#f8d7da', color: '#721c24', borderRadius: '4px', marginTop: '8px' }}>{err}</div>;
  return null;
}

export default function OfferDetail() {
  const { guid } = useParams<{ guid: string }>();
  const queryClient = useQueryClient();
  const [attributes, setAttributes] = useState<EditRow[]>([]);
  const [feedback, setFeedback] = useState<{ ok?: string; err?: string }>({});

  const { data, isLoading } = useQuery({
    queryKey: ['offer', guid],
    queryFn: () => offersApi.getByGuid(guid!).then((r) => r.data),
    enabled: !!guid,
  });

  useEffect(() => {
    if (data?.attributes) setAttributes(data.attributes.map((a) => ({ ...a })));
  }, [data?.attributes]);

  const saveMutation = useMutation({
    mutationFn: () =>
      offersApi.saveAttributes(
        guid!,
        attributes.map((a) => ({
          id: a._new ? undefined : a.id,
          parameter_id: a.parameter_id ?? undefined,
          display_name: a.display_name ?? undefined,
          description: a.description ?? undefined,
          type: a.type ?? undefined,
          values_list: a.values_list ?? undefined,
        })),
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['offer', guid] });
      setFeedback({ ok: 'Attributes saved.' });
    },
    onError: (e: unknown) => setFeedback({ err: `Save failed: ${(e as Error).message}` }),
  });

  const deleteMutation = useMutation({
    mutationFn: ({ attrId }: { attrId: number }) => offersApi.deleteAttribute(guid!, attrId),
    onSuccess: (_data, { attrId }) => {
      setAttributes((prev) => prev.filter((a) => a.id !== attrId));
      queryClient.invalidateQueries({ queryKey: ['offer', guid] });
      setFeedback({ ok: 'Attribute deleted.' });
    },
    onError: (e: unknown) => setFeedback({ err: `Delete failed: ${(e as Error).message}` }),
  });

  const updateAttr = (index: number, field: keyof OfferAttribute, value: string) => {
    setAttributes((prev) => prev.map((a, i) => (i === index ? { ...a, [field]: value || null } : a)));
  };

  const addRow = () => {
    setAttributes((prev) => [...prev, emptyAttr(data?.offer.id ?? 0)]);
    setFeedback({});
  };

  const removeRow = (index: number) => {
    const row = attributes[index];
    if (row._new) {
      setAttributes((prev) => prev.filter((_, i) => i !== index));
    } else {
      if (window.confirm('Delete this attribute? This cannot be undone.')) {
        deleteMutation.mutate({ attrId: row.id });
      }
    }
  };

  if (!guid)    return <div>Missing offer GUID</div>;
  if (isLoading) return <div>Loading…</div>;
  if (!data)    return <div>Offer not found</div>;

  const { offer } = data;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/admin/offers" style={{ color: '#3498db' }}>← Back to Offers</Link>
      </div>
      <h1>Offer: {offer.offer_name || offer.offer_id}</h1>

      <div style={cardStyle}>
        <p><strong>Offer ID:</strong> {offer.offer_id}</p>
        <p><strong>GUID:</strong> {offer.offer_guid}</p>
        <p><strong>Name:</strong> {offer.offer_name ?? '-'}</p>
      </div>

      <div style={{ ...cardStyle, marginTop: '20px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
          <h3 style={{ margin: 0 }}>Attributes</h3>
          <button
            type="button"
            onClick={addRow}
            style={{ padding: '6px 14px', backgroundColor: '#3498db', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
          >
            + Add attribute
          </button>
        </div>

        {attributes.length === 0
          ? <p style={{ color: '#7f8c8d' }}>No attributes. Click "+ Add attribute" to create one.</p>
          : (
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', minWidth: '700px' }}>
                <thead style={{ backgroundColor: '#34495e', color: 'white' }}>
                  <tr>
                    {['Parameter ID', 'Display name', 'Type', 'Description', 'Values list', ''].map((h) => (
                      <th key={h} style={{ padding: '8px', textAlign: 'left', whiteSpace: 'nowrap' }}>{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {attributes.map((a, index) => (
                    <tr key={`${a.id}-${index}`} style={{ borderBottom: '1px solid #ecf0f1', backgroundColor: a._new ? '#f0f9ff' : undefined }}>
                      {(['parameter_id', 'display_name', 'type', 'description', 'values_list'] as (keyof OfferAttribute)[]).map((field) => (
                        <td key={field} style={{ padding: '6px' }}>
                          <input
                            value={(a[field] as string) ?? ''}
                            onChange={(e) => updateAttr(index, field, e.target.value)}
                            style={{ padding: '5px', width: '100%', minWidth: '80px' }}
                          />
                        </td>
                      ))}
                      <td style={{ padding: '6px', whiteSpace: 'nowrap' }}>
                        <button
                          type="button"
                          onClick={() => removeRow(index)}
                          disabled={deleteMutation.isPending}
                          style={{ padding: '4px 10px', backgroundColor: '#e74c3c', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
                        >
                          {a._new ? 'Cancel' : 'Delete'}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

        <div style={{ marginTop: '14px' }}>
          <button
            type="button"
            onClick={() => { setFeedback({}); saveMutation.mutate(); }}
            disabled={saveMutation.isPending}
            style={{ padding: '10px 20px', backgroundColor: '#27ae60', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
          >
            {saveMutation.isPending ? 'Saving…' : 'Save attributes'}
          </button>
        </div>
        <Feedback ok={feedback.ok ?? null} err={feedback.err ?? null} />
      </div>
    </div>
  );
}
