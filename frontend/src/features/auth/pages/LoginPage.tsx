import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useAuthStore } from '@/stores/authStore';
import { LoginForm } from '../components/LoginForm';
import { AuthLayout } from '../components/AuthLayout';

function LoginHeroContent() {
  return (
    <div className="max-w-md">
      <h1 className="text-4xl xl:text-5xl font-bold leading-tight mb-6">
        Observe everything.
        <br />
        <span className="text-white/80">Miss nothing.</span>
      </h1>
      <p className="text-lg text-white/70 leading-relaxed">
        Open-source observability platform for logs, metrics, and traces.
        Get deep insights into your applications with powerful querying and real-time monitoring.
      </p>

      <div className="mt-10 flex items-center gap-8">
        <div>
          <div className="text-3xl font-bold">100%</div>
          <div className="text-sm text-white/60">Open Source</div>
        </div>
        <div className="w-px h-12 bg-white/20" />
        <div>
          <div className="text-3xl font-bold">3-in-1</div>
          <div className="text-sm text-white/60">Logs, Metrics, Traces</div>
        </div>
        <div className="w-px h-12 bg-white/20" />
        <div>
          <div className="text-3xl font-bold">OTLP</div>
          <div className="text-sm text-white/60">Native Support</div>
        </div>
      </div>
    </div>
  );
}

export function LoginPage() {
  const navigate = useNavigate();
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  useEffect(() => {
    if (isAuthenticated) {
      navigate('/');
    }
  }, [isAuthenticated, navigate]);

  return (
    <AuthLayout
      title="Welcome back"
      subtitle="Sign in to your account to continue"
      alternateAction={{
        text: "Don't have an account?",
        linkText: "Create one now",
        href: "/register",
      }}
      heroContent={<LoginHeroContent />}
    >
      <LoginForm />
    </AuthLayout>
  );
}
