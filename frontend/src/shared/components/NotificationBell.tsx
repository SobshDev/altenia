import { useEffect, useState, useRef } from 'react';
import { createPortal } from 'react-dom';
import { BellRing, Check, X, Building2, Loader2, Clock } from 'lucide-react';
import { useNavigate } from 'react-router';
import { Button } from './Button';
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

export function NotificationBell() {
  const navigate = useNavigate();
  const {
    userInvites,
    inviteCount,
    fetchUserInvites,
    fetchInviteCount,
    acceptInvite,
    declineInvite,
  } = useInviteStore();
  const { fetchOrganizations } = useOrgStore();

  const [isOpen, setIsOpen] = useState(false);
  const [processingId, setProcessingId] = useState<string | null>(null);
  const [dropdownPosition, setDropdownPosition] = useState({ top: 0, left: 0 });
  const buttonRef = useRef<HTMLButtonElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchInviteCount();
    // Poll for invite count every 60 seconds
    const interval = setInterval(fetchInviteCount, 60000);
    return () => clearInterval(interval);
  }, [fetchInviteCount]);

  useEffect(() => {
    if (isOpen) {
      fetchUserInvites();
      // Calculate dropdown position based on button
      if (buttonRef.current) {
        const rect = buttonRef.current.getBoundingClientRect();
        setDropdownPosition({
          top: rect.bottom + 8,
          left: rect.left,
        });
      }
    }
  }, [isOpen, fetchUserInvites]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Node;
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(target) &&
        buttonRef.current &&
        !buttonRef.current.contains(target)
      ) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  const handleAccept = async (inviteId: string) => {
    setProcessingId(inviteId);
    try {
      await acceptInvite(inviteId);
      fetchOrganizations();
    } catch {
      // Error handled silently in dropdown
    } finally {
      setProcessingId(null);
    }
  };

  const handleDecline = async (inviteId: string) => {
    setProcessingId(inviteId);
    try {
      await declineInvite(inviteId);
    } catch {
      // Error handled silently in dropdown
    } finally {
      setProcessingId(null);
    }
  };

  const handleViewAll = () => {
    setIsOpen(false);
    navigate('/settings/account');
  };

  const dropdown = isOpen ? (
    <div
      ref={dropdownRef}
      className="fixed w-80 rounded-xl bg-surface border border-border shadow-xl z-[100] overflow-hidden"
      style={{ top: dropdownPosition.top, left: dropdownPosition.left }}
    >
      <div className="px-4 py-3 border-b border-border">
        <h3 className="font-medium text-foreground">Notifications</h3>
      </div>

      <div className="max-h-[400px] overflow-y-auto">
        {userInvites.length === 0 ? (
          <div className="px-4 py-8 text-center text-foreground-muted">
            <BellRing className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No pending invites</p>
          </div>
        ) : (
          <div className="divide-y divide-border">
            {userInvites.slice(0, 5).map((invite) => (
              <div key={invite.id} className="p-3">
                <div className="flex items-start gap-3">
                  <div className="p-1.5 rounded-lg bg-primary/10 shrink-0">
                    <Building2 className="w-4 h-4 text-primary" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-foreground truncate">
                      {invite.organization_name}
                    </p>
                    <p className="text-xs text-foreground-muted truncate">
                      from {invite.inviter_email}
                    </p>
                    <div className="flex items-center gap-2 mt-1">
                      <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-primary/10 text-primary">
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
                  <div className="flex justify-end mt-2">
                    <Loader2 className="w-4 h-4 animate-spin text-foreground-muted" />
                  </div>
                ) : (
                  <div className="flex justify-end gap-2 mt-2">
                    <Button
                      size="sm"
                      onClick={() => handleAccept(invite.id)}
                      className="h-7 px-2 text-xs gap-1"
                    >
                      <Check className="w-3 h-3" />
                      Accept
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleDecline(invite.id)}
                      className="h-7 px-2 text-xs gap-1 text-foreground-muted hover:text-destructive"
                    >
                      <X className="w-3 h-3" />
                      Decline
                    </Button>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {userInvites.length > 0 && (
        <div className="px-4 py-3 border-t border-border bg-surface-alt">
          <button
            onClick={handleViewAll}
            className="w-full text-sm text-primary hover:text-primary/80 font-medium transition-colors"
          >
            View all in settings
          </button>
        </div>
      )}
    </div>
  ) : null;

  return (
    <div className="relative">
      <button
        ref={buttonRef}
        onClick={() => setIsOpen(!isOpen)}
        className="relative p-2 rounded-lg text-foreground-muted hover:text-foreground hover:bg-surface-alt transition-colors"
        aria-label={`Notifications${inviteCount > 0 ? ` (${inviteCount} pending)` : ''}`}
      >
        <BellRing className="w-5 h-5" />
        {inviteCount > 0 && (
          <span className="absolute -top-0.5 -right-0.5 flex items-center justify-center w-5 h-5 text-xs font-medium text-white bg-primary rounded-full">
            {inviteCount > 9 ? '9+' : inviteCount}
          </span>
        )}
      </button>

      {createPortal(dropdown, document.body)}
    </div>
  );
}
