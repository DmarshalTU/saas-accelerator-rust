import { useQuery } from '@tanstack/react-query';
import { offersApi } from '../../api/client';

export default function AdminOffers() {
  const { data: offers, isLoading } = useQuery({
    queryKey: ['offers'],
    queryFn: () => offersApi.getAll().then(res => res.data),
  });

  if (isLoading) {
    return <div>Loading offers...</div>;
  }

  return (
    <div>
      <h1>Offers</h1>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        {offers?.map((offer) => (
          <div
            key={offer.id}
            style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
            }}
          >
            <h3>{offer.offer_name || offer.offer_id}</h3>
            <p style={{ color: '#7f8c8d', marginTop: '8px' }}>Offer ID: {offer.offer_id}</p>
            <p style={{ color: '#7f8c8d' }}>GUID: {offer.offer_guid}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

