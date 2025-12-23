import { User, Shield, AlertTriangle } from 'lucide-react';
import { Button } from '@/shared/components/Button';
import { useAuthStore } from '@/stores/authStore';
import { SectionCard } from '../components/SectionCard';
import { ChangeEmailForm } from '../components/ChangeEmailForm';
import { ChangePasswordForm } from '../components/ChangePasswordForm';

export function AccountPage() {
  const { logout } = useAuthStore();

  return (
    <div className="p-8">
      <div className="max-w-2xl space-y-6">
        {/* Page Header */}
        <div
          className="animate-fade-in-up"
          style={{ '--stagger': '0ms' } as React.CSSProperties}
        >
          <h1 className="text-2xl font-bold text-foreground">Account Settings</h1>
          <p className="mt-1 text-foreground-muted">
            Manage your account preferences and security
          </p>
        </div>

        {/* Account Section */}
        <SectionCard
          icon={User}
          title="Account"
          description="Your account email and preferences"
          staggerDelay={100}
        >
          <ChangeEmailForm />
        </SectionCard>

        {/* Security Section */}
        <SectionCard
          icon={Shield}
          title="Security"
          description="Protect your account with a strong password"
          staggerDelay={200}
        >
          <ChangePasswordForm />
        </SectionCard>

        {/* Danger Zone */}
        <SectionCard
          icon={AlertTriangle}
          title="Danger Zone"
          variant="destructive"
          staggerDelay={300}
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-foreground">Sign out of your account</p>
              <p className="text-xs text-foreground-muted mt-0.5">
                You will need to sign in again to access your data
              </p>
            </div>
            <Button
              variant="ghost"
              onClick={() => logout()}
              className="text-destructive hover:bg-destructive/10"
            >
              Sign out
            </Button>
          </div>
        </SectionCard>
      </div>
    </div>
  );
}
