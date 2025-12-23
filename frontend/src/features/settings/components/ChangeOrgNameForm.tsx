import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Pencil } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useOrgStore } from '@/stores/orgStore';
import {
  changeOrgNameSchema,
  type ChangeOrgNameFormValues,
} from '../schemas/settingsSchemas';

export function ChangeOrgNameForm() {
  const { currentOrg, updateOrg } = useOrgStore();
  const [isEditing, setIsEditing] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<ChangeOrgNameFormValues>({
    resolver: zodResolver(changeOrgNameSchema),
  });

  const onSubmit = async (data: ChangeOrgNameFormValues) => {
    setIsLoading(true);
    setError(null);
    try {
      await updateOrg(data.name);
      setSuccess(true);
      setIsEditing(false);
      reset();
      setTimeout(() => setSuccess(false), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update organization name');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = () => {
    setIsEditing(false);
    setError(null);
    reset();
  };

  const displayName = currentOrg?.is_personal ? 'Personal' : currentOrg?.name;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <label className="text-sm font-medium text-foreground-muted">
            Organization name
          </label>
          <p className="text-foreground">
            {displayName || (
              <span className="text-foreground-subtle">Loading...</span>
            )}
          </p>
        </div>
        {currentOrg && !currentOrg.is_personal && currentOrg.role === 'owner' && (
          <div className={`transition-all duration-200 ${isEditing ? 'opacity-0 scale-95 pointer-events-none' : 'opacity-100 scale-100'}`}>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsEditing(true)}
              className="gap-2"
            >
              <Pencil className="w-4 h-4" />
              Change
            </Button>
          </div>
        )}
      </div>

      {currentOrg?.is_personal && (
        <p className="text-sm text-foreground-muted">
          Personal organization names cannot be changed.
        </p>
      )}

      {success && (
        <SuccessAlert
          message="Organization name updated successfully"
          onDismiss={() => setSuccess(false)}
        />
      )}

      <div
        className={`grid transition-all duration-300 ease-out ${
          isEditing ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'
        }`}
      >
        <div className="overflow-hidden">
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 pt-2">
          {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}

          <Input
            label="New organization name"
            type="text"
            placeholder="Enter new name"
            error={errors.name?.message}
            {...register('name')}
          />

          <div className="flex gap-3 pt-2">
            <Button type="submit" isLoading={isLoading}>
              Update name
            </Button>
            <Button type="button" variant="ghost" onClick={handleCancel}>
              Cancel
            </Button>
          </div>
        </form>
        </div>
      </div>
    </div>
  );
}
