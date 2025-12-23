import { useState } from 'react';
import { LogOut, Trash2, Loader2 } from 'lucide-react';
import { useNavigate } from 'react-router';
import { Button } from '@/shared/components/Button';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { useOrgStore } from '@/stores/orgStore';

export function LeaveOrgSection() {
  const { currentOrg, leaveOrg, deleteOrg } = useOrgStore();
  const navigate = useNavigate();
  const [isLeaving, setIsLeaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isOwner = currentOrg?.role === 'owner';
  const isPersonal = currentOrg?.is_personal;

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

  const handleDelete = async () => {
    if (!confirm('Are you sure you want to delete this organization? This action cannot be undone.')) return;

    setIsDeleting(true);
    setError(null);
    try {
      await deleteOrg();
      navigate('/settings/organization');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete organization');
    } finally {
      setIsDeleting(false);
    }
  };

  return (
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
            onClick={handleDelete}
            disabled={isDeleting}
            className="gap-2 text-destructive hover:bg-destructive/10 border border-destructive/20"
          >
            {isDeleting ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Trash2 className="w-4 h-4" />
            )}
            Delete organization
          </Button>
        </div>
      )}
    </div>
  );
}
