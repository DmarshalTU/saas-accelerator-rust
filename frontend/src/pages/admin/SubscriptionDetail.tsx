import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  subscriptionsApi,
  plansApi,
  type Subscription,
  type SubscriptionAuditLog,
} from '../../api/client';
import { useState } from 'react';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  marginBottom: '20px',
};

const btnPrimary = { padding: '8px 16px', backgroundColor: '#3498db', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' } as const;
const btnSuccess = { padding: '8px 16px', backgroundColor: '#27ae60', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' } as const;
const btnDanger  = { padding: '8px 16px', backgroundColor: '#e74c3c', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' } as const;
const btnPurple  = { padding: '8px 16px', backgroundColor: '#9b59b6', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' } as const;

function Feedback({ ok, err }: { ok: string | null; err: string | null }) {
  if (ok) return <div style={{ padding: '8px 12px', backgroundColor: '#d4edda', color: '#155724', borderRadius: '4px', marginTop: '8px' }}>{ok}</div>;
  if (err) return <div style={{ padding: '8px 12px', backgroundColor: '#f8d7da', color: '#721c24', borderRadius: '4px', marginTop: '8px' }}>{err}</div>;
  return null;
}

export default function AdminSubscriptionDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [planId, setPlanId] = useState('');
  const [quantity, setQuantity] = useState<number>(0);
  const [usageDimension, setUsageDimension] = useState('');
  const [usageQuantity, setUsageQuantity] = useState<string>('');
  const [feedback, setFeedback] = useState<Record<string, { ok?: string; err?: string }>>({});

  const setOk  = (key: string, msg: string) => setFeedback((f) => ({ ...f, [key]: { ok: msg } }));
  const setErr = (key: string, msg: string) => setFeedback((f) => ({ ...f, [key]: { err: msg } }));
  const clearFeedback = (key: string) => setFeedback((f) => ({ ...f, [key]: {} }));

  const { data: subscription, isLoading } = useQuery({
    queryKey: ['subscription', id],
    queryFn: () => subscriptionsApi.getById(id!).then((r) => r.data),
    enabled: !!id,
  });

  const { data: plans } = useQuery({
    queryKey: ['plans'],
    queryFn: () => plansApi.getAll().then((r) => r.data),
  });

  const { data: auditLogs } = useQuery({
    queryKey: ['audit-logs', id],
    queryFn: () => subscriptionsApi.getAuditLogs(id!).then((r) => r.data),
    enabled: !!id,
  });

  const activateMutation = useMutation({
    mutationFn: (subId: string) => subscriptionsApi.activate(subId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      setOk('activate', 'Subscription activated successfully.');
    },
    onError: (e: unknown) => setErr('activate', `Activation failed: ${(e as Error).message}`),
  });

  const changePlanMutation = useMutation({
    mutationFn: ({ subId, planId: p }: { subId: string; planId: string }) =>
      subscriptionsApi.changePlan(subId, p),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      setPlanId('');
      setOk('plan', 'Plan change submitted. The subscription will update once the marketplace confirms.');
    },
    onError: (e: unknown) => setErr('plan', `Plan change failed: ${(e as Error).message}`),
  });

  const changeQuantityMutation = useMutation({
    mutationFn: ({ subId, qty }: { subId: string; qty: number }) =>
      subscriptionsApi.changeQuantity(subId, qty),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      setQuantity(0);
      setOk('quantity', 'Quantity change submitted.');
    },
    onError: (e: unknown) => setErr('quantity', `Quantity change failed: ${(e as Error).message}`),
  });

  const deleteMutation = useMutation({
    mutationFn: (subId: string) => subscriptionsApi.delete(subId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      navigate('/admin/subscriptions');
    },
    onError: (e: unknown) => setErr('delete', `Unsubscribe failed: ${(e as Error).message}`),
  });

  const usageMutation = useMutation({
    mutationFn: ({ subId, dimension, quantity: qty }: { subId: string; dimension: string; quantity: number }) =>
      subscriptionsApi.emitUsage(subId, dimension, qty),
    onSuccess: () => {
      setUsageDimension('');
      setUsageQuantity('');
      setOk('usage', 'Usage event recorded.');
    },
    onError: (e: unknown) => setErr('usage', `Failed to emit usage: ${(e as Error).message}`),
  });

  if (!id) return <div>Missing subscription ID</div>;
  if (isLoading || !subscription) return <div>Loading…</div>;

  const sub = subscription as Subscription;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/admin/subscriptions" style={{ color: '#3498db' }}>← Back to Subscriptions</Link>
      </div>
      <h1>Subscription: {sub.amp_subscription_id}</h1>

      {/* Details */}
      <div style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Details</h3>
        <p><strong>Status:</strong> {sub.subscription_status}</p>
        <p><strong>Plan:</strong> {sub.amp_plan_id}</p>
        <p><strong>Quantity:</strong> {sub.amp_quantity}</p>
        <p><strong>Purchaser:</strong> {sub.purchaser_email ?? '-'}</p>
      </div>

      {/* Activate */}
      {sub.subscription_status === 'PendingFulfillmentStart' && (
        <div style={cardStyle}>
          <h3 style={{ marginTop: 0 }}>Activate</h3>
          <p style={{ color: '#7f8c8d', marginBottom: '12px' }}>This subscription is pending fulfillment. Click to activate it in the marketplace.</p>
          <button
            onClick={() => { clearFeedback('activate'); activateMutation.mutate(id); }}
            disabled={activateMutation.isPending}
            style={btnSuccess}
          >
            {activateMutation.isPending ? 'Activating…' : 'Activate subscription'}
          </button>
          <Feedback ok={feedback.activate?.ok ?? null} err={feedback.activate?.err ?? null} />
        </div>
      )}

      {/* Change Plan */}
      <div id="change-plan" style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Change Plan</h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', flexWrap: 'wrap' }}>
          <select
            value={planId}
            onChange={(e) => setPlanId(e.target.value)}
            style={{ padding: '8px', minWidth: '200px' }}
          >
            <option value="">Select plan…</option>
            {plans?.map((p) => (
              <option key={p.id} value={p.plan_id}>
                {(p as { display_name?: string }).display_name || p.plan_id}
              </option>
            ))}
          </select>
          <button
            disabled={!planId || changePlanMutation.isPending}
            onClick={() => { clearFeedback('plan'); changePlanMutation.mutate({ subId: id, planId }); }}
            style={btnPrimary}
          >
            {changePlanMutation.isPending ? 'Updating…' : 'Change Plan'}
          </button>
        </div>
        <Feedback ok={feedback.plan?.ok ?? null} err={feedback.plan?.err ?? null} />
      </div>

      {/* Change Quantity */}
      <div id="change-quantity" style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Change Quantity</h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <input
            type="number" min={1}
            value={quantity || ''}
            onChange={(e) => setQuantity(parseInt(e.target.value, 10) || 0)}
            style={{ padding: '8px', width: '80px' }}
          />
          <button
            disabled={quantity < 1 || changeQuantityMutation.isPending}
            onClick={() => { clearFeedback('quantity'); changeQuantityMutation.mutate({ subId: id, qty: quantity }); }}
            style={btnPrimary}
          >
            {changeQuantityMutation.isPending ? 'Updating…' : 'Change Quantity'}
          </button>
        </div>
        <Feedback ok={feedback.quantity?.ok ?? null} err={feedback.quantity?.err ?? null} />
      </div>

      {/* Metered usage */}
      <div id="record-usage" style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Record metered usage</h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', flexWrap: 'wrap' }}>
          <input
            type="text" placeholder="Dimension"
            value={usageDimension}
            onChange={(e) => setUsageDimension(e.target.value)}
            style={{ padding: '8px', width: '140px' }}
          />
          <input
            type="number" step="any" placeholder="Quantity"
            value={usageQuantity}
            onChange={(e) => setUsageQuantity(e.target.value)}
            style={{ padding: '8px', width: '80px' }}
          />
          <button
            disabled={!usageDimension || !usageQuantity || usageMutation.isPending}
            onClick={() => { clearFeedback('usage'); usageMutation.mutate({ subId: id, dimension: usageDimension, quantity: parseFloat(usageQuantity) }); }}
            style={btnPurple}
          >
            {usageMutation.isPending ? 'Sending…' : 'Emit usage'}
          </button>
        </div>
        <Feedback ok={feedback.usage?.ok ?? null} err={feedback.usage?.err ?? null} />
      </div>

      {/* Unsubscribe */}
      <div id="unsubscribe" style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Unsubscribe</h3>
        <p style={{ color: '#7f8c8d', marginBottom: '12px' }}>This will cancel the subscription in the marketplace and mark it as Unsubscribed.</p>
        <button
          onClick={() => { if (window.confirm('Unsubscribe this subscription? This cannot be undone.')) { clearFeedback('delete'); deleteMutation.mutate(id); } }}
          disabled={deleteMutation.isPending}
          style={btnDanger}
        >
          {deleteMutation.isPending ? 'Unsubscribing…' : 'Unsubscribe'}
        </button>
        <Feedback ok={feedback.delete?.ok ?? null} err={feedback.delete?.err ?? null} />
      </div>

      {/* Audit Logs */}
      <div id="audit-logs" style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Audit logs</h3>
        {(!auditLogs || (auditLogs as SubscriptionAuditLog[]).length === 0)
          ? <p style={{ color: '#7f8c8d' }}>No audit logs yet.</p>
          : (
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ backgroundColor: '#f8f9fa', borderBottom: '2px solid #dee2e6' }}>
                    <th style={{ padding: '8px', textAlign: 'left' }}>Attribute</th>
                    <th style={{ padding: '8px', textAlign: 'left' }}>Old value</th>
                    <th style={{ padding: '8px', textAlign: 'left' }}>New value</th>
                    <th style={{ padding: '8px', textAlign: 'left' }}>Date</th>
                  </tr>
                </thead>
                <tbody>
                  {(auditLogs as SubscriptionAuditLog[]).map((log) => (
                    <tr key={log.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
                      <td style={{ padding: '8px' }}>{log.attribute ?? '-'}</td>
                      <td style={{ padding: '8px' }}>{log.old_value ?? '-'}</td>
                      <td style={{ padding: '8px' }}>{log.new_value ?? '-'}</td>
                      <td style={{ padding: '8px' }}>{log.create_date ? new Date(log.create_date).toLocaleString() : '-'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
      </div>
    </div>
  );
}
