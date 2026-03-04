import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import AdminLayout from './layouts/AdminLayout';
import CustomerLayout from './layouts/CustomerLayout';
import AdminDashboard from './pages/admin/Dashboard';
import AdminSubscriptions from './pages/admin/Subscriptions';
import AdminPlans from './pages/admin/Plans';
import AdminOffers from './pages/admin/Offers';
import AdminConfig from './pages/admin/Config';
import AdminSubscriptionDetail from './pages/admin/SubscriptionDetail';
import AdminKnownUsers from './pages/admin/KnownUsers';
import AdminApplicationLog from './pages/admin/ApplicationLog';
import AdminEmailTemplates from './pages/admin/EmailTemplates';
import AdminScheduler from './pages/admin/Scheduler';
import AdminSchedulerLog from './pages/admin/SchedulerLog';
import AdminPlanDetail from './pages/admin/PlanDetail';
import AdminOfferDetail from './pages/admin/OfferDetail';
import CustomerLanding from './pages/customer/Landing';
import CustomerSubscriptions from './pages/customer/Subscriptions';
import CustomerSubscriptionDetail from './pages/customer/SubscriptionDetail';
import CustomerPrivacy from './pages/customer/Privacy';
import CustomerProcessMessage from './pages/customer/ProcessMessage';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/admin" element={<AdminLayout />}>
          <Route index element={<Navigate to="dashboard" replace />} />
          <Route path="dashboard" element={<AdminDashboard />} />
          <Route path="subscriptions" element={<AdminSubscriptions />} />
          <Route path="subscriptions/:id" element={<AdminSubscriptionDetail />} />
          <Route path="plans" element={<AdminPlans />} />
          <Route path="plans/:guid" element={<AdminPlanDetail />} />
          <Route path="offers" element={<AdminOffers />} />
          <Route path="offers/:guid" element={<AdminOfferDetail />} />
          <Route path="config" element={<AdminConfig />} />
          <Route path="users" element={<AdminKnownUsers />} />
          <Route path="logs" element={<AdminApplicationLog />} />
          <Route path="email-templates" element={<AdminEmailTemplates />} />
          <Route path="scheduler" element={<AdminScheduler />} />
          <Route path="scheduler/:id/log" element={<AdminSchedulerLog />} />
        </Route>
        <Route path="/" element={<CustomerLayout />}>
          <Route index element={<CustomerLanding />} />
          <Route path="subscriptions" element={<CustomerSubscriptions />} />
          <Route path="subscriptions/:id" element={<CustomerSubscriptionDetail />} />
          <Route path="privacy" element={<CustomerPrivacy />} />
          <Route path="process-message" element={<CustomerProcessMessage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;

