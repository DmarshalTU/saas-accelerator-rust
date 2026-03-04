import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { customerApiEndpoints } from '../../api/client';
import { useState } from 'react';

const defaultEmail =
  typeof window !== 'undefined'
    ? sessionStorage.getItem('customerEmail') || 'user@example.com'
    : 'user@example.com';

export default function CustomerSubscriptions() {
  const [emailInput, setEmailInput] = useState(defaultEmail);
  const [queryEmail, setQueryEmail] = useState(defaultEmail);

  const { data: subscriptions, isLoading } = useQuery({
    queryKey: ['user-subscriptions', queryEmail],
    queryFn: () => customerApiEndpoints.getUserSubscriptions(queryEmail).then((res) => res.data),
  });

  const handleLoad = () => {
    sessionStorage.setItem('customerEmail', emailInput);
    setQueryEmail(emailInput);
  };

  if (isLoading) {
    return <div>Loading subscriptions...</div>;
  }

  return (
    <div>
      <h1>My Subscriptions</h1>
      <div style={{ marginBottom: '20px', padding: '12px 16px', backgroundColor: '#ecf0f1', borderRadius: '6px', fontSize: '14px' }}>
        <strong>Your control:</strong> Enter your email and click <strong>Load</strong> to list your subscriptions. Click a subscription card to open it. On the subscription page you can: <strong>Activate</strong> (if pending), <strong>Change plan</strong>, <strong>Change quantity</strong>. Success messages appear after each action.
      </div>
      <div style={{ marginBottom: '20px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label>
          View subscriptions for email:
          <input
            type="email"
            value={emailInput}
            onChange={(e) => setEmailInput(e.target.value)}
            style={{ marginLeft: '8px', padding: '8px', minWidth: '240px' }}
          />
        </label>
        <button
          type="button"
          onClick={handleLoad}
          style={{
            padding: '8px 16px',
            backgroundColor: '#3498db',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Load
        </button>
      </div>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        {subscriptions?.map((sub: any, index: number) => (
          <Link
            key={sub.id ?? index}
            to={`/subscriptions/${sub.amp_subscription_id ?? sub.subscription_id}`}
            style={{ textDecoration: 'none', color: 'inherit' }}
          >
            <div
              style={{
                backgroundColor: 'white',
                padding: '20px',
                borderRadius: '8px',
                boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
                cursor: 'pointer',
              }}
            >
              <h3 style={{ color: '#3498db' }}>
                Subscription {sub.amp_subscription_id ?? sub.subscription_id}
              </h3>
              <p style={{ color: '#7f8c8d', marginTop: '8px' }}>
                Plan: {sub.amp_plan_id ?? sub.plan_id}
              </p>
              <p style={{ color: '#7f8c8d' }}>
                Status: {sub.subscription_status ?? sub.status}
              </p>
              <p style={{ color: '#7f8c8d' }}>Quantity: {sub.amp_quantity ?? sub.quantity}</p>
            </div>
          </Link>
        ))}
      </div>
      {(!subscriptions || subscriptions.length === 0) && (
        <p style={{ color: '#7f8c8d', marginTop: '20px' }}>No subscriptions found for this email.</p>
      )}
    </div>
  );
}

