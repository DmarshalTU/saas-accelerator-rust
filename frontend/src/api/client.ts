import axios from 'axios';

const ADMIN_API_BASE = import.meta.env.VITE_ADMIN_API_URL || 'http://localhost:3000';
const CUSTOMER_API_BASE = import.meta.env.VITE_CUSTOMER_API_URL || 'http://localhost:3001';

export const adminApi = axios.create({
  baseURL: `${ADMIN_API_BASE}/api`,
  headers: {
    'Content-Type': 'application/json',
  },
});

export const customerApi = axios.create({
  baseURL: `${CUSTOMER_API_BASE}/api`,
  headers: {
    'Content-Type': 'application/json',
  },
});

export interface Subscription {
  id: number;
  amp_subscription_id: string;
  subscription_status: string;
  amp_plan_id: string;
  amp_offer_id: string;
  amp_quantity: number;
  is_active: boolean | null;
  name: string | null;
  purchaser_email: string | null;
  create_date: string | null;
}

export interface Plan {
  id: number;
  plan_id: string;
  plan_name: string | null;
  offer_id: number;
  plan_guid: string;
}

export interface Offer {
  id: number;
  offer_id: string;
  offer_name: string | null;
  offer_guid: string;
}

export interface SubscriptionAuditLog {
  id: number;
  subscription_id: number;
  attribute: string | null;
  old_value: string | null;
  new_value: string | null;
  create_date: string | null;
}

export interface ApplicationConfiguration {
  id: number;
  name: string;
  value: string | null;
  description: string | null;
}

export const subscriptionsApi = {
  getAll: () => adminApi.get<Subscription[]>('/subscriptions'),
  getById: (id: string) => adminApi.get<Subscription>(`/subscriptions/${id}`),
  activate: (id: string) => adminApi.post(`/subscriptions/${id}/activate`),
  changePlan: (id: string, planId: string) =>
    adminApi.patch(`/subscriptions/${id}/plan`, { plan_id: planId }),
  changeQuantity: (id: string, quantity: number) =>
    adminApi.patch(`/subscriptions/${id}/quantity`, { quantity }),
  emitUsage: (id: string, dimension: string, quantity: number) =>
    adminApi.post(`/subscriptions/${id}/usage`, { dimension, quantity }),
  getAuditLogs: (id: string) => adminApi.get<SubscriptionAuditLog[]>(`/subscriptions/${id}/audit-logs`),
  delete: (id: string) => adminApi.delete(`/subscriptions/${id}`),
};

export const plansApi = {
  getAll: () => adminApi.get<Plan[]>('/plans'),
  getById: (id: number) => adminApi.get<Plan>(`/plans/${id}`),
};

export const offersApi = {
  getAll: () => adminApi.get<Offer[]>('/offers'),
};

export const configApi = {
  getAll: () => adminApi.get<ApplicationConfiguration[]>('/config'),
  update: (name: string, value: string) => adminApi.put(`/config/${name}`, { value }),
};

export const customerApiEndpoints = {
  getLanding: (token?: string) => customerApi.get('/landing', { params: { token } }),
  getSubscription: (id: string) => customerApi.get(`/subscriptions/${id}`),
  activateSubscription: (id: string) => customerApi.post(`/subscriptions/${id}/activate`),
  getUserSubscriptions: (email: string) => customerApi.get(`/users/${email}/subscriptions`),
};

