import { User, Shield, AlertTriangle, LogOut } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { useAuthStore } from '@/stores/authStore';
import { SectionCard } from '../components/SectionCard';
import { ChangeDisplayNameForm } from '../components/ChangeDisplayNameForm';
import { ChangeEmailForm } from '../components/ChangeEmailForm';
import { ChangePasswordForm } from '../components/ChangePasswordForm';
import { DeleteAccountSection } from '../components/DeleteAccountSection';
import { InvitesSection } from '../components/InvitesSection';
import { PrivacySettings } from '../components/PrivacySettings';

export function AccountPage() {
  const { logout } = useAuthStore();

  return (
    <div className="p-8">
      {/* Page Header */}
      <div
        className="flex items-start justify-between animate-fade-in-up mb-6"
        style={{ '--stagger': '0ms' } as React.CSSProperties}
      >
        <div>
          <h1 className="text-2xl font-bold text-foreground">
            Account Settings
          </h1>
          <p className="mt-1 text-foreground-muted">
            Manage your account preferences and security
          </p>
        </div>
        <Button
          variant="ghost"
          onClick={() => logout()}
          className="gap-2 text-foreground-muted hover:text-foreground"
        >
          <LogOut className="w-4 h-4" />
          Sign out
        </Button>
      </div>

      <div className="flex flex-col lg:flex-row gap-6 lg:items-start">
        {/* Left column */}
        <div className="flex-1 space-y-6">
          {/* Profile Section */}
          <SectionCard
            icon={User}
            title="Profile"
            description="Your display name and account details"
            staggerDelay={50}
          >
            <div className="space-y-6">
              <ChangeDisplayNameForm />
              <div className="border-t border-border pt-6">
                <ChangeEmailForm />
              </div>
            </div>
          </SectionCard>

          {/* Security Section */}
          <SectionCard
            icon={Shield}
            title="Security"
            description="Protect your account with a strong password"
            staggerDelay={120}
          >
            <ChangePasswordForm />
          </SectionCard>

          {/* Danger Zone */}
          <SectionCard
            icon={AlertTriangle}
            title="Danger Zone"
            variant="destructive"
            staggerDelay={190}
          >
            <DeleteAccountSection />
          </SectionCard>
        </div>

        {/* Right column */}
        <div className="flex-1 space-y-6">
          {/* Pending Invites */}
          <InvitesSection />

          {/* Privacy Settings */}
          <PrivacySettings />
        </div>
      </div>
    </div>
  );
}
