import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Pencil, Loader2 } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useProjectStore } from '@/stores/projectStore';
import {
  updateProjectSchema,
  type UpdateProjectFormValues,
} from '../schemas/projectSchemas';

export function ChangeProjectNameForm() {
  const { currentProject, updateProject } = useProjectStore();
  const [isEditing, setIsEditing] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<UpdateProjectFormValues>({
    resolver: zodResolver(updateProjectSchema),
    defaultValues: {
      name: currentProject?.name || '',
      description: currentProject?.description || '',
    },
  });

  const onSubmit = async (data: UpdateProjectFormValues) => {
    if (!currentProject) return;

    setIsLoading(true);
    setError(null);
    try {
      await updateProject(currentProject.id, {
        name: data.name,
        description: data.description || undefined,
      });
      setSuccess('Project updated successfully');
      setIsEditing(false);
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update project');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = () => {
    reset({
      name: currentProject?.name || '',
      description: currentProject?.description || '',
    });
    setIsEditing(false);
    setError(null);
  };

  if (!isEditing) {
    return (
      <div className="space-y-4">
        {success && <SuccessAlert message={success} onDismiss={() => setSuccess(null)} />}
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <p className="text-sm font-medium text-foreground">
              {currentProject?.name}
            </p>
            {currentProject?.description && (
              <p className="text-sm text-foreground-muted mt-1">
                {currentProject.description}
              </p>
            )}
          </div>
          <Button variant="ghost" size="sm" onClick={() => setIsEditing(true)} className="gap-2">
            <Pencil className="w-4 h-4" />
            Edit
          </Button>
        </div>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}

      <Input
        label="Project name"
        error={errors.name?.message}
        {...register('name')}
      />

      <div>
        <label className="block text-sm font-medium text-foreground mb-2">
          Description{' '}
          <span className="text-foreground-muted font-normal">(optional)</span>
        </label>
        <textarea
          placeholder="Brief description of this project..."
          className="w-full px-4 py-3 rounded-xl bg-surface-alt border border-border text-foreground placeholder:text-foreground-muted focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent resize-none"
          rows={3}
          {...register('description')}
        />
        {errors.description?.message && (
          <p className="mt-1 text-sm text-destructive">
            {errors.description.message}
          </p>
        )}
      </div>

      <div className="flex gap-3">
        <Button type="submit" disabled={isLoading}>
          {isLoading && <Loader2 className="w-4 h-4 animate-spin mr-2" />}
          Save changes
        </Button>
        <Button type="button" variant="ghost" onClick={handleCancel}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
