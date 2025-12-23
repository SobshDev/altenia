import { useAuthStore } from '@/stores/authStore';

export function AccountPage() {
  const { user, logout } = useAuthStore();

  return (
    <div className="p-8">
      <div className="max-w-2xl">
        <h1 className="text-2xl font-bold text-foreground mb-6">Account Settings</h1>

        <div className="bg-surface rounded-xl border border-border p-6 space-y-6">
          <div>
            <h2 className="text-sm font-medium text-foreground-muted mb-1">Email</h2>
            <p className="text-foreground">{user?.email || 'Not available'}</p>
          </div>

          <div className="pt-4 border-t border-border">
            <button
              onClick={() => logout()}
              className="px-4 py-2 text-sm font-medium text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
            >
              Sign out
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
