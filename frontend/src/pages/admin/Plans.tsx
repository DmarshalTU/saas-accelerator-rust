import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { plansApi } from '../../api/client';

export default function AdminPlans() {
  const { data: plans, isLoading } = useQuery({
    queryKey: ['plans'],
    queryFn: () => plansApi.getAll().then(res => res.data),
  });

  if (isLoading) {
    return <div>Loading plans...</div>;
  }

  return (
    <div>
      <h1>Plans</h1>
      <div style={{ marginBottom: '16px', padding: '12px 16px', backgroundColor: '#ecf0f1', borderRadius: '6px', fontSize: '14px' }}>
        <strong>What you can do:</strong> View plans below. Click a plan to open its details. Plans are synced from your marketplace offer when subscriptions are created (webhook); you cannot add plans manually here. To have plans appear, configure your offer in Partner Center and create subscriptions.
      </div>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        {plans?.map((plan) => (
          <Link
            key={plan.id}
            to={`/admin/plans/${encodeURIComponent(plan.plan_guid ?? String(plan.id))}`}
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
              <h3>{plan.plan_name || plan.plan_id}</h3>
              <p style={{ color: '#7f8c8d', marginTop: '8px' }}>Plan ID: {plan.plan_id}</p>
              <p style={{ color: '#7f8c8d' }}>Offer ID: {plan.offer_id}</p>
              <p style={{ color: '#3498db', marginTop: '8px', fontSize: '14px' }}>View details →</p>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}

