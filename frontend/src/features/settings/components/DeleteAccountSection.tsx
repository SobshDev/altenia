import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { Trash2, AlertTriangle, X, Loader2 } from 'lucide-react';
import { useNavigate } from 'react-router';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { PasswordInput } from '@/shared/components/PasswordInput';
import { useAuthStore } from '@/stores/authStore';

const CONFIRMATION_PHRASE = 'delete my account';

export function DeleteAccountSection() {
  const [isOpen, setIsOpen] = useState(false);
  const [confirmText, setConfirmText] = useState('');
  const [password, setPassword] = useState('');
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { deleteAccount } = useAuthStore();
  const navigate = useNavigate();

  const isConfirmed = confirmText.toLowerCase() === CONFIRMATION_PHRASE && password.length > 0;

  const handleClose = () => {
    setIsOpen(false);
    setConfirmText('');
    setPassword('');
    setError(null);
  };

  const handleDelete = async () => {
    if (!isConfirmed) return;

    setIsDeleting(true);
    setError(null);

    try {
      await deleteAccount(password);
      handleClose();
      navigate('/login');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete account');
    } finally {
      setIsDeleting(false);
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

  const modalContent = (
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
        className="animate-modal-enter bg-surface border border-destructive/20 shadow-2xl"
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
          className="absolute top-4 right-4 p-2 rounded-lg text-foreground-subtle hover:text-foreground hover:bg-surface-alt transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Header */}
        <div className="flex items-start gap-4 mb-6">
          <div className="flex-shrink-0 p-3 rounded-xl bg-destructive/10">
            <AlertTriangle className="w-6 h-6 text-destructive" />
          </div>
          <div>
            <h2 className="text-xl font-semibold text-foreground">
              Delete your account?
            </h2>
            <p className="text-sm text-foreground-muted mt-1">
              This action is permanent and cannot be undone.
            </p>
          </div>
        </div>

        {/* Warning content */}
        <div className="space-y-4 mb-6">
          <div className="p-4 rounded-xl bg-destructive/5 border border-destructive/10">
            <p className="text-sm text-foreground">
              Deleting your account will:
            </p>
            <ul className="mt-2 space-y-1 text-sm text-foreground-muted">
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Permanently remove all your data after 30 days
              </li>
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Delete all projects and settings
              </li>
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Transfer ownership of shared organizations
              </li>
            </ul>
          </div>

          {error && (
            <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20">
              <p className="text-sm text-destructive">{error}</p>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              Type{' '}
              <span className="font-mono text-destructive bg-destructive/10 px-1.5 py-0.5 rounded">
                {CONFIRMATION_PHRASE}
              </span>{' '}
              to confirm
            </label>
            <Input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
              placeholder={CONFIRMATION_PHRASE}
              className="font-mono"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              Enter your password
            </label>
            <PasswordInput
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Your current password"
            />
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <Button
            variant="secondary"
            onClick={handleClose}
            disabled={isDeleting}
            className="flex-1"
          >
            Cancel
          </Button>
          <Button
            onClick={handleDelete}
            disabled={!isConfirmed || isDeleting}
            className={`flex-1 gap-2 ${
              isConfirmed && !isDeleting
                ? 'bg-destructive hover:bg-destructive-hover text-destructive-foreground'
                : 'bg-destructive/30 text-destructive-foreground/50 cursor-not-allowed'
            }`}
          >
            {isDeleting ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Trash2 className="w-4 h-4" />
            )}
            {isDeleting ? 'Deleting...' : 'Delete forever'}
          </Button>
        </div>
      </div>
    </div>
  );

  return (
    <>
      <div className="space-y-4">
        <div>
          <p className="text-sm text-foreground">Delete your account</p>
          <p className="text-xs text-foreground-muted mt-0.5">
            Permanently delete your account and all associated data. This action
            cannot be undone.
          </p>
        </div>

        <Button
          variant="ghost"
          onClick={() => setIsOpen(true)}
          className="w-full justify-center gap-2 text-destructive hover:bg-destructive/10 border border-destructive/20"
        >
          <Trash2 className="w-4 h-4" />
          Delete account
        </Button>
      </div>

      {/* Render modal in a portal at document body */}
      {isOpen && createPortal(modalContent, document.body)}
    </>
  );
}
