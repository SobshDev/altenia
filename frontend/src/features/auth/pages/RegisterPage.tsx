import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Button, Input } from '@/shared/components';
import { useAuthStore } from '@/stores/authStore';
import { useDeviceFingerprint } from '@/shared/hooks/useDeviceFingerprint';
import { registerSchema, type RegisterFormValues } from '../schemas/authSchemas';
import { AuthLayout } from '../components/AuthLayout';

function RegisterHeroContent() {
  return (
    <div className="max-w-md">
      <h1 className="text-4xl xl:text-5xl font-bold leading-tight mb-6">
        Start your
        <br />
        <span className="text-white/80">observability journey.</span>
      </h1>
      <p className="text-lg text-white/70 leading-relaxed">
        Create your free account and get instant access to powerful observability tools.
        No credit card required.
      </p>

      <div className="mt-10 space-y-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
            </svg>
          </div>
          <span className="text-white/80">Unlimited log ingestion</span>
        </div>
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
            </svg>
          </div>
          <span className="text-white/80">Real-time metrics & traces</span>
        </div>
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
            </svg>
          </div>
          <span className="text-white/80">OpenTelemetry native</span>
        </div>
      </div>
    </div>
  );
}

function RegisterForm() {
  const fingerprint = useDeviceFingerprint();
  const { register: registerUser, isLoading, error, clearError } = useAuthStore();

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<RegisterFormValues>({
    resolver: zodResolver(registerSchema),
  });

  const navigate = useNavigate();

  const onSubmit = async (data: RegisterFormValues) => {
    try {
      await registerUser(data.email, data.password, fingerprint);
      navigate('/');
    } catch {
      // Error is handled by the store
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
      {error && (
        <div
          className="p-4 bg-destructive-muted border border-destructive-border rounded-xl flex items-start gap-3"
          role="alert"
        >
          <svg className="w-5 h-5 text-destructive flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
          </svg>
          <div className="flex-1">
            <p className="text-sm font-medium text-destructive-muted-foreground">{error}</p>
            <button
              type="button"
              onClick={clearError}
              className="text-xs text-destructive hover:underline mt-1"
            >
              Dismiss
            </button>
          </div>
        </div>
      )}

      <Input
        label="Email address"
        type="email"
        autoComplete="email"
        placeholder="you@example.com"
        error={errors.email?.message}
        {...register('email')}
      />

      <Input
        label="Password"
        type="password"
        autoComplete="new-password"
        placeholder="Create a strong password"
        error={errors.password?.message}
        {...register('password')}
      />

      <Input
        label="Confirm password"
        type="password"
        autoComplete="new-password"
        placeholder="Confirm your password"
        error={errors.confirmPassword?.message}
        {...register('confirmPassword')}
      />

      <Button type="submit" className="w-full" size="lg" isLoading={isLoading}>
        Create account
      </Button>
    </form>
  );
}

export function RegisterPage() {
  const navigate = useNavigate();
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  useEffect(() => {
    if (isAuthenticated) {
      navigate('/');
    }
  }, [isAuthenticated, navigate]);

  return (
    <AuthLayout
      title="Create your account"
      subtitle="Get started with Altenia for free"
      alternateAction={{
        text: "Already have an account?",
        linkText: "Sign in",
        href: "/login",
      }}
      heroContent={<RegisterHeroContent />}
    >
      <RegisterForm />
    </AuthLayout>
  );
}
