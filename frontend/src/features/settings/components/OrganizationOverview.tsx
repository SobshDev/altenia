import { Users, Calendar, Crown, Shield, User } from 'lucide-react';
import { useOrgStore } from '@/stores/orgStore';

const roleConfig = {
  owner: { icon: Crown, label: 'Owner', color: 'text-yellow-500' },
  admin: { icon: Shield, label: 'Admins', color: 'text-blue-500' },
  member: { icon: User, label: 'Members', color: 'text-foreground-muted' },
};

export function OrganizationOverview() {
  const { currentOrg, members } = useOrgStore();

  if (!currentOrg) return null;

  const roleCounts = members.reduce(
    (acc, member) => {
      acc[member.role] = (acc[member.role] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  const createdDate = new Date(currentOrg.created_at).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });

  return (
    <div className="glass-card glow rounded-2xl p-6 space-y-5 animate-fade-in-up" style={{ '--stagger': '80ms' } as React.CSSProperties}>
      <div className="flex items-start gap-4">
        <div
          className="p-3 rounded-xl bg-primary/10 animate-icon-pop card-icon"
          style={{ '--stagger': '80ms' } as React.CSSProperties}
        >
          <Users className="w-5 h-5 text-primary" />
        </div>
        <div className="flex-1">
          <h2 className="text-lg font-semibold text-foreground">Organization Overview</h2>
          <p className="mt-1 text-sm text-foreground-muted">Quick stats and info</p>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-surface-alt rounded-lg p-4">
          <div className="flex items-center gap-2 text-foreground-muted mb-1">
            <Users className="w-4 h-4" />
            <span className="text-xs font-medium">Total Members</span>
          </div>
          <p className="text-2xl font-bold text-foreground">{members.length}</p>
        </div>
        <div className="bg-surface-alt rounded-lg p-4">
          <div className="flex items-center gap-2 text-foreground-muted mb-1">
            <Calendar className="w-4 h-4" />
            <span className="text-xs font-medium">Created</span>
          </div>
          <p className="text-sm font-medium text-foreground">{createdDate}</p>
        </div>
      </div>

      {/* Role breakdown */}
      {!currentOrg.is_personal && members.length > 0 && (
        <div className="space-y-3">
          <p className="text-xs font-medium text-foreground-muted uppercase tracking-wider">
            Role Distribution
          </p>
          <div className="space-y-2">
            {(['owner', 'admin', 'member'] as const).map((role) => {
              const config = roleConfig[role];
              const count = roleCounts[role] || 0;
              const percentage = members.length > 0 ? (count / members.length) * 100 : 0;
              const Icon = config.icon;

              if (count === 0) return null;

              return (
                <div key={role} className="flex items-center gap-3">
                  <Icon className={`w-4 h-4 ${config.color}`} />
                  <div className="flex-1">
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-foreground-muted">{config.label}</span>
                      <span className="font-medium text-foreground">{count}</span>
                    </div>
                    <div className="h-1.5 bg-surface-alt rounded-full overflow-hidden">
                      <div
                        className="h-full bg-primary rounded-full transition-all duration-300"
                        style={{ width: `${percentage}%` }}
                      />
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Organization type badge */}
      <div className="pt-5 border-t border-border">
        <div className="flex items-center justify-between">
          <span className="text-sm text-foreground-muted">Type</span>
          <span
            className={`text-xs font-medium px-2.5 py-1 rounded-full ${
              currentOrg.is_personal
                ? 'bg-foreground-muted/10 text-foreground-muted'
                : 'bg-primary/10 text-primary'
            }`}
          >
            {currentOrg.is_personal ? 'Personal' : 'Team'}
          </span>
        </div>
      </div>
    </div>
  );
}
