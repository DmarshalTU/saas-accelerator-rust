import { Outlet } from 'react-router-dom';

export default function CustomerLayout() {
  return (
    <div style={{ minHeight: '100vh' }}>
      <header style={{
        backgroundColor: '#3498db',
        color: 'white',
        padding: '20px',
        marginBottom: '20px',
      }}>
        <h1>SaaS Accelerator - Customer Portal</h1>
      </header>
      <main style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto' }}>
        <Outlet />
      </main>
    </div>
  );
}

