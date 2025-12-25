import { useEffect } from 'react';
import { createBrowserRouter, Navigate } from 'react-router';
import { LoginPage } from '@/features/auth/pages/LoginPage';
import { RegisterPage } from '@/features/auth/pages/RegisterPage';
import { AccountPage } from '@/features/settings/pages/AccountPage';
import { OrganizationPage } from '@/features/settings/pages/OrganizationPage';
import { ProjectLogsPage } from '@/features/projects/pages/ProjectLogsPage';
import { ProjectMetricsPage } from '@/features/projects/pages/ProjectMetricsPage';
import { ProjectTracesPage } from '@/features/projects/pages/ProjectTracesPage';
import { ProjectAlertsPage } from '@/features/projects/pages/ProjectAlertsPage';
import { ProjectSettingsPage } from '@/features/projects/pages/ProjectSettingsPage';
import { AppLayout } from '@/shared/components/layout/AppLayout';
import { UsernamePromptModal } from '@/shared/components/UsernamePromptModal';
import { useAuthStore } from '@/stores/authStore';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading, user, checkAuth } = useAuthStore();

  useEffect(() => {
    if (isAuthenticated && !user) {
      checkAuth();
    }
  }, [isAuthenticated, user, checkAuth]);

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return <>{children}</>;
}

function DashboardPage() {
  const { user } = useAuthStore();
  const needsDisplayName = user && !user.display_name;

  return (
    <>
      <UsernamePromptModal isOpen={needsDisplayName ?? false} />
      <div className="p-8">
        <h1 className="text-2xl font-bold text-foreground">Dashboard</h1>
        <p className="mt-2 text-foreground-muted">
          Welcome{user?.display_name ? `, ${user.display_name}` : ''} to Altenia
        </p>
      </div>
    </>
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
      {
        path: 'settings/organization',
        element: <OrganizationPage />,
      },
      {
        path: 'projects/:projectId/logs',
        element: <ProjectLogsPage />,
      },
      {
        path: 'projects/:projectId/metrics',
        element: <ProjectMetricsPage />,
      },
      {
        path: 'projects/:projectId/traces',
        element: <ProjectTracesPage />,
      },
      {
        path: 'projects/:projectId/alerts',
        element: <ProjectAlertsPage />,
      },
      {
        path: 'projects/:projectId/settings',
        element: <ProjectSettingsPage />,
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
]);
