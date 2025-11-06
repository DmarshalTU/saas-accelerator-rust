# SaaS Accelerator Frontend

Modern React frontend for the SaaS Accelerator - Rust Edition with hot reload support.

## Features

- ⚡ Vite for fast hot module replacement
- ⚛️ React 18 with TypeScript
- 🎯 React Query for data fetching
- 🎨 Modern UI with inline styles (ready for CSS framework integration)
- 🔄 Real-time updates via React Query

## Development

```bash
# Install dependencies
npm install

# Start development server with hot reload
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

The frontend will be available at `http://localhost:5173`.

## Environment Variables

Create a `.env` file:

```env
VITE_ADMIN_API_URL=http://localhost:3000
VITE_CUSTOMER_API_URL=http://localhost:3001
```

## Project Structure

```
src/
├── api/           # API client and endpoints
├── layouts/       # Layout components
├── pages/         # Page components
│   ├── admin/     # Admin portal pages
│   └── customer/  # Customer portal pages
└── App.tsx        # Main app component
```

## Pages

### Admin Portal (`/admin`)
- Dashboard - Overview of subscriptions, plans, and offers
- Subscriptions - Manage all subscriptions with activation and audit logs
- Plans - View all available plans
- Offers - View all offers
- Configuration - Manage application settings

### Customer Portal (`/`)
- Landing Page - Subscription activation with token support
- Subscriptions - View user's subscriptions

