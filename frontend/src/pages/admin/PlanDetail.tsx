import { Link, useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { plansApi, eventsApi, offersApi, type PlanDetailResponse, type PlanEventsMapping } from '../../api/client';
import { useState, useEffect } from 'react';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

type PlanEventEdit = { id?: number; event_id: number; success_state_emails: string; failure_state_emails: string; copy_to_customer: boolean };

export default function PlanDetail() {
  const { guid } = useParams<{ guid: string }>();
  const queryClient = useQueryClient();
  const [planEvents, setPlanEvents] = useState<PlanEventEdit[]>([]);
  const [offerAttributeIds, setOfferAttributeIds] = useState<number[]>([]);

  const { data, isLoading } = useQuery({
    queryKey: ['plan', guid],
    queryFn: () => plansApi.getByGuid(guid!).then((r) => r.data),
    enabled: !!guid,
  });

  const { data: events } = useQuery({
    queryKey: ['events'],
    queryFn: () => eventsApi.getAll().then((r) => r.data),
  });

  const offerGuid = (data as PlanDetailResponse)?.offer_id as string | undefined;
  const { data: offerData } = useQuery({
    queryKey: ['offer', offerGuid],
    queryFn: () => offersApi.getByGuid(offerGuid!).then((r) => r.data),
    enabled: !!offerGuid,
  });

  useEffect(() => {
    if (data?.plan_events) {
      setPlanEvents(
        data.plan_events.map((e: PlanEventsMapping) => ({
          id: e.id,
          event_id: e.event_id,
          success_state_emails: e.success_state_emails ?? '',
          failure_state_emails: e.failure_state_emails ?? '',
          copy_to_customer: e.copy_to_customer ?? false,
        })),
      );
    }
    if (data?.offer_attribute_ids) setOfferAttributeIds(data.offer_attribute_ids);
  }, [data?.plan_events, data?.offer_attribute_ids]);

  const saveMutation = useMutation({
    mutationFn: () =>
      plansApi.saveByGuid(guid!, {
        plan_events: planEvents.map((e) => ({
          id: e.id,
          event_id: e.event_id,
          success_state_emails: e.success_state_emails || undefined,
          failure_state_emails: e.failure_state_emails || undefined,
          copy_to_customer: e.copy_to_customer,
        })),
        offer_attribute_ids: offerAttributeIds,
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plan', guid] }),
  });

  const addEventRow = () => {
    setPlanEvents((prev) => [...prev, { event_id: events?.[0]?.id ?? 0, success_state_emails: '', failure_state_emails: '', copy_to_customer: false }]);
  };

  const removeEventRow = (index: number) => {
    setPlanEvents((prev) => prev.filter((_, i) => i !== index));
  };

  const toggleOfferAttribute = (id: number) => {
    setOfferAttributeIds((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]));
  };

  if (!guid) return <div>Missing plan GUID</div>;
  if (isLoading) return <div>Loading...</div>;
  if (!data) return <div>Plan not found</div>;

  const planDetail = data as PlanDetailResponse;
  const offerIdGuid = planDetail.offer_id as string | undefined;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/admin/plans" style={{ color: '#3498db' }}>← Back to Plans</Link>
      </div>
      <h1>Plan: {planDetail.display_name || planDetail.plan_id}</h1>
      <div style={cardStyle}>
        <p><strong>Plan ID:</strong> {planDetail.plan_id}</p>
        <p><strong>Plan GUID:</strong> {planDetail.plan_guid}</p>
        <p><strong>Offer ID:</strong> {offerIdGuid ?? '-'}</p>
        <p><strong>Display name:</strong> {planDetail.display_name ?? '-'}</p>
      </div>

      <div style={{ ...cardStyle, marginTop: '20px' }}>
        <h3>Plan events</h3>
        <p style={{ fontSize: '14px', color: '#7f8c8d' }}>Map events to this plan. Add rows and save.</p>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead style={{ backgroundColor: '#34495e', color: 'white' }}>
            <tr>
              <th style={{ padding: '8px', textAlign: 'left' }}>Event</th>
              <th style={{ padding: '8px', textAlign: 'left' }}>Success emails</th>
              <th style={{ padding: '8px', textAlign: 'left' }}>Failure emails</th>
              <th style={{ padding: '8px', textAlign: 'left' }}>Copy to customer</th>
              <th style={{ padding: '8px', width: '80px' }}></th>
            </tr>
          </thead>
          <tbody>
            {planEvents.map((row, index) => (
              <tr key={index} style={{ borderBottom: '1px solid #ecf0f1' }}>
                <td style={{ padding: '8px' }}>
                  <select
                    value={row.event_id}
                    onChange={(e) => setPlanEvents((prev) => prev.map((p, i) => (i === index ? { ...p, event_id: parseInt(e.target.value, 10) } : p)))}
                    style={{ padding: '6px', minWidth: '160px' }}
                  >
                    {events?.map((ev) => (
                      <option key={ev.id} value={ev.id}>{ev.events_name ?? `Event ${ev.id}`}</option>
                    ))}
                  </select>
                </td>
                <td style={{ padding: '8px' }}>
                  <input
                    type="text"
                    value={row.success_state_emails}
                    onChange={(e) => setPlanEvents((prev) => prev.map((p, i) => (i === index ? { ...p, success_state_emails: e.target.value } : p)))}
                    style={{ padding: '6px', width: '100%' }}
                  />
                </td>
                <td style={{ padding: '8px' }}>
                  <input
                    type="text"
                    value={row.failure_state_emails}
                    onChange={(e) => setPlanEvents((prev) => prev.map((p, i) => (i === index ? { ...p, failure_state_emails: e.target.value } : p)))}
                    style={{ padding: '6px', width: '100%' }}
                  />
                </td>
                <td style={{ padding: '8px' }}>
                  <input
                    type="checkbox"
                    checked={row.copy_to_customer}
                    onChange={(e) => setPlanEvents((prev) => prev.map((p, i) => (i === index ? { ...p, copy_to_customer: e.target.checked } : p)))}
                  />
                </td>
                <td style={{ padding: '8px' }}>
                  <button type="button" onClick={() => removeEventRow(index)} style={{ padding: '4px 8px', backgroundColor: '#e74c3c', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}>Remove</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <button type="button" onClick={addEventRow} style={{ marginTop: '8px', padding: '6px 12px', backgroundColor: '#3498db', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}>Add event</button>
      </div>

      {offerData?.attributes && offerData.attributes.length > 0 && (
        <div style={{ ...cardStyle, marginTop: '20px' }}>
          <h3>Offer attributes linked to this plan</h3>
          <p style={{ fontSize: '14px', color: '#7f8c8d' }}>Select which offer attributes apply to this plan.</p>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '12px', marginTop: '8px' }}>
            {offerData.attributes.map((attr) => (
              <label key={attr.id} style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={offerAttributeIds.includes(attr.id)}
                  onChange={() => toggleOfferAttribute(attr.id)}
                />
                <span>{attr.display_name || attr.parameter_id || `Attribute ${attr.id}`}</span>
              </label>
            ))}
          </div>
        </div>
      )}

      <div style={{ marginTop: '20px' }}>
        <button
          type="button"
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending}
          style={{ padding: '10px 20px', backgroundColor: '#27ae60', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
        >
          {saveMutation.isPending ? 'Saving...' : 'Save plan'}
        </button>
      </div>
    </div>
  );
}
