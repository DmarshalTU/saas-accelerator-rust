const cardStyle = {
  backgroundColor: 'white',
  padding: '24px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  maxWidth: '720px',
};

export default function Privacy() {
  return (
    <div>
      <h1>Privacy Policy</h1>
      <div style={cardStyle}>
        <p>Use this page to detail your site's privacy policy.</p>
      </div>
    </div>
  );
}
