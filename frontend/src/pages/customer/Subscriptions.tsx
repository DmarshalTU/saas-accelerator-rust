import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { customerApiEndpoints } from '../../api/client';
import { useState, useCallback } from 'react';

const STATUS_COLORS: Record<string, string> = {
  Subscribed:              '#27ae60',
  PendingFulfillmentStart: '#f39c12',
  Unsubscribed:            '#95a5a6',
  Suspended:               '#e74c3c',
};

export default function CustomerSubscriptions() {
  const savedEmail = typeof window !== 'undefined' ? sessionStorage.getItem('customerEmail') ?? '' : '';
  const [emailInput, setEmailInput] = useState(savedEmail);
  const [queryEmail, setQueryEmail] = useState(savedEmail);

  const { data: subscriptions, isLoading, error } = useQuery({
    queryKey: ['user-subscriptions', queryEmail],
    queryFn: () => customerApiEndpoints.getUserSubscriptions(queryEmail).then((r) => r.data),
    enabled: queryEmail.length > 0,
  });

  const handleLoad = useCallback(() => {
    const e = emailInput.trim();
    if (!e) return;
    sessionStorage.setItem('customerEmail', e);
    setQueryEmail(e);
  }, [emailInput]);

  return (
    <div>
      <h1>My Subscriptions</h1>

      {/* Email lookup */}
      <div style={{ marginBottom: '20px', display: 'flex', gap: '8px', alignItems: 'flex-end', flexWrap: 'wrap' }}>
        <div>
          <label style={{ display: 'block', marginBottom: '4px', fontWeight: 500 }}>
            Your email address
          </label>
          <input
            type="email"
            value={emailInput}
            onChange={(e) => setEmailInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleLoad()}
            placeholder="you@example.com"
            style={{ padding: '8px', minWidth: '260px', borderRadius: '4px', border: '1px solid #ccc' }}
          />
        </div>
        <button
          type="button"
          onClick={handleLoad}
          disabled={!emailInput.trim()}
          style={{ padding: '8px 20px', backgroundColor: '#3498db', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
        >
          Load subscriptions
        </button>
      </div>

      {/* State indicators */}
      {!queryEmail && (
        <p style={{ color: '#7f8c8d' }}>Enter your email address above to view your subscriptions.</p>
      )}
      {isLoading && <p>Loading…</p>}
      {error && (
        <p style={{ color: '#e74c3c' }}>Failed to load subscriptions. Please check the email and try again.</p>
      )}

      {/* Subscription cards */}
      {queryEmail && !isLoading && !error && (
        <>
          {(!subscriptions || subscriptions.length === 0)
            ? <p style={{ color: '#7f8c8d' }}>No subscriptions found for <strong>{queryEmail}</strong>.</p>
            : (
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '16px', marginTop: '8px' }}>
                {subscriptions.map((sub: any, index: number) => {
                  const status = sub.subscription_status ?? sub.status ?? 'Unknown';
                  return (
                    <Link
                      key={sub.id ?? index}
                      to={`/subscriptions/${sub.amp_subscription_id ?? sub.subscription_id}`}
                      style={{ textDecoration: 'none', color: 'inherit' }}
                    >
                      <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)', borderLeft: `4px solid ${STATUS_COLORS[status] ?? '#3498db'}`, transition: 'box-shadow .2s' }}
                        onMouseEnter={(e) => (e.currentTarget.style.boxShadow = '0 4px 12px rgba(0,0,0,0.15)')}
                        onMouseLeave={(e) => (e.currentTarget.style.boxShadow = '0 2px 4px rgba(0,0,0,0.1)')}
                      >
                        <h3 style={{ margin: '0 0 8px', fontSize: '14px', color: '#555', wordBreak: 'break-all' }}>
                          {sub.amp_subscription_id ?? sub.subscription_id}
                        </h3>
                        <p style={{ margin: '4px 0', fontWeight: 600 }}>{sub.amp_plan_id ?? sub.plan_id}</p>
                        <p style={{ margin: '4px 0', fontSize: '13px' }}>
                          <span style={{ display: 'inline-block', padding: '2px 8px', borderRadius: '12px', backgroundColor: STATUS_COLORS[status] ?? '#3498db', color: 'white', fontSize: '12px' }}>
                            {status}
                          </span>
                        </p>
                        <p style={{ margin: '4px 0', color: '#7f8c8d', fontSize: '13px' }}>Qty: {sub.amp_quantity ?? sub.quantity}</p>
                      </div>
                    </Link>
                  );
                })}
              </div>
            )}
        </>
      )}
    </div>
  );
}
