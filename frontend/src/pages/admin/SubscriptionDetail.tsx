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

export default function AdminSubscriptionDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [planId, setPlanId] = useState('');
  const [quantity, setQuantity] = useState<number>(0);
  const [usageDimension, setUsageDimension] = useState('');
  const [usageQuantity, setUsageQuantity] = useState<string>('');

  const { data: subscription, isLoading: subLoading } = useQuery({
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

  const changePlanMutation = useMutation({
    mutationFn: ({ subId, planId: p }: { subId: string; planId: string }) =>
      subscriptionsApi.changePlan(subId, p),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      setPlanId('');
    },
  });

  const changeQuantityMutation = useMutation({
    mutationFn: ({ subId, qty }: { subId: string; qty: number }) =>
      subscriptionsApi.changeQuantity(subId, qty),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscription', id] });
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      setQuantity(0);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (subId: string) => subscriptionsApi.delete(subId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      navigate('/admin/subscriptions');
    },
  });

  const usageMutation = useMutation({
    mutationFn: ({
      subId,
      dimension,
      quantity: qty,
    }: {
      subId: string;
      dimension: string;
      quantity: number;
    }) => subscriptionsApi.emitUsage(subId, dimension, qty),
    onSuccess: () => {
      setUsageDimension('');
      setUsageQuantity('');
    },
  });

  if (!id) return <div>Missing subscription ID</div>;
  if (subLoading || !subscription) return <div>Loading...</div>;

  const sub = subscription as Subscription;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/admin/subscriptions" style={{ color: '#3498db' }}>
          ← Back to Subscriptions
        </Link>
      </div>
      <h1>Subscription: {sub.amp_subscription_id}</h1>

      <div style={{ ...cardStyle, borderLeft: '4px solid #3498db', marginBottom: '20px' }}>
        <h3 style={{ marginTop: 0 }}>Actions you can do here</h3>
        <ul style={{ margin: 0, paddingLeft: '20px', lineHeight: 1.8 }}>
          <li><a href="#change-plan" style={{ color: '#2980b9' }}>Change plan</a> – select a new plan and submit</li>
          <li><a href="#change-quantity" style={{ color: '#2980b9' }}>Change quantity</a> – set new quantity and submit</li>
          <li><a href="#record-usage" style={{ color: '#2980b9' }}>Record metered usage</a> – send a usage event (dimension + quantity)</li>
          <li><a href="#unsubscribe" style={{ color: '#2980b9' }}>Unsubscribe</a> – delete this subscription</li>
          <li><a href="#audit-logs" style={{ color: '#2980b9' }}>View audit logs</a> – see history below</li>
          {sub.subscription_status === 'PendingFulfillmentStart' && (
            <li>Activate – use the green “Activate” block below or from the list</li>
          )}
        </ul>
      </div>

      <div style={cardStyle}>
        <h3>Details</h3>
        <p>
          <strong>Status:</strong> {sub.subscription_status}
        </p>
        <p>
          <strong>Plan:</strong> {sub.amp_plan_id}
        </p>
        <p>
          <strong>Quantity:</strong> {sub.amp_quantity}
        </p>
        <p>
          <strong>Purchaser:</strong> {sub.purchaser_email || '-'}
        </p>
      </div>

      {sub.subscription_status === 'PendingFulfillmentStart' && (
        <div style={cardStyle}>
          <h3>Activate</h3>
          <Link
            to="/admin/subscriptions"
            style={{
              display: 'inline-block',
              padding: '8px 16px',
              backgroundColor: '#27ae60',
              color: 'white',
              borderRadius: '4px',
              textDecoration: 'none',
            }}
          >
            Activate from list
          </Link>
        </div>
      )}

      <div id="change-plan" style={cardStyle}>
        <h3>Change Plan</h3>
        <select
          value={planId}
          onChange={(e) => setPlanId(e.target.value)}
          style={{ padding: '8px', marginRight: '8px', minWidth: '200px' }}
        >
          <option value="">Select plan</option>
          {plans?.map((p) => (
            <option key={p.id} value={p.plan_id}>
              {(p as { plan_name?: string; display_name?: string }).plan_name ||
                (p as { plan_name?: string; display_name?: string }).display_name ||
                p.plan_id}
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

      <div id="change-quantity" style={cardStyle}>
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
          onClick={() => changeQuantityMutation.mutate({ subId: id, qty: quantity })}
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

      <div id="record-usage" style={cardStyle}>
        <h3>Record usage (metered)</h3>
        <input
          type="text"
          placeholder="Dimension"
          value={usageDimension}
          onChange={(e) => setUsageDimension(e.target.value)}
          style={{ padding: '8px', marginRight: '8px', width: '120px' }}
        />
        <input
          type="number"
          step="any"
          placeholder="Quantity"
          value={usageQuantity}
          onChange={(e) => setUsageQuantity(e.target.value)}
          style={{ padding: '8px', marginRight: '8px', width: '80px' }}
        />
        <button
          disabled={
            !usageDimension ||
            !usageQuantity ||
            usageMutation.isPending
          }
          onClick={() =>
            usageMutation.mutate({
              subId: id,
              dimension: usageDimension,
              quantity: parseFloat(usageQuantity),
            })
          }
          style={{
            padding: '8px 16px',
            backgroundColor: '#9b59b6',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {usageMutation.isPending ? 'Sending...' : 'Emit usage'}
        </button>
      </div>

      <div id="unsubscribe" style={cardStyle}>
        <h3>Unsubscribe (delete)</h3>
        <button
          onClick={() => {
            if (window.confirm('Unsubscribe this subscription?'))
              deleteMutation.mutate(id);
          }}
          disabled={deleteMutation.isPending}
          style={{
            padding: '8px 16px',
            backgroundColor: '#e74c3c',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {deleteMutation.isPending ? 'Deleting...' : 'Unsubscribe'}
        </button>
      </div>

      <div id="audit-logs" style={cardStyle}>
        <h3>Audit logs</h3>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #ecf0f1' }}>
              <th style={{ padding: '8px', textAlign: 'left' }}>Attribute</th>
              <th style={{ padding: '8px', textAlign: 'left' }}>Old</th>
              <th style={{ padding: '8px', textAlign: 'left' }}>New</th>
              <th style={{ padding: '8px', textAlign: 'left' }}>Date</th>
            </tr>
          </thead>
          <tbody>
            {(auditLogs as SubscriptionAuditLog[] | undefined)?.map((log) => (
              <tr key={log.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
                <td style={{ padding: '8px' }}>{log.attribute ?? '-'}</td>
                <td style={{ padding: '8px' }}>{log.old_value ?? '-'}</td>
                <td style={{ padding: '8px' }}>{log.new_value ?? '-'}</td>
                <td style={{ padding: '8px' }}>
                  {log.create_date
                    ? new Date(log.create_date).toLocaleString()
                    : '-'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {(!auditLogs || (auditLogs as SubscriptionAuditLog[]).length === 0) && (
          <p style={{ color: '#7f8c8d' }}>No audit logs.</p>
        )}
      </div>
    </div>
  );
}
