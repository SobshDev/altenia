import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2, ScrollText, LineChart, Route } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useProjectStore } from '@/stores/projectStore';
import {
  retentionSchema,
  type RetentionFormValues,
} from '../schemas/projectSchemas';

export function ProjectRetentionForm() {
  const { currentProject, updateProject } = useProjectStore();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<RetentionFormValues>({
    resolver: zodResolver(retentionSchema),
    defaultValues: {
      retention_days: currentProject?.retention_days || 30,
      metrics_retention_days: currentProject?.metrics_retention_days || 90,
      traces_retention_days: currentProject?.traces_retention_days || 14,
    },
  });

  // Reset form when project changes
  useEffect(() => {
    if (currentProject) {
      reset({
        retention_days: currentProject.retention_days,
        metrics_retention_days: currentProject.metrics_retention_days,
        traces_retention_days: currentProject.traces_retention_days,
      });
    }
  }, [currentProject, reset]);

  const onSubmit = async (data: RetentionFormValues) => {
    if (!currentProject) return;

    setIsLoading(true);
    setError(null);
    try {
      await updateProject(currentProject.id, data);
      setSuccess('Retention settings updated');
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update retention');
    } finally {
      setIsLoading(false);
    }
  };

  const retentionFields = [
    {
      name: 'retention_days' as const,
      label: 'Logs',
      icon: ScrollText,
      max: 365,
      description: 'How long to keep log data',
    },
    {
      name: 'metrics_retention_days' as const,
      label: 'Metrics',
      icon: LineChart,
      max: 365,
      description: 'How long to keep metrics data',
    },
    {
      name: 'traces_retention_days' as const,
      label: 'Traces',
      icon: Route,
      max: 90,
      description: 'How long to keep trace data',
    },
  ];

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}
      {success && <SuccessAlert message={success} onDismiss={() => setSuccess(null)} />}

      <div className="space-y-4">
        {retentionFields.map((field) => {
          const Icon = field.icon;
          return (
            <div
              key={field.name}
              className="flex items-center gap-4 p-4 rounded-xl bg-surface-alt"
            >
              <div className="p-2 rounded-lg bg-primary/10">
                <Icon className="w-4 h-4 text-primary" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-foreground">
                  {field.label}
                </p>
                <p className="text-xs text-foreground-muted">
                  {field.description}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min={1}
                  max={field.max}
                  className="w-20 px-3 py-2 rounded-lg bg-surface border border-border text-foreground text-sm text-center focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
                  {...register(field.name, { valueAsNumber: true })}
                />
                <span className="text-sm text-foreground-muted">days</span>
              </div>
            </div>
          );
        })}
      </div>

      {errors.retention_days && (
        <p className="text-sm text-destructive">{errors.retention_days.message}</p>
      )}
      {errors.metrics_retention_days && (
        <p className="text-sm text-destructive">{errors.metrics_retention_days.message}</p>
      )}
      {errors.traces_retention_days && (
        <p className="text-sm text-destructive">{errors.traces_retention_days.message}</p>
      )}

      <Button type="submit" disabled={isLoading || !isDirty}>
        {isLoading && <Loader2 className="w-4 h-4 animate-spin mr-2" />}
        Save changes
      </Button>
    </form>
  );
}
