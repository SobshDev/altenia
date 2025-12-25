import { useEffect, useState } from 'react';
import { Mail, Check, X, Building2, Loader2, Clock } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { SectionCard } from './SectionCard';
import { useInviteStore } from '@/stores/inviteStore';
import { useOrgStore } from '@/stores/orgStore';

function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays <= 0) return 'Expired';
  if (diffDays === 1) return 'Expires tomorrow';
  return `Expires in ${diffDays} days`;
}

export function InvitesSection() {
  const {
    userInvites,
    userInvitesLoading,
    userInvitesError,
    fetchUserInvites,
    acceptInvite,
    declineInvite,
    clearUserInvitesError,
  } = useInviteStore();
  const { fetchOrganizations } = useOrgStore();

  const [processingId, setProcessingId] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchUserInvites();
  }, [fetchUserInvites]);

  const handleAccept = async (inviteId: string, orgName: string) => {
    setProcessingId(inviteId);
    setError(null);
    try {
      await acceptInvite(inviteId);
      setSuccess(`You have joined ${orgName}`);
      fetchOrganizations(); // Refresh the orgs list
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to accept invite');
    } finally {
      setProcessingId(null);
    }
  };

  const handleDecline = async (inviteId: string) => {
    setProcessingId(inviteId);
    setError(null);
    try {
      await declineInvite(inviteId);
      setSuccess('Invite declined');
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to decline invite');
    } finally {
      setProcessingId(null);
    }
  };

  if (userInvitesLoading && userInvites.length === 0) {
    return (
      <SectionCard
        icon={Mail}
        title="Pending Invites"
        description="Organization invites waiting for your response"
        staggerDelay={300}
      >
        <div className="flex items-center justify-center py-8">
          <Loader2 className="w-6 h-6 animate-spin text-foreground-muted" />
        </div>
      </SectionCard>
    );
  }

  return (
    <SectionCard
      icon={Mail}
      title="Pending Invites"
      description="Organization invites waiting for your response"
      staggerDelay={300}
    >
      {(error || userInvitesError) && (
        <ErrorAlert
          message={error || userInvitesError || ''}
          onDismiss={() => {
            setError(null);
            clearUserInvitesError();
          }}
        />
      )}
      {success && <SuccessAlert message={success} onDismiss={() => setSuccess(null)} />}

      {userInvites.length === 0 ? (
        <div className="text-center py-8 text-foreground-muted">
          <Mail className="w-12 h-12 mx-auto mb-3 opacity-50" />
          <p>No pending invites</p>
        </div>
      ) : (
        <div className="space-y-3">
          {userInvites.map((invite) => (
            <div
              key={invite.id}
              className="p-4 rounded-lg bg-surface-alt border border-border"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex items-start gap-3 min-w-0">
                  <div className="p-2 rounded-lg bg-primary/10 shrink-0">
                    <Building2 className="w-5 h-5 text-primary" />
                  </div>
                  <div className="min-w-0">
                    <p className="font-medium text-foreground truncate">
                      {invite.organization_name}
                    </p>
                    <p className="text-sm text-foreground-muted">
                      Invited by {invite.inviter_email}
                    </p>
                    <div className="flex items-center gap-2 mt-1">
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-primary/10 text-primary">
                        {invite.role === 'admin' ? 'Admin' : 'Member'}
                      </span>
                      <span className="flex items-center gap-1 text-xs text-foreground-muted">
                        <Clock className="w-3 h-3" />
                        {formatRelativeTime(invite.expires_at)}
                      </span>
                    </div>
                  </div>
                </div>

                {processingId === invite.id ? (
                  <Loader2 className="w-5 h-5 animate-spin text-foreground-muted shrink-0" />
                ) : (
                  <div className="flex items-center gap-2 shrink-0">
                    <Button
                      size="sm"
                      onClick={() => handleAccept(invite.id, invite.organization_name)}
                      className="gap-1"
                    >
                      <Check className="w-4 h-4" />
                      Accept
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleDecline(invite.id)}
                      className="gap-1 text-foreground-muted hover:text-destructive"
                    >
                      <X className="w-4 h-4" />
                      Decline
                    </Button>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </SectionCard>
  );
}
