import { useSearchParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation } from '@tanstack/react-query';
import { customerApiEndpoints } from '../../api/client';
import { useState } from 'react';

export default function CustomerLanding() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const token = searchParams.get('token');
  const [subscriptionId, setSubscriptionId] = useState<string>('');

  const { data: landingData } = useQuery({
    queryKey: ['landing', token],
    queryFn: () => customerApiEndpoints.getLanding(token || undefined).then(res => res.data),
    enabled: !!token,
  });

  const activateMutation = useMutation({
    mutationFn: (id: string) => customerApiEndpoints.activateSubscription(id),
    onSuccess: () => {
      navigate('/subscriptions');
    },
  });

  const handleActivate = () => {
    if (subscriptionId) {
      activateMutation.mutate(subscriptionId);
    } else if (landingData?.subscription_id) {
      activateMutation.mutate(landingData.subscription_id);
    }
  };

  return (
    <div style={{
      backgroundColor: 'white',
      padding: '40px',
      borderRadius: '8px',
      boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
      maxWidth: '600px',
      margin: '0 auto',
    }}>
      <h1>Welcome to SaaS Accelerator</h1>
      {token && landingData ? (
        <div style={{ marginTop: '20px' }}>
          <p>Subscription found!</p>
          <div style={{ marginTop: '20px', padding: '20px', backgroundColor: '#ecf0f1', borderRadius: '4px' }}>
            <p><strong>Subscription ID:</strong> {landingData.subscription_id}</p>
            <p><strong>Plan ID:</strong> {landingData.plan_id}</p>
            <p><strong>Offer ID:</strong> {landingData.offer_id}</p>
          </div>
          <button
            onClick={handleActivate}
            disabled={activateMutation.isPending}
            style={{
              marginTop: '20px',
              padding: '12px 24px',
              backgroundColor: '#27ae60',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '16px',
            }}
          >
            {activateMutation.isPending ? 'Activating...' : 'Activate Subscription'}
          </button>
        </div>
      ) : (
        <div style={{ marginTop: '20px' }}>
          <p>No subscription token found. Please provide a token or subscription ID to continue.</p>
          <div style={{ marginTop: '20px' }}>
            <input
              type="text"
              placeholder="Enter subscription ID"
              value={subscriptionId}
              onChange={(e) => setSubscriptionId(e.target.value)}
              style={{
                padding: '10px',
                border: '1px solid #bdc3c7',
                borderRadius: '4px',
                width: '100%',
                marginBottom: '10px',
              }}
            />
            <button
              onClick={handleActivate}
              disabled={!subscriptionId || activateMutation.isPending}
              style={{
                padding: '12px 24px',
                backgroundColor: '#3498db',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                width: '100%',
              }}
            >
              {activateMutation.isPending ? 'Activating...' : 'Activate Subscription'}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

