import { Link } from 'react-router-dom';
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
      <div style={{ marginBottom: '16px', padding: '12px 16px', backgroundColor: '#ecf0f1', borderRadius: '6px', fontSize: '14px' }}>
        <strong>What you can do:</strong> View offers and open one to see details and attributes. Offers come from your marketplace setup; you can edit offer attributes from the detail page.
      </div>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        {offers?.map((offer) => (
          <Link
            key={offer.id}
            to={`/admin/offers/${encodeURIComponent(offer.offer_guid)}`}
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
              <h3>{offer.offer_name || offer.offer_id}</h3>
              <p style={{ color: '#7f8c8d', marginTop: '8px' }}>Offer ID: {offer.offer_id}</p>
              <p style={{ color: '#7f8c8d' }}>GUID: {offer.offer_guid}</p>
              <p style={{ color: '#3498db', marginTop: '8px', fontSize: '14px' }}>View details →</p>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}

