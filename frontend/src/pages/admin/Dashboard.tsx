import { useQuery } from '@tanstack/react-query';
import { subscriptionsApi, plansApi, offersApi } from '../../api/client';

export default function AdminDashboard() {
  const { data: subscriptions } = useQuery({
    queryKey: ['subscriptions'],
    queryFn: () => subscriptionsApi.getAll().then(res => res.data),
  });

  const { data: plans } = useQuery({
    queryKey: ['plans'],
    queryFn: () => plansApi.getAll().then(res => res.data),
  });

  const { data: offers } = useQuery({
    queryKey: ['offers'],
    queryFn: () => offersApi.getAll().then(res => res.data),
  });

  const activeSubscriptions = subscriptions?.filter(s => s.is_active) || [];
  const pendingSubscriptions = subscriptions?.filter(s => s.subscription_status === 'PendingFulfillmentStart') || [];

  return (
    <div>
      <h1>Dashboard</h1>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <h3>Total Subscriptions</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0' }}>
            {subscriptions?.length || 0}
          </p>
        </div>
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <h3>Active Subscriptions</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0', color: '#27ae60' }}>
            {activeSubscriptions.length}
          </p>
        </div>
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <h3>Pending Activation</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0', color: '#e67e22' }}>
            {pendingSubscriptions.length}
          </p>
        </div>
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <h3>Plans</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0' }}>
            {plans?.length || 0}
          </p>
        </div>
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <h3>Offers</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0' }}>
            {offers?.length || 0}
          </p>
        </div>
      </div>
    </div>
  );
}

