import { useQuery } from '@tanstack/react-query';
import { customerApiEndpoints } from '../../api/client';

export default function CustomerSubscriptions() {
  const { data: subscriptions, isLoading } = useQuery({
    queryKey: ['user-subscriptions'],
    queryFn: () => customerApiEndpoints.getUserSubscriptions('user@example.com').then(res => res.data),
  });

  if (isLoading) {
    return <div>Loading subscriptions...</div>;
  }

  return (
    <div>
      <h1>My Subscriptions</h1>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        {subscriptions?.map((sub: any, index: number) => (
          <div
            key={index}
            style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
            }}
          >
            <h3>Subscription {sub.subscription_id}</h3>
            <p style={{ color: '#7f8c8d', marginTop: '8px' }}>Plan: {sub.plan_id}</p>
            <p style={{ color: '#7f8c8d' }}>Status: {sub.status}</p>
            <p style={{ color: '#7f8c8d' }}>Quantity: {sub.quantity}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

