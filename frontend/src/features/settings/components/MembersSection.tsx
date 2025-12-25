import { useState, useEffect } from 'react';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { UserPlus, Crown, Shield, User, X, Loader2, Mail, Clock } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { Input } from '@/shared/components/Input';
import { ErrorAlert } from '@/shared/components/ErrorAlert';
import { SuccessAlert } from '@/shared/components/SuccessAlert';
import { RoleSelect } from '@/shared/components/RoleSelect';
import { useOrgStore } from '@/stores/orgStore';
import { useAuthStore } from '@/stores/authStore';
import { useInviteStore } from '@/stores/inviteStore';
import {
  addMemberSchema,
  type AddMemberFormValues,
} from '../schemas/settingsSchemas';

const roleIcons = {
  owner: Crown,
  admin: Shield,
  member: User,
};

const roleLabels = {
  owner: 'Owner',
  admin: 'Admin',
  member: 'Member',
};

function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays <= 0) return 'Expired';
  if (diffDays === 1) return 'Expires tomorrow';
  return `Expires in ${diffDays} days`;
}

export function MembersSection() {
  const { currentOrg, members, fetchMembers, updateMemberRole, removeMember } = useOrgStore();
  const { user } = useAuthStore();
  const { orgInvites, fetchOrgInvites, sendInvite, cancelInvite } = useInviteStore();
  const [isAdding, setIsAdding] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [loadingMember, setLoadingMember] = useState<string | null>(null);
  const [cancellingInvite, setCancellingInvite] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    reset,
    control,
    formState: { errors },
  } = useForm<AddMemberFormValues>({
    resolver: zodResolver(addMemberSchema),
    defaultValues: { role: 'member' },
  });

  useEffect(() => {
    fetchMembers();
    if (currentOrg?.id) {
      fetchOrgInvites(currentOrg.id);
    }
  }, [fetchMembers, fetchOrgInvites, currentOrg?.id]);

  const isOwner = currentOrg?.role === 'owner';
  const isAdmin = currentOrg?.role === 'admin' || isOwner;

  const onSubmit = async (data: AddMemberFormValues) => {
    if (!currentOrg) return;
    setIsLoading(true);
    setError(null);
    try {
      await sendInvite(currentOrg.id, data.email, data.role as 'admin' | 'member');
      setSuccess('Invite sent successfully');
      setIsAdding(false);
      reset();
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send invite');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancelInvite = async (inviteId: string) => {
    if (!currentOrg) return;
    setCancellingInvite(inviteId);
    setError(null);
    try {
      await cancelInvite(currentOrg.id, inviteId);
      setSuccess('Invite cancelled');
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cancel invite');
    } finally {
      setCancellingInvite(null);
    }
  };

  const handleRoleChange = async (userId: string, newRole: string) => {
    setLoadingMember(userId);
    setError(null);
    try {
      await updateMemberRole(userId, newRole);
      setSuccess('Role updated successfully');
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update role');
    } finally {
      setLoadingMember(null);
    }
  };

  const handleRemove = async (userId: string) => {
    setLoadingMember(userId);
    setError(null);
    try {
      await removeMember(userId);
      setSuccess('Member removed successfully');
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove member');
    } finally {
      setLoadingMember(null);
    }
  };

  const handleCancel = () => {
    setIsAdding(false);
    setError(null);
    reset();
  };

  return (
    <div className="space-y-4">
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}
      {success && <SuccessAlert message={success} onDismiss={() => setSuccess(null)} />}

      {/* Members list */}
      <div className="space-y-2">
        {members.map((member, index) => {
          const RoleIcon = roleIcons[member.role];
          const isCurrentUser = member.user_id === user?.id;
          const canManage = isOwner && !isCurrentUser && member.role !== 'owner';
          const canRemove = (isOwner || (isAdmin && member.role === 'member')) && !isCurrentUser;

          return (
            <div
              key={member.id}
              className="flex items-center justify-between p-3 rounded-lg bg-surface-alt animate-list-item hover:bg-surface-alt/80 transition-colors"
              style={{ '--item-index': index } as React.CSSProperties}
            >
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-primary/10">
                  <RoleIcon className="w-4 h-4 text-primary" />
                </div>
                <div>
                  <p className="text-sm font-medium text-foreground">
                    {member.display_name || member.email}
                    {isCurrentUser && (
                      <span className="ml-2 text-xs text-foreground-muted">(you)</span>
                    )}
                  </p>
                  <p className="text-xs text-foreground-muted">
                    {member.display_name ? member.email : roleLabels[member.role]}
                  </p>
                </div>
              </div>

              {loadingMember === member.user_id ? (
                <Loader2 className="w-4 h-4 animate-spin text-foreground-muted" />
              ) : (
                <div className="flex items-center gap-2">
                  {canManage && (
                    <RoleSelect
                      value={member.role as 'admin' | 'member'}
                      onChange={(role) => handleRoleChange(member.user_id, role)}
                      size="sm"
                    />
                  )}
                  {canRemove && (
                    <button
                      onClick={() => handleRemove(member.user_id)}
                      className="p-1 rounded text-foreground-muted hover:text-destructive hover:bg-destructive/10 transition-colors"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Pending invites */}
      {isAdmin && orgInvites.length > 0 && (
        <div className="space-y-2">
          <p className="text-sm font-medium text-foreground-muted">Pending Invites</p>
          {orgInvites.map((invite, index) => (
            <div
              key={invite.id}
              className="flex items-center justify-between p-3 rounded-lg bg-surface-alt border border-border/50 animate-list-item hover:border-primary/20 transition-colors"
              style={{ '--item-index': index } as React.CSSProperties}
            >
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-primary/10">
                  <Mail className="w-4 h-4 text-primary" />
                </div>
                <div>
                  <p className="text-sm font-medium text-foreground">
                    {invite.invitee_email}
                  </p>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-foreground-muted">
                      {invite.role === 'admin' ? 'Admin' : 'Member'}
                    </span>
                    <span className="flex items-center gap-1 text-xs text-foreground-muted">
                      <Clock className="w-3 h-3" />
                      {formatRelativeTime(invite.expires_at)}
                    </span>
                  </div>
                </div>
              </div>
              {cancellingInvite === invite.id ? (
                <Loader2 className="w-4 h-4 animate-spin text-foreground-muted" />
              ) : (
                <button
                  onClick={() => handleCancelInvite(invite.id)}
                  className="p-1 rounded text-foreground-muted hover:text-destructive hover:bg-destructive/10 transition-colors"
                  title="Cancel invite"
                >
                  <X className="w-4 h-4" />
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Invite member form */}
      {isAdmin && (
        <>
          {!isAdding ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsAdding(true)}
              className="gap-2"
            >
              <UserPlus className="w-4 h-4" />
              Invite member
            </Button>
          ) : (
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 pt-2">
              <Input
                label="Email address"
                type="email"
                placeholder="Enter member's email"
                error={errors.email?.message}
                {...register('email')}
              />

              <div>
                <label className="block text-sm font-medium text-foreground mb-2">
                  Role
                </label>
                <Controller
                  name="role"
                  control={control}
                  render={({ field }) => (
                    <RoleSelect
                      value={field.value}
                      onChange={field.onChange}
                      showAdminOption={isOwner}
                    />
                  )}
                />
                {errors.role?.message && (
                  <p className="mt-1 text-sm text-destructive">{errors.role.message}</p>
                )}
              </div>

              <div className="flex gap-3 pt-2">
                <Button type="submit" isLoading={isLoading}>
                  Send invite
                </Button>
                <Button type="button" variant="ghost" onClick={handleCancel}>
                  Cancel
                </Button>
              </div>
            </form>
          )}
        </>
      )}
    </div>
  );
}
