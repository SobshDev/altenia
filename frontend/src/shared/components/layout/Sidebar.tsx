import { NavLink } from 'react-router';
import {
  PanelLeftClose,
  PanelLeft,
  Settings,
  LayoutDashboard,
  FileText,
  Activity,
  GitBranch,
  Bell,
} from 'lucide-react';
import { useSidebarStore } from '@/stores/sidebarStore';
import { Tooltip } from '@/shared/components/Tooltip';

interface NavItem {
  label: string;
  href: string;
  icon: React.ComponentType<{ className?: string }>;
}

const mainNavItems: NavItem[] = [
  { label: 'Dashboard', href: '/', icon: LayoutDashboard },
  { label: 'Logs', href: '/logs', icon: FileText },
  { label: 'Metrics', href: '/metrics', icon: Activity },
  { label: 'Traces', href: '/traces', icon: GitBranch },
  { label: 'Alerts', href: '/alerts', icon: Bell },
];

const bottomNavItems: NavItem[] = [
  { label: 'Settings', href: '/settings/account', icon: Settings },
];

function Logo({
  collapsed,
  onClick,
}: {
  collapsed: boolean;
  onClick?: () => void;
}) {
  if (collapsed && onClick) {
    return (
      <button
        onClick={onClick}
        className="p-2 rounded-lg text-foreground-muted hover:bg-surface-alt hover:text-foreground transition-colors"
        aria-label="Expand sidebar"
      >
        <PanelLeft className="w-5 h-5" />
      </button>
    );
  }

  return (
    <div className="flex items-center gap-3 overflow-hidden">
      <svg
        viewBox="0 0 40 40"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className="w-8 h-8 flex-shrink-0"
      >
        <rect width="40" height="40" rx="8" className="fill-primary" />
        <path
          d="M12 28L20 12L28 28H12Z"
          className="stroke-white"
          strokeWidth="2"
          strokeLinejoin="round"
          fill="none"
        />
        <circle cx="20" cy="22" r="3" className="fill-white" />
      </svg>
      <span className="font-bold text-lg text-foreground whitespace-nowrap">
        Altenia
      </span>
    </div>
  );
}

function NavItemLink({ item, collapsed }: { item: NavItem; collapsed: boolean }) {
  const Icon = item.icon;

  return (
    <Tooltip content={item.label} enabled={collapsed}>
      <NavLink
        to={item.href}
        className={({ isActive }) =>
          `flex items-center rounded-lg transition-all duration-200 ${
            collapsed ? 'justify-center p-2.5' : 'gap-3 px-3 py-2.5'
          } ${
            isActive
              ? 'bg-primary/10 text-primary'
              : 'text-foreground-muted hover:bg-surface-alt hover:text-foreground'
          }`
        }
      >
        {({ isActive }) => (
          <>
            <Icon className={`w-5 h-5 flex-shrink-0 ${isActive ? 'text-primary' : ''}`} />
            {!collapsed && (
              <span className="whitespace-nowrap">{item.label}</span>
            )}
          </>
        )}
      </NavLink>
    </Tooltip>
  );
}

export function Sidebar() {
  const { isCollapsed, toggle } = useSidebarStore();

  return (
    <aside
      className={`fixed left-0 top-0 h-screen bg-background border-r border-border flex flex-col transition-all duration-300 ease-out z-40 overflow-hidden ${
        isCollapsed ? 'w-16' : 'w-60'
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between h-14 px-3 border-b border-border overflow-hidden">
        <Logo collapsed={isCollapsed} onClick={isCollapsed ? toggle : undefined} />
        <button
          onClick={toggle}
          className={`p-2 rounded-lg text-foreground-muted hover:bg-surface-alt hover:text-foreground transition-all duration-300 flex-shrink-0 ${
            isCollapsed ? 'opacity-0 pointer-events-none w-0' : 'opacity-100'
          }`}
          aria-label="Collapse sidebar"
        >
          <PanelLeftClose className="w-5 h-5" />
        </button>
      </div>

      {/* Main navigation */}
      <nav className="flex-1 p-3 space-y-1 overflow-y-auto overflow-x-hidden">
        {mainNavItems.map((item) => (
          <NavItemLink key={item.href} item={item} collapsed={isCollapsed} />
        ))}
      </nav>

      {/* Bottom navigation */}
      <div className="p-3 border-t border-border space-y-1 overflow-hidden">
        {bottomNavItems.map((item) => (
          <NavItemLink key={item.href} item={item} collapsed={isCollapsed} />
        ))}
      </div>
    </aside>
  );
}
