import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Pencil } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useAuthStore } from '@/stores/authStore';
import {
  changeDisplayNameSchema,
  type ChangeDisplayNameFormValues,
} from '../schemas/settingsSchemas';

export function ChangeDisplayNameForm() {
  const { user, updateDisplayName } = useAuthStore();
  const [isEditing, setIsEditing] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<ChangeDisplayNameFormValues>({
    resolver: zodResolver(changeDisplayNameSchema),
    defaultValues: {
      displayName: user?.display_name || '',
    },
  });

  const onSubmit = async (data: ChangeDisplayNameFormValues) => {
    setIsLoading(true);
    setError(null);
    try {
      await updateDisplayName(data.displayName);
      setSuccess(true);
      setIsEditing(false);
      setTimeout(() => setSuccess(false), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update display name');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = () => {
    setIsEditing(false);
    setError(null);
    reset({ displayName: user?.display_name || '' });
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <label className="text-sm font-medium text-foreground-muted">
            Display name
          </label>
          <p className="text-foreground">
            {user?.display_name || (
              <span className="text-foreground-subtle italic">Not set</span>
            )}
          </p>
        </div>
        {!isEditing && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsEditing(true)}
            className="gap-2"
          >
            <Pencil className="w-4 h-4" />
            {user?.display_name ? 'Change' : 'Set'}
          </Button>
        )}
      </div>

      {success && (
        <SuccessAlert
          message="Display name updated successfully"
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
            label="Display name"
            placeholder="e.g., John Doe"
            error={errors.displayName?.message}
            maxLength={30}
            {...register('displayName')}
          />

          <p className="text-xs text-foreground-subtle">
            1-30 characters. Letters, spaces, dashes, and apostrophes allowed.
          </p>

          <div className="flex gap-3 pt-2">
            <Button type="submit" isLoading={isLoading}>
              Update display name
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
