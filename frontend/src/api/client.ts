import axios from 'axios';

// Empty or unset = same-origin (relative /api); set URL = that origin (e.g. dev or cross-site)
const ADMIN_API_BASE = import.meta.env.VITE_ADMIN_API_URL;
const CUSTOMER_API_BASE = import.meta.env.VITE_CUSTOMER_API_URL;

const adminBase = (ADMIN_API_BASE && String(ADMIN_API_BASE).trim() !== '') ? `${ADMIN_API_BASE}/api` : '/api';
const customerBase = (CUSTOMER_API_BASE && String(CUSTOMER_API_BASE).trim() !== '') ? `${CUSTOMER_API_BASE}/api` : '/api';

export const adminApi = axios.create({
  baseURL: adminBase,
  headers: {
    'Content-Type': 'application/json',
  },
});

export const customerApi = axios.create({
  baseURL: customerBase,
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
  plan_name?: string | null;
  display_name?: string | null;
  offer_id: number | string;
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

export interface PlanEventsMapping {
  id: number;
  plan_id: number;
  event_id: number;
  success_state_emails: string | null;
  failure_state_emails: string | null;
  create_date: string | null;
  copy_to_customer: boolean | null;
}

export interface PlanDetailResponse extends Plan {
  plan_events: PlanEventsMapping[];
  offer_attribute_ids: number[];
}

export const plansApi = {
  getAll: () => adminApi.get<Plan[]>('/plans'),
  getById: (id: number) => adminApi.get<Plan>(`/plans/${id}`),
  getByGuid: (guid: string) => adminApi.get<PlanDetailResponse>(`/plans/by-guid/${guid}`),
  saveByGuid: (guid: string, body: { plan_events?: Array<{ id?: number; event_id: number; success_state_emails?: string; failure_state_emails?: string; copy_to_customer?: boolean }>; offer_attribute_ids?: number[] }) =>
    adminApi.put(`/plans/by-guid/${guid}`, body),
};

export interface Event {
  id: number;
  events_name: string | null;
  is_active: boolean | null;
  create_date: string | null;
}

export const eventsApi = {
  getAll: () => adminApi.get<Event[]>('/events'),
};

export const offersApi = {
  getAll: () => adminApi.get<Offer[]>('/offers'),
  getByGuid: (guid: string) => adminApi.get<OfferWithAttributes>(`/offers/by-guid/${guid}`),
  saveAttributes: (guid: string, attributes: Array<{ id?: number; parameter_id?: string; display_name?: string; description?: string; type?: string; values_list?: string }>) =>
    adminApi.put(`/offers/by-guid/${guid}/attributes`, { attributes }),
};

export interface OfferAttribute {
  id: number;
  offer_id: number;
  parameter_id: string | null;
  display_name: string | null;
  description: string | null;
  type: string | null;
  values_list: string | null;
  create_date: string | null;
}

export interface OfferWithAttributes extends Offer {
  attributes: OfferAttribute[];
}

export const configApi = {
  getAll: () => adminApi.get<ApplicationConfiguration[]>('/config'),
  update: (name: string, value: string) => adminApi.put(`/config/${name}`, { value }),
  uploadFile: (configName: string, base64Value: string) =>
    adminApi.post('/config/upload', { config_name: configName, value: base64Value }),
};

export interface KnownUser {
  id: number;
  user_email: string;
  role_id: number;
}

export interface ApplicationLog {
  id: number;
  action_time: string | null;
  log_detail: string | null;
}

export interface EmailTemplate {
  id: number;
  status: string | null;
  description: string | null;
  insert_date: string | null;
  template_body: string | null;
  subject: string | null;
  to_recipients: string | null;
  cc: string | null;
  bcc: string | null;
  is_active: boolean;
}

export interface SchedulerItem {
  id: number;
  scheduler_name: string;
  subscription_id: number;
  plan_id: number;
  dimension_id: number;
  frequency_id: number;
  quantity: number;
  start_date: string;
  next_run_time: string | null;
}

export interface MeteredDimension {
  id: number;
  plan_id: number;
  dimension: string;
  description: string | null;
}

export interface SchedulerFrequency {
  id: number;
  frequency: string;
}

export const knownUsersApi = {
  getAll: () => adminApi.get<KnownUser[]>('/known-users'),
  saveAll: (users: { user_email: string; role_id?: number }[]) =>
    adminApi.post('/known-users', users),
};

export const applicationLogsApi = {
  getAll: () => adminApi.get<ApplicationLog[]>('/application-logs'),
};

export const emailTemplatesApi = {
  getAll: () => adminApi.get<EmailTemplate[]>('/email-templates'),
  getByStatus: (status: string) => adminApi.get<EmailTemplate | null>(`/email-templates/${encodeURIComponent(status)}`),
  save: (status: string, body: Partial<EmailTemplate>) =>
    adminApi.put(`/email-templates/${encodeURIComponent(status)}`, body),
};

export interface MeteredAuditLog {
  id: number;
  subscription_id: number;
  request_json: string | null;
  response_json: string | null;
  status_code: string | null;
  created_date: string | null;
  subscription_usage_date: string | null;
  run_by: string | null;
}

export const schedulerApi = {
  getList: () => adminApi.get<SchedulerItem[]>('/scheduler'),
  getById: (id: number) => adminApi.get<SchedulerItem>(`/scheduler/${id}`),
  getLog: (id: number) => adminApi.get<MeteredAuditLog[]>(`/scheduler/${id}/log`),
  getFrequencies: () => adminApi.get<SchedulerFrequency[]>('/scheduler/frequencies'),
  getDimensionsBySubscription: (subscriptionId: number) =>
    adminApi.get<MeteredDimension[]>('/scheduler/dimensions', { params: { subscription_id: subscriptionId } }),
  add: (body: {
    scheduler_name: string;
    subscription_id: number;
    plan_id: number;
    dimension_id: number;
    frequency_id: number;
    quantity: number;
    start_date: string;
  }) => adminApi.post('/scheduler', body),
  delete: (id: number) => adminApi.delete(`/scheduler/${id}`),
};

export const customerApiEndpoints = {
  getLanding: (token?: string) => customerApi.get('/landing', { params: { token } }),
  getSubscription: (id: string) => customerApi.get(`/subscriptions/${id}`),
  activateSubscription: (id: string) => customerApi.post(`/subscriptions/${id}/activate`),
  getUserSubscriptions: (email: string) => customerApi.get(`/users/${email}/subscriptions`),
  getPlans: () => customerApi.get<Plan[]>('/plans'),
  changePlan: (id: string, planId: string) =>
    customerApi.patch(`/subscriptions/${id}/plan`, { plan_id: planId }),
  changeQuantity: (id: string, quantity: number) =>
    customerApi.patch(`/subscriptions/${id}/quantity`, { quantity }),
};

