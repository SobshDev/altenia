import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { LogOut, Trash2, Loader2, AlertTriangle, X } from 'lucide-react';
import { useNavigate } from 'react-router';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { useOrgStore } from '@/stores/orgStore';

export function LeaveOrgSection() {
  const { currentOrg, leaveOrg, deleteOrg } = useOrgStore();
  const navigate = useNavigate();
  const [isLeaving, setIsLeaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [confirmText, setConfirmText] = useState('');
  const [modalError, setModalError] = useState<string | null>(null);

  const isOwner = currentOrg?.role === 'owner';
  const isPersonal = currentOrg?.is_personal;
  const orgName = currentOrg?.name || '';
  const isConfirmed = confirmText.toLowerCase() === orgName.toLowerCase();

  const handleLeave = async () => {
    if (!confirm('Are you sure you want to leave this organization?')) return;

    setIsLeaving(true);
    setError(null);
    try {
      await leaveOrg();
      navigate('/settings/organization');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to leave organization');
    } finally {
      setIsLeaving(false);
    }
  };

  const handleCloseModal = () => {
    setIsDeleteModalOpen(false);
    setConfirmText('');
    setModalError(null);
  };

  const handleDelete = async () => {
    if (!isConfirmed) return;

    setIsDeleting(true);
    setModalError(null);
    try {
      await deleteOrg();
      handleCloseModal();
      navigate('/settings/organization');
    } catch (err) {
      setModalError(err instanceof Error ? err.message : 'Failed to delete organization');
    } finally {
      setIsDeleting(false);
    }
  };

  // Prevent body scroll when modal is open
  useEffect(() => {
    if (isDeleteModalOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isDeleteModalOpen]);

  // Close on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') handleCloseModal();
    };
    if (isDeleteModalOpen) {
      window.addEventListener('keydown', handleEscape);
    }
    return () => window.removeEventListener('keydown', handleEscape);
  }, [isDeleteModalOpen]);

  const deleteModal = (
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
        onClick={handleCloseModal}
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
          onClick={handleCloseModal}
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
              Delete organization?
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
              Deleting this organization will:
            </p>
            <ul className="mt-2 space-y-1 text-sm text-foreground-muted">
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Permanently remove all projects and data
              </li>
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Remove all members from the organization
              </li>
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Delete all organization settings
              </li>
            </ul>
          </div>

          {modalError && (
            <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20">
              <p className="text-sm text-destructive">{modalError}</p>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              Type{' '}
              <span className="font-mono text-destructive bg-destructive/10 px-1.5 py-0.5 rounded">
                {orgName}
              </span>{' '}
              to confirm
            </label>
            <Input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
              placeholder={orgName}
              className="font-mono"
              autoFocus
            />
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <Button
            variant="secondary"
            onClick={handleCloseModal}
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
        {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}

        {isPersonal ? (
          <p className="text-sm text-foreground-muted">
            Your personal organization cannot be deleted. To remove it, delete your account.
          </p>
        ) : !isOwner ? (
          <div>
            <p className="text-sm text-foreground mb-3">Leave organization</p>
            <p className="text-xs text-foreground-muted mb-4">
              You will lose access to all projects and data in this organization.
            </p>
            <Button
              variant="ghost"
              onClick={handleLeave}
              disabled={isLeaving}
              className="gap-2 text-destructive hover:bg-destructive/10 border border-destructive/20"
            >
              {isLeaving ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <LogOut className="w-4 h-4" />
              )}
              Leave organization
            </Button>
          </div>
        ) : (
          <div>
            <p className="text-sm text-foreground mb-3">Delete organization</p>
            <p className="text-xs text-foreground-muted mb-4">
              Permanently delete this organization and all its projects. This action cannot be undone.
            </p>
            <Button
              variant="ghost"
              onClick={() => setIsDeleteModalOpen(true)}
              className="gap-2 text-destructive hover:bg-destructive/10 border border-destructive/20"
            >
              <Trash2 className="w-4 h-4" />
              Delete organization
            </Button>
          </div>
        )}
      </div>

      {/* Render modal in a portal at document body */}
      {isDeleteModalOpen && createPortal(deleteModal, document.body)}
    </>
  );
}
