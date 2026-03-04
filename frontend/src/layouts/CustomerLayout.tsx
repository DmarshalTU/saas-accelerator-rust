import { Outlet, Link, useLocation } from 'react-router-dom';

export default function CustomerLayout() {
  const location = useLocation();
  return (
    <div style={{ minHeight: '100vh' }}>
      <header style={{
        backgroundColor: '#3498db',
        color: 'white',
        padding: '20px',
        marginBottom: '20px',
      }}>
        <h1 style={{ marginBottom: '12px' }}>SaaS Accelerator - Customer Portal</h1>
        <nav style={{ display: 'flex', gap: '16px' }}>
          <Link
            to="/"
            style={{
              color: location.pathname === '/' ? '#fff' : 'rgba(255,255,255,0.85)',
              textDecoration: 'none',
              fontWeight: location.pathname === '/' ? 'bold' : 'normal',
            }}
          >
            Home / Landing
          </Link>
          <Link
            to="/subscriptions"
            style={{
              color: location.pathname === '/subscriptions' ? '#fff' : 'rgba(255,255,255,0.85)',
              textDecoration: 'none',
              fontWeight: location.pathname === '/subscriptions' ? 'bold' : 'normal',
            }}
          >
            My Subscriptions
          </Link>
          <Link
            to="/privacy"
            style={{
              color: location.pathname === '/privacy' ? '#fff' : 'rgba(255,255,255,0.85)',
              textDecoration: 'none',
              fontWeight: location.pathname === '/privacy' ? 'bold' : 'normal',
            }}
          >
            Privacy
          </Link>
        </nav>
      </header>
      <main style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto' }}>
        <Outlet />
      </main>
    </div>
  );
}

