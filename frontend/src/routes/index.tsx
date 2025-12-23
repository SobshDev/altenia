import { createBrowserRouter, Navigate } from 'react-router';
import { LoginPage } from '@/features/auth/pages/LoginPage';
import { RegisterPage } from '@/features/auth/pages/RegisterPage';
import { AccountPage } from '@/features/settings/pages/AccountPage';
import { AppLayout } from '@/shared/components/layout/AppLayout';
import { useAuthStore } from '@/stores/authStore';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

function DashboardPage() {
  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold text-foreground">Dashboard</h1>
      <p className="mt-2 text-foreground-muted">Welcome to Altenia</p>
    </div>
  );
}

function PlaceholderPage({ title }: { title: string }) {
  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold text-foreground">{title}</h1>
      <p className="mt-2 text-foreground-muted">Coming soon...</p>
    </div>
  );
}

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/register',
    element: <RegisterPage />,
  },
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <AppLayout />
      </ProtectedRoute>
    ),
    children: [
      {
        index: true,
        element: <DashboardPage />,
      },
      {
        path: 'logs',
        element: <PlaceholderPage title="Logs" />,
      },
      {
        path: 'metrics',
        element: <PlaceholderPage title="Metrics" />,
      },
      {
        path: 'traces',
        element: <PlaceholderPage title="Traces" />,
      },
      {
        path: 'alerts',
        element: <PlaceholderPage title="Alerts" />,
      },
      {
        path: 'settings/account',
        element: <AccountPage />,
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
]);
