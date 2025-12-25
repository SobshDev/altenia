import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { useNavigate } from 'react-router';
import { Trash2, Loader2, AlertTriangle, X } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { useProjectStore } from '@/stores/projectStore';

export function DeleteProjectSection() {
  const navigate = useNavigate();
  const { currentProject, deleteProject } = useProjectStore();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [confirmText, setConfirmText] = useState('');
  const [error, setError] = useState<string | null>(null);

  const projectName = currentProject?.name || '';
  const isConfirmed = confirmText.toLowerCase() === projectName.toLowerCase();

  const handleCloseModal = () => {
    setIsModalOpen(false);
    setConfirmText('');
    setError(null);
  };

  const handleDelete = async () => {
    if (!isConfirmed || !currentProject) return;

    setIsDeleting(true);
    setError(null);
    try {
      await deleteProject(currentProject.id);
      handleCloseModal();
      navigate('/');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete project');
    } finally {
      setIsDeleting(false);
    }
  };

  // Prevent body scroll when modal is open
  useEffect(() => {
    if (isModalOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isModalOpen]);

  // Close on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') handleCloseModal();
    };
    if (isModalOpen) {
      window.addEventListener('keydown', handleEscape);
    }
    return () => window.removeEventListener('keydown', handleEscape);
  }, [isModalOpen]);

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
          className="absolute top-4 right-4 p-2 rounded-lg text-foreground-muted hover:text-foreground hover:bg-surface-alt transition-colors"
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
              Delete project?
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
              Deleting this project will:
            </p>
            <ul className="mt-2 space-y-1 text-sm text-foreground-muted">
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Permanently delete all logs, metrics, and traces
              </li>
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Revoke all API keys for this project
              </li>
              <li className="flex items-center gap-2">
                <span className="w-1 h-1 rounded-full bg-destructive" />
                Remove all alert rules and configurations
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
                {projectName}
              </span>{' '}
              to confirm
            </label>
            <Input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
              placeholder={projectName}
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
      <div>
        <p className="text-sm text-foreground mb-3">Delete project</p>
        <p className="text-xs text-foreground-muted mb-4">
          Permanently delete this project and all its data. This action cannot be undone.
        </p>
        <Button
          variant="ghost"
          onClick={() => setIsModalOpen(true)}
          className="gap-2 text-destructive hover:bg-destructive/10 border border-destructive/20"
        >
          <Trash2 className="w-4 h-4" />
          Delete project
        </Button>
      </div>

      {isModalOpen && createPortal(deleteModal, document.body)}
    </>
  );
}
