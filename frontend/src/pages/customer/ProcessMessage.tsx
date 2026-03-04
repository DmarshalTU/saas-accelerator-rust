import { Link, useSearchParams } from 'react-router-dom';

const cardStyle = {
  backgroundColor: 'white',
  padding: '24px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  maxWidth: '560px',
  margin: '0 auto',
};

export default function ProcessMessage() {
  const [searchParams] = useSearchParams();
  const action = searchParams.get('action') ?? '';
  const status = searchParams.get('status') ?? '';

  const message = status || action ? `Operation: ${action || status}. ${status ? `Status: ${status}.` : ''}` : 'Done.';

  return (
    <div style={cardStyle}>
      <h1>Process message</h1>
      <p style={{ marginTop: '16px', fontSize: '16px' }}>{message}</p>
      <p style={{ marginTop: '24px' }}>
        <Link to="/subscriptions" style={{ color: '#3498db' }}>← Back to My Subscriptions</Link>
      </p>
      <p style={{ marginTop: '8px' }}>
        <Link to="/" style={{ color: '#3498db' }}>Go to Home</Link>
      </p>
    </div>
  );
}
