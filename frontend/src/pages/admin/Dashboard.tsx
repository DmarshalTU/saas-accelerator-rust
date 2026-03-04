import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { subscriptionsApi, plansApi, offersApi } from '../../api/client';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

const actionLinkStyle = {
  display: 'block',
  padding: '8px 0',
  color: '#2980b9',
  textDecoration: 'none',
  fontWeight: 500 as const,
};

export default function AdminDashboard() {
  const { data: subscriptions } = useQuery({
    queryKey: ['subscriptions'],
    queryFn: () => subscriptionsApi.getAll().then(res => res.data),
  });

  const { data: plans } = useQuery({
    queryKey: ['plans'],
    queryFn: () => plansApi.getAll().then(res => res.data),
  });

  const { data: offers } = useQuery({
    queryKey: ['offers'],
    queryFn: () => offersApi.getAll().then(res => res.data),
  });

  const activeSubscriptions = subscriptions?.filter(s => s.is_active) || [];
  const pendingSubscriptions = subscriptions?.filter(s => s.subscription_status === 'PendingFulfillmentStart') || [];

  return (
    <div>
      <h1>Dashboard</h1>

      <div style={{ ...cardStyle, marginBottom: '24px', borderLeft: '4px solid #3498db' }}>
        <h2 style={{ marginTop: 0, marginBottom: '12px', fontSize: '1.1rem' }}>What you can do in Admin</h2>
        <ul style={{ margin: 0, paddingLeft: '20px', lineHeight: 1.8 }}>
          <li><Link to="/admin/subscriptions" style={actionLinkStyle}>Subscriptions</Link> – list all, open one to activate, change plan/quantity, record metered usage, view audit log, or delete</li>
          <li><Link to="/admin/plans" style={actionLinkStyle}>Plans</Link> – view plans (synced from your marketplace offer when subscriptions are created); open a plan to see details</li>
          <li><Link to="/admin/offers" style={actionLinkStyle}>Offers</Link> – view offers and their attributes (from your marketplace setup); open one to see details</li>
          <li><Link to="/admin/users" style={actionLinkStyle}>Users</Link> – add/remove known users (emails allowed to access admin), then Save All</li>
          <li><Link to="/admin/scheduler" style={actionLinkStyle}>Scheduler</Link> – add or delete metered-usage schedules; View log per item for run history</li>
          <li><Link to="/admin/config" style={actionLinkStyle}>Configuration</Link> – edit app settings (key/value) and upload Logo / Favicon</li>
          <li><Link to="/admin/logs" style={actionLinkStyle}>Application logs</Link> – view activity log</li>
          <li><Link to="/admin/email-templates" style={actionLinkStyle}>Email templates</Link> – view and edit templates by status</li>
        </ul>
        <p style={{ marginTop: '12px', marginBottom: 0, fontSize: '13px', color: '#7f8c8d' }}>
          Plans and offers are not added manually here; they come from your offer in Partner Center and appear when subscriptions are created (e.g. via webhook). Use Config to change behaviour and branding.
        </p>
      </div>

      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
        gap: '20px',
        marginTop: '20px',
      }}>
        <div style={cardStyle}>
          <h3>Total Subscriptions</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0' }}>
            {subscriptions?.length || 0}
          </p>
        </div>
        <div style={cardStyle}>
          <h3>Active Subscriptions</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0', color: '#27ae60' }}>
            {activeSubscriptions.length}
          </p>
        </div>
        <div style={cardStyle}>
          <h3>Pending Activation</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0', color: '#e67e22' }}>
            {pendingSubscriptions.length}
          </p>
        </div>
        <div style={cardStyle}>
          <h3>Plans</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0' }}>
            {plans?.length || 0}
          </p>
        </div>
        <div style={cardStyle}>
          <h3>Offers</h3>
          <p style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0' }}>
            {offers?.length || 0}
          </p>
        </div>
      </div>
    </div>
  );
}

