import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Key, Loader2, X } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { useProjectStore } from '@/stores/projectStore';
import {
  createApiKeySchema,
  type CreateApiKeyFormValues,
} from '../schemas/projectSchemas';
import type { CreateApiKeyResponse } from '@/shared/types/api';

interface CreateApiKeyModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreated: (response: CreateApiKeyResponse) => void;
}

const expirationOptions = [
  { label: 'Never', value: undefined },
  { label: '30 days', value: 30 },
  { label: '90 days', value: 90 },
  { label: '1 year', value: 365 },
];

export function CreateApiKeyModal({
  isOpen,
  onClose,
  onCreated,
}: CreateApiKeyModalProps) {
  const { currentProject, createApiKey } = useProjectStore();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expiration, setExpiration] = useState<number | undefined>(undefined);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateApiKeyFormValues>({
    resolver: zodResolver(createApiKeySchema),
    defaultValues: {
      name: '',
    },
  });

  const handleClose = () => {
    reset();
    setError(null);
    setExpiration(undefined);
    onClose();
  };

  const onSubmit = async (data: CreateApiKeyFormValues) => {
    if (!currentProject) return;

    setIsLoading(true);
    setError(null);
    try {
      const response = await createApiKey(currentProject.id, {
        name: data.name,
        expires_in_days: expiration,
      });
      handleClose();
      onCreated(response);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create API key');
    } finally {
      setIsLoading(false);
    }
  };

  // Prevent body scroll when modal is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  // Close on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') handleClose();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleEscape);
    }
    return () => window.removeEventListener('keydown', handleEscape);
  }, [isOpen]);

  if (!isOpen) return null;

  const modal = (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: '100vw',
        height: '100vh',
        zIndex: 99999,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      {/* Backdrop */}
      <div
        onClick={handleClose}
        className="animate-fade-in"
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: '100%',
          backgroundColor: 'rgba(0, 0, 0, 0.7)',
          backdropFilter: 'blur(4px)',
        }}
      />

      {/* Modal */}
      <div
        className="animate-modal-enter bg-surface border border-border shadow-2xl"
        style={{
          position: 'relative',
          width: '100%',
          maxWidth: '28rem',
          margin: '1rem',
          padding: '1.5rem',
          borderRadius: '1rem',
        }}
      >
        {/* Close button */}
        <button
          onClick={handleClose}
          className="absolute top-4 right-4 p-2 rounded-lg text-foreground-muted hover:text-foreground hover:bg-surface-alt transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Header */}
        <div className="flex items-start gap-4 mb-6">
          <div className="flex-shrink-0 p-3 rounded-xl bg-primary/10">
            <Key className="w-6 h-6 text-primary" />
          </div>
          <div>
            <h2 className="text-xl font-semibold text-foreground">
              Create API key
            </h2>
            <p className="text-sm text-foreground-muted mt-1">
              API keys are used to send data to this project.
            </p>
          </div>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          {error && (
            <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20">
              <p className="text-sm text-destructive">{error}</p>
            </div>
          )}

          <Input
            label="Key name"
            placeholder="Production API Key"
            error={errors.name?.message}
            autoFocus
            {...register('name')}
          />

          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              Expiration
            </label>
            <div className="flex flex-wrap gap-2">
              {expirationOptions.map((option) => (
                <button
                  key={option.label}
                  type="button"
                  onClick={() => setExpiration(option.value)}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                    expiration === option.value
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-surface-alt text-foreground-muted hover:text-foreground'
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-3 pt-2">
            <Button
              type="button"
              variant="secondary"
              onClick={handleClose}
              disabled={isLoading}
              className="flex-1"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={isLoading}
              className="flex-1 gap-2"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Key className="w-4 h-4" />
              )}
              Create key
            </Button>
          </div>
        </form>
      </div>
    </div>
  );

  return createPortal(modal, document.body);
}
