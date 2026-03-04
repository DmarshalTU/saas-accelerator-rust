import { Outlet, Link, useLocation } from 'react-router-dom';

export default function AdminLayout() {
  const location = useLocation();

  const navItems = [
    { path: '/admin/dashboard', label: 'Dashboard' },
    { path: '/admin/subscriptions', label: 'Subscriptions' },
    { path: '/admin/plans', label: 'Plans' },
    { path: '/admin/offers', label: 'Offers' },
    { path: '/admin/users', label: 'Users' },
    { path: '/admin/scheduler', label: 'Scheduler' },
    { path: '/admin/config', label: 'Configuration' },
  ];

  return (
    <div style={{ display: 'flex', minHeight: '100vh' }}>
      <aside style={{
        width: '250px',
        backgroundColor: '#2c3e50',
        color: 'white',
        padding: '20px',
      }}>
        <h2 style={{ marginBottom: '4px' }}>Admin Portal</h2>
        <p style={{ fontSize: '12px', color: 'rgba(255,255,255,0.8)', marginBottom: '20px' }}>
          Subscriptions, plans, offers, config
        </p>
        <nav>
          <ul style={{ listStyle: 'none' }}>
            {navItems.map((item) => (
              <li key={item.path} style={{ marginBottom: '10px' }}>
                <Link
                  to={item.path}
                  style={{
                    color: location.pathname === item.path ? '#3498db' : 'white',
                    textDecoration: 'none',
                    display: 'block',
                    padding: '10px',
                    borderRadius: '4px',
                    backgroundColor: location.pathname === item.path ? '#34495e' : 'transparent',
                  }}
                >
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
        </nav>
      </aside>
      <main style={{ flex: 1, padding: '20px' }}>
        <Outlet />
      </main>
    </div>
  );
}

