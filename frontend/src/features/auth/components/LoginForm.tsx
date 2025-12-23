import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from 'react-router';
import { Button, Input, PasswordInput, ErrorAlert } from '@/shared/components';
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
