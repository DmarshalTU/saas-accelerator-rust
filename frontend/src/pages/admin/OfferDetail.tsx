import { Link, useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { offersApi, type OfferAttribute } from '../../api/client';
import { useState, useEffect } from 'react';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

export default function OfferDetail() {
  const { guid } = useParams<{ guid: string }>();
  const queryClient = useQueryClient();
  const [attributes, setAttributes] = useState<OfferAttribute[]>([]);

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
          id: a.id,
          parameter_id: a.parameter_id ?? undefined,
          display_name: a.display_name ?? undefined,
          description: a.description ?? undefined,
          type: a.type ?? undefined,
          values_list: a.values_list ?? undefined,
        })),
      ),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['offer', guid] }),
  });

  const updateAttr = (index: number, field: keyof OfferAttribute, value: string | null) => {
    setAttributes((prev) => prev.map((a, i) => (i === index ? { ...a, [field]: value } : a)));
  };

  if (!guid) return <div>Missing offer GUID</div>;
  if (isLoading) return <div>Loading...</div>;
  if (!data) return <div>Offer not found</div>;

  const { offer } = data;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/admin/offers" style={{ color: '#3498db' }}>← Back to Offers</Link>
      </div>
      <h1>Offer: {offer.offer_name || offer.offer_id}</h1>
      <div style={cardStyle}>
        <p><strong>Offer ID:</strong> {offer.offer_id}</p>
        <p><strong>Offer GUID:</strong> {offer.offer_guid}</p>
        <p><strong>Offer name:</strong> {offer.offer_name ?? '-'}</p>
      </div>
      <div style={{ ...cardStyle, marginTop: '20px' }}>
        <h3>Attributes</h3>
        <p style={{ fontSize: '14px', color: '#7f8c8d', marginBottom: '12px' }}>Edit attributes and click Save.</p>
        {attributes.length > 0 ? (
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead style={{ backgroundColor: '#34495e', color: 'white' }}>
              <tr>
                <th style={{ padding: '8px', textAlign: 'left' }}>Parameter</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Display name</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Type</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Description</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Values list</th>
              </tr>
            </thead>
            <tbody>
              {attributes.map((a, index) => (
                <tr key={a.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
                  <td style={{ padding: '8px' }}>
                    <input
                      value={a.parameter_id ?? ''}
                      onChange={(e) => updateAttr(index, 'parameter_id', e.target.value || null)}
                      style={{ padding: '6px', width: '100%' }}
                    />
                  </td>
                  <td style={{ padding: '8px' }}>
                    <input
                      value={a.display_name ?? ''}
                      onChange={(e) => updateAttr(index, 'display_name', e.target.value || null)}
                      style={{ padding: '6px', width: '100%' }}
                    />
                  </td>
                  <td style={{ padding: '8px' }}>
                    <input
                      value={a.type ?? ''}
                      onChange={(e) => updateAttr(index, 'type', e.target.value || null)}
                      style={{ padding: '6px', width: '100%' }}
                    />
                  </td>
                  <td style={{ padding: '8px' }}>
                    <input
                      value={a.description ?? ''}
                      onChange={(e) => updateAttr(index, 'description', e.target.value || null)}
                      style={{ padding: '6px', width: '100%' }}
                    />
                  </td>
                  <td style={{ padding: '8px' }}>
                    <input
                      value={a.values_list ?? ''}
                      onChange={(e) => updateAttr(index, 'values_list', e.target.value || null)}
                      style={{ padding: '6px', width: '100%' }}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p style={{ color: '#7f8c8d' }}>No attributes. Add them in the database or via API.</p>
        )}
        <button
          type="button"
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending || attributes.length === 0}
          style={{ marginTop: '12px', padding: '10px 20px', backgroundColor: '#27ae60', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
        >
          {saveMutation.isPending ? 'Saving...' : 'Save attributes'}
        </button>
      </div>
    </div>
  );
}
