import { useState } from 'react';
import { Plus, Loader2, Key } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { useProjectStore } from '@/stores/projectStore';
import { ApiKeyList } from './ApiKeyList';
import { CreateApiKeyModal } from './CreateApiKeyModal';
import { ApiKeyCreatedModal } from './ApiKeyCreatedModal';
import type { CreateApiKeyResponse } from '@/shared/types/api';

export function ApiKeySection() {
  const { apiKeys, apiKeysLoading, apiKeysError, clearApiKeysError } = useProjectStore();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createdKey, setCreatedKey] = useState<CreateApiKeyResponse | null>(null);

  const handleKeyCreated = (response: CreateApiKeyResponse) => {
    setShowCreateModal(false);
    setCreatedKey(response);
  };

  if (apiKeysLoading && apiKeys.length === 0) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="w-6 h-6 animate-spin text-foreground-muted" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {apiKeysError && (
        <ErrorAlert message={apiKeysError} onDismiss={clearApiKeysError} />
      )}

      {apiKeys.length === 0 ? (
        <div className="text-center py-8">
          <Key className="w-12 h-12 text-foreground-muted mx-auto mb-3 opacity-50" />
          <p className="text-foreground-muted mb-4">No API keys yet</p>
          <Button onClick={() => setShowCreateModal(true)} className="gap-2">
            <Plus className="w-4 h-4" />
            Create API key
          </Button>
        </div>
      ) : (
        <>
          <ApiKeyList />
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowCreateModal(true)}
            className="gap-2"
          >
            <Plus className="w-4 h-4" />
            Create API key
          </Button>
        </>
      )}

      <CreateApiKeyModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onCreated={handleKeyCreated}
      />

      <ApiKeyCreatedModal
        apiKey={createdKey}
        onClose={() => setCreatedKey(null)}
      />
    </div>
  );
}
