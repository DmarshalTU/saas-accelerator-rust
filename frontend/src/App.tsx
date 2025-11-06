import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import AdminLayout from './layouts/AdminLayout';
import CustomerLayout from './layouts/CustomerLayout';
import AdminDashboard from './pages/admin/Dashboard';
import AdminSubscriptions from './pages/admin/Subscriptions';
import AdminPlans from './pages/admin/Plans';
import AdminOffers from './pages/admin/Offers';
import AdminConfig from './pages/admin/Config';
import CustomerLanding from './pages/customer/Landing';
import CustomerSubscriptions from './pages/customer/Subscriptions';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/admin" element={<AdminLayout />}>
          <Route index element={<Navigate to="dashboard" replace />} />
          <Route path="dashboard" element={<AdminDashboard />} />
          <Route path="subscriptions" element={<AdminSubscriptions />} />
          <Route path="plans" element={<AdminPlans />} />
          <Route path="offers" element={<AdminOffers />} />
          <Route path="config" element={<AdminConfig />} />
        </Route>
        <Route path="/" element={<CustomerLayout />}>
          <Route index element={<CustomerLanding />} />
          <Route path="subscriptions" element={<CustomerSubscriptions />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;

