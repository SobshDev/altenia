import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { Key, Copy, Check, AlertTriangle } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import type { CreateApiKeyResponse } from '@/shared/types/api';

interface ApiKeyCreatedModalProps {
  apiKey: CreateApiKeyResponse | null;
  onClose: () => void;
}

export function ApiKeyCreatedModal({ apiKey, onClose }: ApiKeyCreatedModalProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!apiKey) return;
    try {
      await navigator.clipboard.writeText(apiKey.plain_key);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  // Prevent body scroll when modal is open
  useEffect(() => {
    if (apiKey) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [apiKey]);

  // Close on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (apiKey) {
      window.addEventListener('keydown', handleEscape);
    }
    return () => window.removeEventListener('keydown', handleEscape);
  }, [apiKey, onClose]);

  if (!apiKey) return null;

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
      {/* Backdrop - no click to close for this modal */}
      <div
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
          maxWidth: '32rem',
          margin: '1rem',
          padding: '1.5rem',
          borderRadius: '1rem',
        }}
      >
        {/* Header */}
        <div className="flex items-start gap-4 mb-6">
          <div className="flex-shrink-0 p-3 rounded-xl bg-green-500/10">
            <Key className="w-6 h-6 text-green-500" />
          </div>
          <div>
            <h2 className="text-xl font-semibold text-foreground">
              API key created
            </h2>
            <p className="text-sm text-foreground-muted mt-1">
              Your new API key has been created successfully.
            </p>
          </div>
        </div>

        {/* Warning */}
        <div className="flex items-start gap-3 p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 mb-6">
          <AlertTriangle className="w-5 h-5 text-amber-500 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-sm font-medium text-foreground">
              Copy your API key now
            </p>
            <p className="text-sm text-foreground-muted mt-1">
              This is the only time you will be able to see this key. Store it securely.
            </p>
          </div>
        </div>

        {/* API Key display */}
        <div className="space-y-3 mb-6">
          <div>
            <label className="block text-sm font-medium text-foreground-muted mb-2">
              Key name
            </label>
            <p className="text-sm text-foreground">{apiKey.name}</p>
          </div>

          <div>
            <label className="block text-sm font-medium text-foreground-muted mb-2">
              API key
            </label>
            <div className="flex items-center gap-2">
              <code className="flex-1 p-3 rounded-lg bg-surface-alt border border-border font-mono text-sm text-foreground break-all">
                {apiKey.plain_key}
              </code>
              <Button
                variant="secondary"
                onClick={handleCopy}
                className="shrink-0 gap-2"
              >
                {copied ? (
                  <>
                    <Check className="w-4 h-4 text-green-500" />
                    Copied
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4" />
                    Copy
                  </>
                )}
              </Button>
            </div>
          </div>
        </div>

        {/* Action */}
        <Button onClick={onClose} className="w-full">
          Done
        </Button>
      </div>
    </div>
  );

  return createPortal(modal, document.body);
}
