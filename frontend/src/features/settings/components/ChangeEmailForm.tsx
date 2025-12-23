import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Pencil } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { PasswordInput } from '@/shared/components/PasswordInput';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useAuthStore } from '@/stores/authStore';
import {
  changeEmailSchema,
  type ChangeEmailFormValues,
} from '../schemas/settingsSchemas';

export function ChangeEmailForm() {
  const { user, updateEmail } = useAuthStore();
  const [isEditing, setIsEditing] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<ChangeEmailFormValues>({
    resolver: zodResolver(changeEmailSchema),
  });

  const onSubmit = async (data: ChangeEmailFormValues) => {
    setIsLoading(true);
    setError(null);
    try {
      await updateEmail(data.newEmail, data.currentPassword);
      setSuccess(true);
      setIsEditing(false);
      reset();
      setTimeout(() => setSuccess(false), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update email');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = () => {
    setIsEditing(false);
    setError(null);
    reset();
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <label className="text-sm font-medium text-foreground-muted">
            Email address
          </label>
          <p className="text-foreground">{user?.email}</p>
        </div>
        {!isEditing && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsEditing(true)}
            className="gap-2"
          >
            <Pencil className="w-4 h-4" />
            Change
          </Button>
        )}
      </div>

      {success && (
        <SuccessAlert
          message="Email updated successfully"
          onDismiss={() => setSuccess(false)}
        />
      )}

      <div
        className={`overflow-hidden transition-all duration-300 ease-out ${
          isEditing ? 'opacity-100 max-h-96' : 'opacity-0 max-h-0'
        }`}
      >
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 pt-2">
          {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}

          <Input
            label="New email address"
            type="email"
            placeholder="Enter your new email"
            error={errors.newEmail?.message}
            {...register('newEmail')}
          />

          <PasswordInput
            label="Current password"
            placeholder="Enter your current password"
            error={errors.currentPassword?.message}
            {...register('currentPassword')}
          />

          <div className="flex gap-3 pt-2">
            <Button type="submit" isLoading={isLoading}>
              Update email
            </Button>
            <Button type="button" variant="ghost" onClick={handleCancel}>
              Cancel
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
