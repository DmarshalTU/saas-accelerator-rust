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
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        {plans?.map((plan) => (
          <div
            key={plan.id}
            style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
            }}
          >
            <h3>{plan.plan_name || plan.plan_id}</h3>
            <p style={{ color: '#7f8c8d', marginTop: '8px' }}>Plan ID: {plan.plan_id}</p>
            <p style={{ color: '#7f8c8d' }}>Offer ID: {plan.offer_id}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

