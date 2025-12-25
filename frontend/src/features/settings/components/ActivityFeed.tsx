import { useEffect, useState } from 'react';
import {
  Activity as ActivityIcon,
  UserPlus,
  UserMinus,
  Shield,
  Building2,
  Pencil,
  Mail,
  Check,
  X,
} from 'lucide-react';
import { useOrgStore } from '@/stores/orgStore';
import { apiClient } from '@/shared/api/client';
import type { Activity, ActivityType } from '@/shared/types/api';

const activityConfig: Record<
  ActivityType,
  {
    icon: React.ComponentType<{ className?: string }>;
    color: string;
    bgColor: string;
    getMessage: (activity: Activity) => string;
  }
> = {
  member_added: {
    icon: UserPlus,
    color: 'text-green-500',
    bgColor: 'bg-green-500/10',
    getMessage: (a) => `${a.actor_email} added ${a.target_email} to the organization`,
  },
  member_removed: {
    icon: UserMinus,
    color: 'text-red-500',
    bgColor: 'bg-red-500/10',
    getMessage: (a) => `${a.actor_email} removed ${a.target_email} from the organization`,
  },
  member_role_changed: {
    icon: Shield,
    color: 'text-blue-500',
    bgColor: 'bg-blue-500/10',
    getMessage: (a) =>
      `${a.actor_email} changed ${a.target_email}'s role to ${a.metadata?.new_role || 'member'}`,
  },
  org_created: {
    icon: Building2,
    color: 'text-primary',
    bgColor: 'bg-primary/10',
    getMessage: (a) => `${a.actor_email} created the organization`,
  },
  org_name_changed: {
    icon: Pencil,
    color: 'text-yellow-500',
    bgColor: 'bg-yellow-500/10',
    getMessage: (a) =>
      `${a.actor_email} renamed the organization to "${a.metadata?.new_name || 'Unknown'}"`,
  },
  invite_sent: {
    icon: Mail,
    color: 'text-blue-500',
    bgColor: 'bg-blue-500/10',
    getMessage: (a) =>
      `${a.actor_email} invited ${a.metadata?.invitee_email || 'someone'} as ${a.metadata?.role || 'member'}`,
  },
  invite_accepted: {
    icon: Check,
    color: 'text-green-500',
    bgColor: 'bg-green-500/10',
    getMessage: (a) => `${a.actor_email} accepted the invite and joined the organization`,
  },
  invite_declined: {
    icon: X,
    color: 'text-red-500',
    bgColor: 'bg-red-500/10',
    getMessage: (a) => `${a.actor_email} declined the invite`,
  },
};

function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

function ActivityItem({ activity }: { activity: Activity }) {
  const config = activityConfig[activity.type];
  const Icon = config.icon;

  return (
    <div className="flex gap-3 py-3">
      <div className={`w-8 h-8 rounded-lg ${config.bgColor} flex-shrink-0 flex items-center justify-center`}>
        <Icon className={`w-4 h-4 ${config.color}`} />
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-sm text-foreground leading-relaxed">
          {config.getMessage(activity)}
        </p>
        <p className="text-xs text-foreground-muted mt-1">
          {formatRelativeTime(activity.created_at)}
        </p>
      </div>
    </div>
  );
}

export function ActivityFeed() {
  const { currentOrg } = useOrgStore();
  const [activities, setActivities] = useState<Activity[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!currentOrg) return;

    const fetchActivities = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const data = await apiClient.get<Activity[]>(
          `/orgs/${currentOrg.id}/activities?limit=50`
        );
        setActivities(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch activities');
      } finally {
        setIsLoading(false);
      }
    };

    fetchActivities();
  }, [currentOrg]);

  return (
    <div className="glass-card glow rounded-2xl p-6 animate-fade-in-up" style={{ '--stagger': '150ms' } as React.CSSProperties}>
      <div className="flex items-start gap-4 mb-5">
        <div
          className="p-3 rounded-xl bg-primary/10 animate-icon-pop card-icon"
          style={{ '--stagger': '150ms' } as React.CSSProperties}
        >
          <ActivityIcon className="w-5 h-5 text-primary" />
        </div>
        <div className="flex-1">
          <h2 className="text-lg font-semibold text-foreground">Recent Activity</h2>
          <p className="mt-1 text-sm text-foreground-muted">Latest organization events</p>
        </div>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="flex gap-3 py-3 animate-pulse">
              <div className="w-8 h-8 rounded-lg bg-surface-alt" />
              <div className="flex-1 space-y-2">
                <div className="h-4 bg-surface-alt rounded w-3/4" />
                <div className="h-3 bg-surface-alt rounded w-1/4" />
              </div>
            </div>
          ))}
        </div>
      ) : error ? (
        <div className="py-8 text-center">
          <ActivityIcon className="w-10 h-10 text-red-500/50 mx-auto mb-3" />
          <p className="text-sm text-foreground-muted">{error}</p>
        </div>
      ) : activities.length === 0 ? (
        <div className="py-8 text-center">
          <ActivityIcon className="w-10 h-10 text-foreground-muted/50 mx-auto mb-3" />
          <p className="text-sm text-foreground-muted">No activity yet</p>
        </div>
      ) : (
        <div className="max-h-80 overflow-y-auto divide-y divide-border">
          {activities.map((activity) => (
            <ActivityItem key={activity.id} activity={activity} />
          ))}
        </div>
      )}
    </div>
  );
}
