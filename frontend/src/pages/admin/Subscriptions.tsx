import { Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { subscriptionsApi, type Subscription } from '../../api/client';
import { useState } from 'react';

export default function AdminSubscriptions() {
  const queryClient = useQueryClient();
  const [selectedSubscription, setSelectedSubscription] = useState<string | null>(null);

  const { data: subscriptions, isLoading } = useQuery({
    queryKey: ['subscriptions'],
    queryFn: () => subscriptionsApi.getAll().then(res => res.data),
  });

  const { data: auditLogs } = useQuery({
    queryKey: ['audit-logs', selectedSubscription],
    queryFn: () => subscriptionsApi.getAuditLogs(selectedSubscription!).then(res => res.data),
    enabled: !!selectedSubscription,
  });

  const activateMutation = useMutation({
    mutationFn: (id: string) => subscriptionsApi.activate(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
    },
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Subscribed':
        return '#27ae60';
      case 'PendingFulfillmentStart':
      case 'PendingActivation':
        return '#e67e22';
      case 'Suspended':
        return '#e74c3c';
      case 'Unsubscribed':
        return '#95a5a6';
      default:
        return '#34495e';
    }
  };

  if (isLoading) {
    return <div>Loading subscriptions...</div>;
  }

  return (
    <div>
      <h1>Subscriptions</h1>
      <p style={{ color: '#7f8c8d', marginBottom: '16px' }}>
        Click a subscription ID or <strong>Manage</strong> to open it and change plan, quantity, record usage, or unsubscribe.
      </p>
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
            <th style={{ padding: '12px', textAlign: 'left' }}>Subscription</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Status</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Plan</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Quantity</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Email</th>
            <th style={{ padding: '12px', textAlign: 'left' }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {subscriptions?.map((sub) => (
            <tr key={sub.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
              <td style={{ padding: '12px' }}>
                <Link to={`/admin/subscriptions/${sub.amp_subscription_id}`} style={{ color: '#3498db' }}>
                  {sub.amp_subscription_id}
                </Link>
              </td>
              <td style={{ padding: '12px' }}>
                <span style={{
                  padding: '4px 8px',
                  borderRadius: '4px',
                  backgroundColor: getStatusColor(sub.subscription_status),
                  color: 'white',
                  fontSize: '12px',
                }}>
                  {sub.subscription_status}
                </span>
              </td>
              <td style={{ padding: '12px' }}>{sub.amp_plan_id}</td>
              <td style={{ padding: '12px' }}>{sub.amp_quantity}</td>
              <td style={{ padding: '12px' }}>{sub.purchaser_email || '-'}</td>
              <td style={{ padding: '12px' }}>
                <Link
                  to={`/admin/subscriptions/${sub.amp_subscription_id}`}
                  style={{
                    display: 'inline-block',
                    padding: '6px 12px',
                    backgroundColor: '#27ae60',
                    color: 'white',
                    textDecoration: 'none',
                    borderRadius: '4px',
                    marginRight: '8px',
                    fontSize: '13px',
                  }}
                >
                  Manage
                </Link>
                {sub.subscription_status === 'PendingFulfillmentStart' && (
                  <button
                    onClick={() => activateMutation.mutate(sub.amp_subscription_id)}
                    style={{
                      padding: '6px 12px',
                      backgroundColor: '#3498db',
                      color: 'white',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: 'pointer',
                      marginRight: '8px',
                    }}
                  >
                    Activate
                  </button>
                )}
                <button
                  onClick={() => setSelectedSubscription(selectedSubscription === sub.amp_subscription_id ? null : sub.amp_subscription_id)}
                  style={{
                    padding: '6px 12px',
                    backgroundColor: '#95a5a6',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                  }}
                >
                  {selectedSubscription === sub.amp_subscription_id ? 'Hide' : 'View'} Logs
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {selectedSubscription && auditLogs && (
        <div style={{
          marginTop: '20px',
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <h3>Audit Logs for {selectedSubscription}</h3>
          <table style={{ width: '100%', marginTop: '10px' }}>
            <thead>
              <tr>
                <th style={{ padding: '8px', textAlign: 'left' }}>Attribute</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Old Value</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>New Value</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Date</th>
              </tr>
            </thead>
            <tbody>
              {auditLogs.map((log) => (
                <tr key={log.id}>
                  <td style={{ padding: '8px' }}>{log.attribute || '-'}</td>
                  <td style={{ padding: '8px' }}>{log.old_value || '-'}</td>
                  <td style={{ padding: '8px' }}>{log.new_value || '-'}</td>
                  <td style={{ padding: '8px' }}>
                    {log.create_date ? new Date(log.create_date).toLocaleString() : '-'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

