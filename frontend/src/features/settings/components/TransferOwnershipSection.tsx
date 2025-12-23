import { useState, useRef, useEffect } from 'react';
import { ArrowRightLeft, Loader2, Crown, ChevronDown, User } from 'lucide-react';
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
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const isOwner = currentOrg?.role === 'owner';
  const isPersonal = currentOrg?.is_personal;

  // Get eligible members (non-owners, excluding current user)
  const eligibleMembers = members.filter(
    (m) => m.user_id !== user?.id && m.role !== 'owner'
  );

  const selectedMember = members.find((m) => m.user_id === selectedUserId);

  // Close dropdown on click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsDropdownOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Close on escape
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setIsDropdownOpen(false);
    };
    if (isDropdownOpen) {
      window.addEventListener('keydown', handleEscape);
    }
    return () => window.removeEventListener('keydown', handleEscape);
  }, [isDropdownOpen]);

  const handleSelectMember = (userId: string) => {
    setSelectedUserId(userId);
    setIsDropdownOpen(false);
  };

  const handleTransfer = async () => {
    if (!selectedUserId) return;

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
          <div className="relative flex-1" ref={dropdownRef}>
            {/* Custom dropdown trigger */}
            <button
              type="button"
              onClick={() => setIsDropdownOpen(!isDropdownOpen)}
              className="w-full flex items-center gap-3 pl-3 pr-4 py-2.5 rounded-lg bg-surface border border-border text-foreground hover:border-foreground-muted focus:outline-none focus:ring-2 focus:ring-primary/50 transition-colors"
            >
              <div className="p-1.5 rounded-md bg-primary/10">
                <Crown className="w-4 h-4 text-primary" />
              </div>
              <span className="flex-1 text-left text-sm">
                {selectedMember ? (
                  <span className="text-foreground">
                    {selectedMember.display_name || selectedMember.email}
                  </span>
                ) : (
                  <span className="text-foreground-muted">Select a member</span>
                )}
              </span>
              <ChevronDown className={`w-4 h-4 text-foreground-muted transition-transform duration-200 ${isDropdownOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Dropdown menu */}
            {isDropdownOpen && (
              <div className="absolute z-50 w-full mt-2 py-1 rounded-lg bg-surface border border-border shadow-lg animate-fade-in">
                {eligibleMembers.map((member) => (
                  <button
                    key={member.user_id}
                    type="button"
                    onClick={() => handleSelectMember(member.user_id)}
                    className={`w-full flex items-center gap-3 px-3 py-2.5 text-left hover:bg-surface-alt transition-colors ${
                      selectedUserId === member.user_id ? 'bg-primary/5' : ''
                    }`}
                  >
                    <div className="p-1.5 rounded-md bg-surface-alt">
                      <User className="w-4 h-4 text-foreground-muted" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-foreground truncate">
                        {member.display_name || member.email}
                      </p>
                      <p className="text-xs text-foreground-muted capitalize">
                        {member.display_name ? member.email : member.role}
                      </p>
                    </div>
                    {selectedUserId === member.user_id && (
                      <div className="w-2 h-2 rounded-full bg-primary" />
                    )}
                  </button>
                ))}
              </div>
            )}
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
