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

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* General - order-1 on mobile */}
        <SectionCard
          icon={Building2}
          title="General"
          description="Organization name and details"
          staggerDelay={50}
          className="order-1 lg:order-none"
        >
          <ChangeOrgNameForm />
        </SectionCard>

        {/* Right column - order-3 on mobile, spans 3 rows on desktop */}
        <div className="order-3 lg:order-none lg:row-span-3 space-y-6">
          <OrganizationOverview />
          <ActivityFeed />
        </div>

        {/* Members - order-2 on mobile */}
        <SectionCard
          icon={Users}
          title="Members"
          description="Manage organization members and their roles"
          staggerDelay={120}
          className="order-2 lg:order-none"
        >
          <MembersSection />
        </SectionCard>

        {/* Danger Zone - order-4 (last) on mobile, left column on desktop */}
        <SectionCard
          icon={AlertTriangle}
          title="Danger Zone"
          variant="destructive"
          staggerDelay={190}
          className="order-4 lg:order-none"
        >
          <div className="space-y-6">
            <TransferOwnershipSection />
            <LeaveOrgSection />
          </div>
        </SectionCard>
      </div>
    </div>
  );
}
