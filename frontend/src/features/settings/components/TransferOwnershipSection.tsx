import { useState } from 'react';
import { ArrowRightLeft, Loader2, Crown } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { useOrgStore } from '@/stores/orgStore';
import { useAuthStore } from '@/stores/authStore';

export function TransferOwnershipSection() {
  const { currentOrg, members, transferOwnership } = useOrgStore();
  const { user } = useAuthStore();
  const [selectedUserId, setSelectedUserId] = useState<string>('');
  const [isTransferring, setIsTransferring] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const isOwner = currentOrg?.role === 'owner';
  const isPersonal = currentOrg?.is_personal;

  // Get eligible members (non-owners, excluding current user)
  const eligibleMembers = members.filter(
    (m) => m.user_id !== user?.id && m.role !== 'owner'
  );

  const handleTransfer = async () => {
    if (!selectedUserId) return;

    const selectedMember = members.find((m) => m.user_id === selectedUserId);
    if (!selectedMember) return;

    const memberName = selectedMember.display_name || selectedMember.email;
    const confirmed = confirm(
      `Are you sure you want to transfer ownership to ${memberName}? They will become an owner of this organization.`
    );
    if (!confirmed) return;

    setIsTransferring(true);
    setError(null);
    try {
      await transferOwnership(selectedUserId);
      setSuccess(`Ownership transferred to ${memberName}`);
      setSelectedUserId('');
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to transfer ownership');
    } finally {
      setIsTransferring(false);
    }
  };

  // Don't show for non-owners or personal orgs
  if (!isOwner || isPersonal) {
    return null;
  }

  if (eligibleMembers.length === 0) {
    return (
      <div>
        <p className="text-sm text-foreground mb-2">Transfer ownership</p>
        <p className="text-xs text-foreground-muted">
          Add other members to the organization before transferring ownership.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}
      {success && <SuccessAlert message={success} onDismiss={() => setSuccess(null)} />}

      <div>
        <p className="text-sm text-foreground mb-2">Transfer ownership</p>
        <p className="text-xs text-foreground-muted mb-4">
          Transfer organization ownership to another member. They will become an owner.
        </p>

        <div className="flex items-center gap-3">
          <div className="relative flex-1">
            <Crown className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground-muted" />
            <select
              value={selectedUserId}
              onChange={(e) => setSelectedUserId(e.target.value)}
              className="w-full pl-10 pr-4 py-2 rounded-lg bg-surface border border-border text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 appearance-none"
            >
              <option value="">Select a member</option>
              {eligibleMembers.map((member) => (
                <option key={member.user_id} value={member.user_id}>
                  {member.display_name || member.email} ({member.role})
                </option>
              ))}
            </select>
          </div>

          <Button
            variant="ghost"
            onClick={handleTransfer}
            disabled={!selectedUserId || isTransferring}
            className="gap-2 text-foreground-muted hover:text-foreground border border-border"
          >
            {isTransferring ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <ArrowRightLeft className="w-4 h-4" />
            )}
            Transfer
          </Button>
        </div>
      </div>
    </div>
  );
}
