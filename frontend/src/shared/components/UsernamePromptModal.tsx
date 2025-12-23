import { useState } from 'react';
import { User } from 'lucide-react';
import { Button } from './Button';
import { Input } from './Input';
import { useAuthStore } from '@/stores/authStore';

interface UsernamePromptModalProps {
  isOpen: boolean;
}

export function UsernamePromptModal({ isOpen }: UsernamePromptModalProps) {
  const [displayName, setDisplayName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const { updateDisplayName } = useAuthStore();

  if (!isOpen) return null;

  const validateDisplayName = (name: string): string | null => {
    const trimmed = name.trim();
    if (trimmed.length < 1) {
      return 'Display name is required';
    }
    if (trimmed.length > 30) {
      return 'Display name must not exceed 30 characters';
    }
    // Check for allowed characters (letters, spaces, dashes, apostrophes)
    const validPattern = /^[\p{L}\s\-']+$/u;
    if (!validPattern.test(trimmed)) {
      return 'Display name can only contain letters, spaces, dashes, and apostrophes';
    }
    return null;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const validationError = validateDisplayName(displayName);
    if (validationError) {
      setError(validationError);
      return;
    }

    setIsLoading(true);
    try {
      await updateDisplayName(displayName.trim());
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to set display name');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
      <div className="w-full max-w-md mx-4">
        <div className="bg-surface border border-border rounded-2xl p-8 shadow-2xl">
          <div className="flex flex-col items-center text-center mb-6">
            <div className="p-4 rounded-full bg-primary/10 mb-4">
              <User className="w-8 h-8 text-primary" />
            </div>
            <h2 className="text-2xl font-bold text-foreground">Welcome!</h2>
            <p className="text-foreground-muted mt-2">
              Please set your display name to continue. This is how others will see you.
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-6">
            <Input
              label="Display Name"
              placeholder="e.g., John Doe"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              error={error || undefined}
              autoFocus
              maxLength={30}
            />

            <p className="text-xs text-foreground-subtle">
              1-30 characters. Letters, spaces, dashes, and apostrophes allowed.
            </p>

            <Button type="submit" className="w-full" size="lg" isLoading={isLoading}>
              {isLoading ? 'Saving...' : 'Continue'}
            </Button>
          </form>
        </div>
      </div>
    </div>
  );
}
