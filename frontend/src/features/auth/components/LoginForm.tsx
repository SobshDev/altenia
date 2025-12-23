import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from 'react-router';
import { Button, Input } from '@/shared/components';
import { useAuthStore } from '@/stores/authStore';
import { useDeviceFingerprint } from '@/shared/hooks/useDeviceFingerprint';
import { loginSchema, type LoginFormValues } from '../schemas/authSchemas';

export function LoginForm() {
  const navigate = useNavigate();
  const fingerprint = useDeviceFingerprint();
  const { login, isLoading, error, clearError } = useAuthStore();

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormValues>({
    resolver: zodResolver(loginSchema),
  });

  const onSubmit = async (data: LoginFormValues) => {
    try {
      await login(data.email, data.password, fingerprint);
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

      <div>
        <Input
          label="Password"
          type="password"
          autoComplete="current-password"
          placeholder="Enter your password"
          error={errors.password?.message}
          {...register('password')}
        />
        <div className="mt-2 text-right">
          <button
            type="button"
            className="text-sm text-primary hover:text-primary-hover font-medium transition-colors"
          >
            Forgot password?
          </button>
        </div>
      </div>

      <Button type="submit" className="w-full" size="lg" isLoading={isLoading}>
        Sign in
      </Button>
    </form>
  );
}
