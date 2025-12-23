import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Button, Input, PasswordInput, PasswordStrength, ErrorAlert } from '@/shared/components';
import { useAuthStore } from '@/stores/authStore';
import { useDeviceFingerprint } from '@/shared/hooks/useDeviceFingerprint';
import { registerSchema, type RegisterFormValues } from '../schemas/authSchemas';
import { AuthLayout } from '../components/AuthLayout';

function RegisterHeroContent() {
  return (
    <div className="max-w-md">
      <h1 className="text-4xl xl:text-5xl font-bold leading-tight mb-6 overflow-hidden">
        <span
          className="block animate-text-reveal"
          style={{ '--stagger': '0ms' } as React.CSSProperties}
        >
          Start your
        </span>
        <span
          className="block text-white/80 animate-text-reveal"
          style={{ '--stagger': '150ms' } as React.CSSProperties}
        >
          observability journey.
        </span>
      </h1>
      <p
        className="text-lg text-white/70 leading-relaxed animate-fade-in-up"
        style={{ '--stagger': '350ms' } as React.CSSProperties}
      >
        Create your free account and get instant access to powerful observability tools.
        No credit card required.
      </p>

      <div className="mt-10 space-y-4">
        <div
          className="flex items-center gap-3 animate-slide-in-right"
          style={{ '--stagger': '500ms' } as React.CSSProperties}
        >
          <div className="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
            </svg>
          </div>
          <span className="text-white/80">Unlimited log ingestion</span>
        </div>
        <div
          className="flex items-center gap-3 animate-slide-in-right"
          style={{ '--stagger': '600ms' } as React.CSSProperties}
        >
          <div className="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
            </svg>
          </div>
          <span className="text-white/80">Real-time metrics & traces</span>
        </div>
        <div
          className="flex items-center gap-3 animate-slide-in-right"
          style={{ '--stagger': '700ms' } as React.CSSProperties}
        >
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
  const [passwordValue, setPasswordValue] = useState('');
  const [isPasswordFocused, setIsPasswordFocused] = useState(false);

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<RegisterFormValues>({
    resolver: zodResolver(registerSchema),
  });

  const navigate = useNavigate();

  // Watch password field for strength indicator
  const watchedPassword = watch('password', '');
  useEffect(() => {
    setPasswordValue(watchedPassword || '');
  }, [watchedPassword]);

  const onSubmit = async (data: RegisterFormValues) => {
    try {
      await registerUser(data.email, data.password, fingerprint);
      navigate('/');
    } catch {
      // Error is handled by the store
    }
  };

  // Get the register props and add focus handlers
  const passwordRegister = register('password');

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
      {error && <ErrorAlert message={error} onDismiss={clearError} />}

      <Input
        label="Email address"
        type="email"
        autoComplete="email"
        placeholder="name@company.com"
        error={errors.email?.message}
        {...register('email')}
      />

      <div>
        <PasswordInput
          label="Password"
          autoComplete="new-password"
          placeholder="Create a password"
          error={errors.password?.message}
          {...passwordRegister}
          onFocus={() => {
            setIsPasswordFocused(true);
          }}
          onBlur={(e) => {
            setIsPasswordFocused(false);
            passwordRegister.onBlur(e);
          }}
        />
        <PasswordStrength
          password={passwordValue}
          show={isPasswordFocused && passwordValue.length > 0}
        />
      </div>

      <PasswordInput
        label="Confirm password"
        autoComplete="new-password"
        placeholder="Re-enter your password"
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
