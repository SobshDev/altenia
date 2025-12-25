import { useState } from 'react';
import { Key, Loader2, Clock, X } from 'lucide-react';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useProjectStore } from '@/stores/projectStore';

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function formatExpiry(expiresAt?: string): string {
  if (!expiresAt) return 'Never expires';
  const date = new Date(expiresAt);
  const now = new Date();
  if (date < now) return 'Expired';
  return `Expires ${formatDate(expiresAt)}`;
}

export function ApiKeyList() {
  const { currentProject, apiKeys, revokeApiKey } = useProjectStore();
  const [confirmingId, setConfirmingId] = useState<string | null>(null);
  const [revokingId, setRevokingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleRevoke = async (keyId: string, keyName: string) => {
    if (!currentProject) return;

    setRevokingId(keyId);
    setError(null);
    try {
      await revokeApiKey(currentProject.id, keyId);
      setSuccess(`API key "${keyName}" has been revoked`);
      setConfirmingId(null);
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke API key');
    } finally {
      setRevokingId(null);
    }
  };

  const activeKeys = apiKeys.filter((key) => key.is_active);

  return (
    <div className="space-y-3">
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}
      {success && <SuccessAlert message={success} onDismiss={() => setSuccess(null)} />}

      {activeKeys.map((apiKey, index) => {
        const isExpired = apiKey.expires_at && new Date(apiKey.expires_at) < new Date();
        const isConfirming = confirmingId === apiKey.id;
        const isRevoking = revokingId === apiKey.id;

        return (
          <div
            key={apiKey.id}
            className="flex items-center justify-between p-4 rounded-xl bg-surface-alt animate-list-item"
            style={{ '--item-index': index } as React.CSSProperties}
          >
            <div className="flex items-center gap-3 min-w-0">
              <div className="p-2 rounded-lg bg-primary/10">
                <Key className="w-4 h-4 text-primary" />
              </div>
              <div className="min-w-0">
                <p className="text-sm font-medium text-foreground truncate">
                  {apiKey.name}
                </p>
                <div className="flex items-center gap-3 text-xs text-foreground-muted">
                  <code className="font-mono bg-surface px-1.5 py-0.5 rounded">
                    {apiKey.key_prefix}...
                  </code>
                  <span className="flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    {isExpired ? (
                      <span className="text-destructive">Expired</span>
                    ) : (
                      formatExpiry(apiKey.expires_at)
                    )}
                  </span>
                </div>
              </div>
            </div>

            {isRevoking ? (
              <Loader2 className="w-4 h-4 animate-spin text-foreground-muted" />
            ) : isConfirming ? (
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setConfirmingId(null)}
                  className="px-3 py-1.5 text-xs font-medium rounded-lg text-foreground-muted hover:bg-surface transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={() => handleRevoke(apiKey.id, apiKey.name)}
                  className="px-3 py-1.5 text-xs font-medium rounded-lg bg-destructive text-destructive-foreground hover:bg-destructive-hover transition-colors"
                >
                  Confirm
                </button>
              </div>
            ) : (
              <button
                onClick={() => setConfirmingId(apiKey.id)}
                className="p-2 rounded-lg text-foreground-muted hover:text-destructive hover:bg-destructive/10 transition-colors"
                title="Revoke API key"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}
