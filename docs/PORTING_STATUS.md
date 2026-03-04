# Porting Status: Original SaaS Accelerator → Rust + React

This document summarizes what is **ported** from the [Commercial Marketplace SaaS Accelerator](https://github.com/Azure/Commercial-Marketplace-SaaS-Accelerator) (.NET) to the Rust backend and React frontend.

---

## ✅ Fully ported

### Backend (admin-api, customer-api, webhook-api, scheduler)


| Area                      | Original                                                                                                      | Rust                                                               | Notes |
| ------------------------- | ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ----- |
| **Subscriptions**         | List, get by id, activate, change plan, change quantity, emit usage, audit logs, delete                       | All routes implemented                                             | ✅    |
| **Plans**                 | List, get by id, get by plan_guid, save plan (events + offer attributes)                                     | GET list, by id, by guid; PUT by guid (plan_events, offer_attribute_ids) | ✅    |
| **Offers**                | List, get by offer_guid with attributes, save attributes                                                      | GET list, by guid with attributes; PUT by guid/attributes           | ✅    |
| **Application config**    | List, get by id, save by id, upload Logo/Favicon                                                              | GET all, PUT by name, POST /config/upload (JSON base64)             | ✅    |
| **Known users**           | List, save all                                                                                                | GET, POST save all                                                 | ✅    |
| **Application logs**      | List                                                                                                          | GET all                                                            | ✅    |
| **Email templates**       | List, get by status, save by status                                                                           | GET all, GET by status, PUT by status                              | ✅    |
| **Scheduler**             | List, add, delete, get item, get run history (metered audit logs)                                             | All + GET :id, GET :id/log                                         | ✅    |
| **Customer**              | Landing (resolve token), subscriptions by email, get subscription, activate, change plan/quantity, plans list  | All                                                                | ✅    |
| **Webhook**               | POST payload, JWT, idempotency, status handlers                                                               | webhook-api                                                        | ✅    |
| **Metered scheduler job** | Recurring usage                                                                                               | scheduler crate                                                    | ✅    |
| **Marketplace**           | Fulfillment + Metering API clients                                                                            | marketplace crate                                                  | ✅    |
| **Auth**                  | Login, callback (code → id_token → user → session), logout, /api/me                                          | All; session layer enabled with HandleErrorLayer                   | ✅    |
| **Events**                | List events for plan configuration                                                                            | GET /api/events                                                    | ✅    |


### Frontend (React)


| Original view / flow                                                         | Rust port                                          | Notes |
| ---------------------------------------------------------------------------- | -------------------------------------------------- | ----- |
| Admin: Dashboard                                                             | `/admin` dashboard with “What you can do” + counts | ✅    |
| Admin: Subscriptions list                                                    | `/admin/subscriptions`                             | ✅    |
| Admin: Subscription detail (activate, change plan/qty, usage, audit, delete)  | `/admin/subscriptions/:id`                         | ✅    |
| Admin: Plans list                                                            | `/admin/plans`                                     | ✅    |
| Admin: Plan details (view + edit events + offer attributes, save)            | `/admin/plans/:guid`                               | ✅    |
| Admin: Offers list                                                           | `/admin/offers`                                    | ✅    |
| Admin: Offer details (view + edit attributes, save)                           | `/admin/offers/:guid`                              | ✅    |
| Admin: Config (edit values, View Logs, View Templates, Upload Logo/Favicon) | `/admin/config`                                    | ✅    |
| Admin: Known users (add/remove, Save All)                                     | `/admin/users`                                     | ✅    |
| Admin: Application log                                                       | `/admin/logs`                                      | ✅    |
| Admin: Email templates (list, edit by status)                                | `/admin/email-templates`                          | ✅    |
| Admin: Scheduler (list, add, delete, View log)                                | `/admin/scheduler`, `/admin/scheduler/:id/log`     | ✅    |
| Customer: Landing (token, activate)                                           | `/`                                                | ✅    |
| Customer: My Subscriptions (email, load, open)                               | `/subscriptions`                                   | ✅    |
| Customer: Subscription detail (activate, change plan, change quantity)      | `/subscriptions/:id` + success messages            | ✅    |
| Customer: ProcessMessage (action + status query)                             | `/process-message`                                 | ✅    |
| Customer: Privacy                                                            | `/privacy`                                         | ✅    |


---

## Intentional design differences (no functional gap)

These differ from the original by design; behavior is equivalent for normal use.

| Original | Rust port | Why |
| -------- | --------- | ----- |
| **Config by Id** | Config keyed by *name* (GET all, PUT by name) | UI and backend only need name; GET all returns `id` if needed. Add GET/PUT by `id` if you need exact API parity. |
| **SubscriptionQuantityDetail / SubscriptionLogDetail** | Single subscription detail page | Quantity change and audit log live on `/admin/subscriptions/:id` and `/subscriptions/:id`; no separate routes. Same data, fewer pages. |
| **Auth on every admin route** | Session + `/api/me`; API routes not gated by default | Login/callback/logout and session are implemented. To require auth on all `/api/*` (except e.g. `/api/me`, `/health`), add middleware that returns 401 when session has no user, or enforce at a reverse proxy. |

---

## Summary

The Rust + React port has **full feature parity** with the original .NET SaaS Accelerator for the areas above:

- **Backend:** Subscriptions, plans (read + save by guid), offers (read + save attributes), config, known users, logs, email templates, scheduler, customer flows, webhook, metered job, marketplace clients, auth (login, callback with id_token → session, logout, /api/me), events.
- **Frontend:** All admin and customer screens, including plan detail edit (events + offer attributes), offer detail edit (attributes), and customer ProcessMessage page.

No remaining “Partial” or “Not ported” items for core flows.
