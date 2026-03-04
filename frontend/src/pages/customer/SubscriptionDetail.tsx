import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { customerApiEndpoints, type Plan } from '../../api/client';
import { useState } from 'react';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  marginBottom: '20px',
};

export default function CustomerSubscriptionDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [planId, setPlanId] = useState('');
  const [quantity, setQuantity] = useState<number>(0);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const email =
    typeof window !== 'undefined'
      ? sessionStorage.getItem('customerEmail') || 'user@example.com'
      : 'user@example.com';

  const { data: subscription, isLoading: subLoading } = useQuery({
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
      setSuccessMessage('Subscription activated successfully.');
    },
  });

  const changePlanMutation = useMutation({
    mutationFn: ({ subId, planId: p }: { subId: string; planId: string }) =>
      customerApiEndpoints.changePlan(subId, p),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customer-subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['user-subscriptions'] });
      setPlanId('');
      setSuccessMessage('Plan updated successfully.');
    },
  });

  const changeQuantityMutation = useMutation({
    mutationFn: ({ subId, qty }: { subId: string; qty: number }) =>
      customerApiEndpoints.changeQuantity(subId, qty),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customer-subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['user-subscriptions'] });
      setQuantity(0);
      setSuccessMessage('Quantity updated successfully.');
    },
  });

  if (!id) return <div>Missing subscription ID</div>;
  if (subLoading || !subscription) return <div>Loading...</div>;

  const sub = subscription as {
    id?: number;
    amp_subscription_id?: string;
    subscription_status?: string;
    amp_plan_id?: string;
    amp_quantity?: number;
    purchaser_email?: string | null;
  };
  const status = sub.subscription_status ?? (subscription as any).subscription_status;
  const planIdCur = sub.amp_plan_id ?? (subscription as any).amp_plan_id;
  const qtyCur = sub.amp_quantity ?? (subscription as any).amp_quantity ?? 0;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/subscriptions" style={{ color: '#3498db' }}>
          ← Back to My Subscriptions
        </Link>
      </div>
      {successMessage && (
        <div style={{
          marginBottom: '16px',
          padding: '12px 16px',
          backgroundColor: '#d4edda',
          border: '1px solid #c3e6cb',
          borderRadius: '8px',
          color: '#155724',
        }}>
          {successMessage}
        </div>
      )}
      <h1>Subscription: {sub.amp_subscription_id ?? id}</h1>

      <div style={{ ...cardStyle, borderLeft: '4px solid #3498db', marginBottom: '20px' }}>
        <h3 style={{ marginTop: 0 }}>Actions you can do here</h3>
        <ul style={{ margin: 0, paddingLeft: '20px', lineHeight: 1.8 }}>
          {status === 'PendingFulfillmentStart' && (
            <li><strong>Activate</strong> – use the green button below</li>
          )}
          <li><strong>Change plan</strong> – select a new plan and click Change Plan</li>
          <li><strong>Change quantity</strong> – enter a number and click Change Quantity</li>
        </ul>
      </div>

      <div style={cardStyle}>
        <h3>Details</h3>
        <p>
          <strong>Status:</strong> {status}
        </p>
        <p>
          <strong>Plan:</strong> {planIdCur}
        </p>
        <p>
          <strong>Quantity:</strong> {qtyCur}
        </p>
      </div>

      {status === 'PendingFulfillmentStart' && (
        <div style={cardStyle}>
          <h3>Activate</h3>
          <button
            onClick={() => activateMutation.mutate(id)}
            disabled={activateMutation.isPending}
            style={{
              padding: '10px 20px',
              backgroundColor: '#27ae60',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            {activateMutation.isPending ? 'Activating...' : 'Activate subscription'}
          </button>
        </div>
      )}

      <div style={cardStyle}>
        <h3>Change Plan</h3>
        <select
          value={planId}
          onChange={(e) => setPlanId(e.target.value)}
          style={{ padding: '8px', marginRight: '8px', minWidth: '200px' }}
        >
          <option value="">Select plan</option>
          {plans?.map((p: Plan) => (
            <option key={p.id} value={p.plan_id}>
              {p.plan_name || p.display_name || p.plan_id}
            </option>
          ))}
        </select>
        <button
          disabled={!planId || changePlanMutation.isPending}
          onClick={() => changePlanMutation.mutate({ subId: id, planId })}
          style={{
            padding: '8px 16px',
            backgroundColor: '#3498db',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {changePlanMutation.isPending ? 'Updating...' : 'Change Plan'}
        </button>
      </div>

      <div style={cardStyle}>
        <h3>Change Quantity</h3>
        <input
          type="number"
          min={1}
          value={quantity || ''}
          onChange={(e) => setQuantity(parseInt(e.target.value, 10) || 0)}
          style={{ padding: '8px', marginRight: '8px', width: '80px' }}
        />
        <button
          disabled={quantity < 1 || changeQuantityMutation.isPending}
          onClick={() =>
            changeQuantityMutation.mutate({ subId: id, qty: quantity })}
          style={{
            padding: '8px 16px',
            backgroundColor: '#3498db',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {changeQuantityMutation.isPending ? 'Updating...' : 'Change Quantity'}
        </button>
      </div>
    </div>
  );
}
