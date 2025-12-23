import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Button } from '@/shared/components/Button';
import { PasswordInput } from '@/shared/components/PasswordInput';
import { PasswordStrength } from '@/shared/components/PasswordStrength';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useAuthStore } from '@/stores/authStore';
import {
  changePasswordSchema,
  type ChangePasswordFormValues,
} from '../schemas/settingsSchemas';

export function ChangePasswordForm() {
  const { updatePassword } = useAuthStore();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [showStrength, setShowStrength] = useState(false);

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors },
  } = useForm<ChangePasswordFormValues>({
    resolver: zodResolver(changePasswordSchema),
  });

  const newPassword = watch('newPassword', '');

  const onSubmit = async (data: ChangePasswordFormValues) => {
    setIsLoading(true);
    setError(null);
    try {
      await updatePassword(data.currentPassword, data.newPassword);
      setSuccess(true);
      setShowStrength(false);
      reset();
      setTimeout(() => setSuccess(false), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update password');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}
      {success && (
        <SuccessAlert
          message="Password updated successfully"
          onDismiss={() => setSuccess(false)}
        />
      )}

      <PasswordInput
        label="Current password"
        placeholder="Enter your current password"
        error={errors.currentPassword?.message}
        {...register('currentPassword')}
      />

      <div className="space-y-2">
        <PasswordInput
          label="New password"
          placeholder="Enter your new password"
          error={errors.newPassword?.message}
          onFocus={() => setShowStrength(true)}
          {...register('newPassword')}
        />
        <PasswordStrength password={newPassword} show={showStrength} />
      </div>

      <PasswordInput
        label="Confirm new password"
        placeholder="Confirm your new password"
        error={errors.confirmNewPassword?.message}
        {...register('confirmNewPassword')}
      />

      <div className="pt-2">
        <Button type="submit" isLoading={isLoading}>
          Update password
        </Button>
      </div>
    </form>
  );
}
