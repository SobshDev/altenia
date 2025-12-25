import { Building2, Users, AlertTriangle } from 'lucide-react';
import { SectionCard } from '../components/SectionCard';
import { ChangeOrgNameForm } from '../components/ChangeOrgNameForm';
import { MembersSection } from '../components/MembersSection';
import { LeaveOrgSection } from '../components/LeaveOrgSection';
import { TransferOwnershipSection } from '../components/TransferOwnershipSection';
import { OrganizationOverview } from '../components/OrganizationOverview';
import { ActivityFeed } from '../components/ActivityFeed';
import { useOrgStore } from '@/stores/orgStore';

export function OrganizationPage() {
  const { currentOrg } = useOrgStore();

  const displayName = currentOrg?.is_personal
    ? 'Personal Organization'
    : currentOrg?.name || 'Organization';

  return (
    <div className="p-8">
      {/* Page Header */}
      <div
        className="animate-fade-in-up mb-6"
        style={{ '--stagger': '0ms' } as React.CSSProperties}
      >
        <h1 className="text-2xl font-bold text-foreground">
          {displayName}
        </h1>
        <p className="mt-1 text-foreground-muted">
          Manage your organization settings and members
        </p>
      </div>

      <div className="flex flex-col lg:flex-row gap-6 lg:items-start">
        {/* Left column */}
        <div className="flex-1 space-y-6">
          <SectionCard
            icon={Building2}
            title="General"
            description="Organization name and details"
            staggerDelay={100}
          >
            <ChangeOrgNameForm />
          </SectionCard>

          <SectionCard
            icon={Users}
            title="Members"
            description="Manage organization members and their roles"
            staggerDelay={200}
          >
            <MembersSection />
          </SectionCard>

          {/* Danger Zone */}
          <SectionCard
            icon={AlertTriangle}
            title="Danger Zone"
            variant="destructive"
            staggerDelay={300}
          >
            <div className="space-y-6">
              <TransferOwnershipSection />
              <LeaveOrgSection />
            </div>
          </SectionCard>
        </div>

        {/* Right column */}
        <div className="flex-1 space-y-6">
          <div
            className="animate-fade-in-up"
            style={{ '--stagger': '100ms' } as React.CSSProperties}
          >
            <OrganizationOverview />
          </div>

          <div
            className="animate-fade-in-up"
            style={{ '--stagger': '200ms' } as React.CSSProperties}
          >
            <ActivityFeed />
          </div>
        </div>
      </div>
    </div>
  );
}
