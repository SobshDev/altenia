import { useEffect, useState } from 'react';
import { Shield, Loader2 } from 'lucide-react';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SectionCard } from './SectionCard';
import { useInviteStore } from '@/stores/inviteStore';

export function PrivacySettings() {
  const { settings, settingsLoading, fetchSettings, updateSettings } = useInviteStore();
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  const handleToggle = async () => {
    if (!settings) return;

    setIsUpdating(true);
    setError(null);
    try {
      await updateSettings({ allow_invites: !settings.allow_invites });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update settings');
    } finally {
      setIsUpdating(false);
    }
  };

  if (settingsLoading && !settings) {
    return (
      <SectionCard
        icon={Shield}
        title="Privacy"
        description="Control who can send you invites"
        staggerDelay={150}
      >
        <div className="flex items-center justify-center py-4">
          <Loader2 className="w-5 h-5 animate-spin text-foreground-muted" />
        </div>
      </SectionCard>
    );
  }

  return (
    <SectionCard
      icon={Shield}
      title="Privacy"
      description="Control who can send you invites"
      staggerDelay={150}
    >
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-foreground">Allow organization invites</p>
          <p className="text-sm text-foreground-muted">
            When disabled, others cannot send you invites to join their organizations
          </p>
        </div>

        <button
          type="button"
          role="switch"
          aria-checked={settings?.allow_invites ?? true}
          disabled={isUpdating}
          onClick={handleToggle}
          className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 ${
            settings?.allow_invites ? 'bg-primary' : 'bg-surface-alt'
          }`}
        >
          <span
            aria-hidden="true"
            className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out ${
              settings?.allow_invites ? 'translate-x-5' : 'translate-x-0'
            }`}
          >
            {isUpdating && (
              <Loader2 className="w-3 h-3 absolute top-1 left-1 animate-spin text-foreground-muted" />
            )}
          </span>
        </button>
      </div>
    </SectionCard>
  );
}
