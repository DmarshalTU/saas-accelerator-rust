import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { customerApiEndpoints, type Plan } from '../../api/client';
import { useState } from 'react';

const cardStyle = { backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)', marginBottom: '20px' };
const btnPrimary = { padding: '8px 16px', backgroundColor: '#3498db', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' } as const;
const btnSuccess = { padding: '10px 20px', backgroundColor: '#27ae60', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' } as const;
const btnDanger  = { padding: '8px 16px', backgroundColor: '#e74c3c', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' } as const;

function Feedback({ ok, err }: { ok: string | null; err: string | null }) {
  if (ok)  return <div style={{ padding: '8px 12px', backgroundColor: '#d4edda', color: '#155724', borderRadius: '4px', marginTop: '8px' }}>{ok}</div>;
  if (err) return <div style={{ padding: '8px 12px', backgroundColor: '#f8d7da', color: '#721c24', borderRadius: '4px', marginTop: '8px' }}>{err}</div>;
  return null;
}

export default function CustomerSubscriptionDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [planId, setPlanId] = useState('');
  const [quantity, setQuantity] = useState<number>(0);
  const [feedback, setFeedback] = useState<Record<string, { ok?: string; err?: string }>>({});

  const setOk  = (k: string, m: string) => setFeedback((f) => ({ ...f, [k]: { ok: m } }));
  const setErr = (k: string, m: string) => setFeedback((f) => ({ ...f, [k]: { err: m } }));
  const clr    = (k: string) => setFeedback((f) => ({ ...f, [k]: {} }));

  const email = typeof window !== 'undefined'
    ? sessionStorage.getItem('customerEmail') || ''
    : '';

  const { data: subscription, isLoading } = useQuery({
    queryKey: ['customer-subscription', id],
    queryFn: () => customerApiEndpoints.getSubscription(id!).then((r) => r.data),
    enabled: !!id,
  });

  const { data: plans } = useQuery({
    queryKey: ['customer-plans'],
    queryFn: () => customerApiEndpoints.getPlans().then((r) => r.data),
  });

  const activateMutation = useMutation({
    mutationFn: (subId: string) => customerApiEndpoints.activateSubscription(subId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customer-subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['user-subscriptions'] });
      setOk('activate', 'Subscription activated successfully.');
    },
    onError: (e: unknown) => setErr('activate', `Activation failed: ${(e as Error).message}`),
  });

  const changePlanMutation = useMutation({
    mutationFn: ({ subId, planId: p }: { subId: string; planId: string }) =>
      customerApiEndpoints.changePlan(subId, p),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customer-subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['user-subscriptions'] });
      setPlanId('');
      setOk('plan', 'Plan change submitted. Your subscription will update once the marketplace confirms.');
    },
    onError: (e: unknown) => setErr('plan', `Plan change failed: ${(e as Error).message}`),
  });

  const changeQuantityMutation = useMutation({
    mutationFn: ({ subId, qty }: { subId: string; qty: number }) =>
      customerApiEndpoints.changeQuantity(subId, qty),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customer-subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['user-subscriptions'] });
      setQuantity(0);
      setOk('quantity', 'Quantity updated successfully.');
    },
    onError: (e: unknown) => setErr('quantity', `Quantity change failed: ${(e as Error).message}`),
  });

  const cancelMutation = useMutation({
    mutationFn: (subId: string) => customerApiEndpoints.cancelSubscription(subId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-subscriptions'] });
      navigate('/subscriptions');
    },
    onError: (e: unknown) => setErr('cancel', `Cancellation failed: ${(e as Error).message}`),
  });

  if (!id)               return <div>Missing subscription ID</div>;
  if (isLoading || !subscription) return <div>Loading…</div>;

  const sub = subscription as {
    id?: number;
    amp_subscription_id?: string;
    subscription_status?: string;
    amp_plan_id?: string;
    amp_quantity?: number;
    purchaser_email?: string | null;
  };
  const status    = sub.subscription_status ?? '';
  const planIdCur = sub.amp_plan_id ?? '';
  const qtyCur    = sub.amp_quantity ?? 0;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/subscriptions" style={{ color: '#3498db' }}>← Back to My Subscriptions</Link>
      </div>

      <h1>Subscription: {sub.amp_subscription_id ?? id}</h1>

      {/* Details */}
      <div style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Details</h3>
        <p><strong>Status:</strong> {status}</p>
        <p><strong>Current plan:</strong> {planIdCur}</p>
        <p><strong>Quantity:</strong> {qtyCur}</p>
        {email && <p><strong>Account:</strong> {email}</p>}
      </div>

      {/* Activate */}
      {status === 'PendingFulfillmentStart' && (
        <div style={cardStyle}>
          <h3 style={{ marginTop: 0 }}>Activate subscription</h3>
          <p style={{ color: '#7f8c8d', marginBottom: '12px' }}>Your subscription is ready to activate. Click below to complete the process.</p>
          <button
            onClick={() => { clr('activate'); activateMutation.mutate(id); }}
            disabled={activateMutation.isPending}
            style={btnSuccess}
          >
            {activateMutation.isPending ? 'Activating…' : 'Activate subscription'}
          </button>
          <Feedback ok={feedback.activate?.ok ?? null} err={feedback.activate?.err ?? null} />
        </div>
      )}

      {/* Change Plan */}
      <div style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Change Plan</h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', flexWrap: 'wrap' }}>
          <select
            value={planId}
            onChange={(e) => setPlanId(e.target.value)}
            style={{ padding: '8px', minWidth: '200px' }}
          >
            <option value="">Select a plan…</option>
            {plans?.map((p: Plan) => (
              <option key={p.id} value={p.plan_id} disabled={p.plan_id === planIdCur}>
                {p.plan_name || p.display_name || p.plan_id}
                {p.plan_id === planIdCur ? ' (current)' : ''}
              </option>
            ))}
          </select>
          <button
            disabled={!planId || changePlanMutation.isPending}
            onClick={() => { clr('plan'); changePlanMutation.mutate({ subId: id, planId }); }}
            style={btnPrimary}
          >
            {changePlanMutation.isPending ? 'Updating…' : 'Change Plan'}
          </button>
        </div>
        <Feedback ok={feedback.plan?.ok ?? null} err={feedback.plan?.err ?? null} />
      </div>

      {/* Change Quantity */}
      <div style={cardStyle}>
        <h3 style={{ marginTop: 0 }}>Change Quantity</h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <input
            type="number" min={1}
            value={quantity || ''}
            placeholder={String(qtyCur)}
            onChange={(e) => setQuantity(parseInt(e.target.value, 10) || 0)}
            style={{ padding: '8px', width: '80px' }}
          />
          <button
            disabled={quantity < 1 || changeQuantityMutation.isPending}
            onClick={() => { clr('quantity'); changeQuantityMutation.mutate({ subId: id, qty: quantity }); }}
            style={btnPrimary}
          >
            {changeQuantityMutation.isPending ? 'Updating…' : 'Change Quantity'}
          </button>
        </div>
        <Feedback ok={feedback.quantity?.ok ?? null} err={feedback.quantity?.err ?? null} />
      </div>

      {/* Cancel */}
      {status !== 'Unsubscribed' && (
        <div style={cardStyle}>
          <h3 style={{ marginTop: 0 }}>Cancel subscription</h3>
          <p style={{ color: '#7f8c8d', marginBottom: '12px' }}>
            This will cancel your subscription. Access will remain until the end of the current billing period.
          </p>
          <button
            onClick={() => {
              if (window.confirm('Cancel this subscription? This action cannot be undone.')) {
                clr('cancel'); cancelMutation.mutate(id);
              }
            }}
            disabled={cancelMutation.isPending}
            style={btnDanger}
          >
            {cancelMutation.isPending ? 'Cancelling…' : 'Cancel subscription'}
          </button>
          <Feedback ok={feedback.cancel?.ok ?? null} err={feedback.cancel?.err ?? null} />
        </div>
      )}
    </div>
  );
}
