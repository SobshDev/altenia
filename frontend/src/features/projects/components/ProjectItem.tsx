import { NavLink, useLocation } from 'react-router';
import {
  FolderKanban,
  ScrollText,
  LineChart,
  Route,
  Bell,
  Settings,
  ChevronRight,
} from 'lucide-react';
import { Tooltip } from '@/shared/components/Tooltip';
import type { Project } from '@/shared/types/api';

interface SubNavItem {
  label: string;
  href: string;
  icon: React.ComponentType<{ className?: string }>;
}

interface ProjectItemProps {
  project: Project;
  collapsed: boolean;
  isExpanded: boolean;
  onToggle: () => void;
}

export function ProjectItem({
  project,
  collapsed,
  isExpanded,
  onToggle,
}: ProjectItemProps) {
  const location = useLocation();
  const basePath = `/projects/${project.id}`;

  const subNavItems: SubNavItem[] = [
    { label: 'Logs', href: `${basePath}/logs`, icon: ScrollText },
    { label: 'Metrics', href: `${basePath}/metrics`, icon: LineChart },
    { label: 'Traces', href: `${basePath}/traces`, icon: Route },
    { label: 'Alerts', href: `${basePath}/alerts`, icon: Bell },
    { label: 'Settings', href: `${basePath}/settings`, icon: Settings },
  ];

  const isProjectActive = location.pathname.startsWith(basePath);

  if (collapsed) {
    return (
      <Tooltip content={project.name} enabled>
        <button
          onClick={onToggle}
          className={`flex items-center justify-center p-2.5 rounded-lg transition-all duration-200 w-full ${
            isProjectActive
              ? 'bg-primary/10 text-primary'
              : 'text-foreground-muted hover:bg-surface-alt hover:text-foreground'
          }`}
        >
          <FolderKanban className="w-5 h-5 flex-shrink-0" />
        </button>
      </Tooltip>
    );
  }

  return (
    <div className="space-y-0.5">
      {/* Project header */}
      <button
        onClick={onToggle}
        className={`flex items-center gap-2 w-full px-3 py-2 rounded-lg transition-all duration-200 ${
          isProjectActive
            ? 'bg-primary/10 text-primary'
            : 'text-foreground-muted hover:bg-surface-alt hover:text-foreground'
        }`}
      >
        <ChevronRight
          className={`w-4 h-4 flex-shrink-0 transition-transform duration-200 ${
            isExpanded ? 'rotate-90' : ''
          }`}
        />
        <FolderKanban className="w-4 h-4 flex-shrink-0" />
        <span className="truncate text-sm font-medium">{project.name}</span>
      </button>

      {/* Sub navigation */}
      {isExpanded && (
        <div className="ml-4 pl-3 border-l border-border space-y-0.5">
          {subNavItems.map((item) => {
            const Icon = item.icon;
            return (
              <NavLink
                key={item.href}
                to={item.href}
                className={({ isActive }) =>
                  `flex items-center gap-2 px-3 py-1.5 rounded-lg transition-all duration-200 text-sm ${
                    isActive
                      ? 'bg-primary/10 text-primary'
                      : 'text-foreground-muted hover:bg-surface-alt hover:text-foreground'
                  }`
                }
              >
                <Icon className="w-4 h-4 flex-shrink-0" />
                <span>{item.label}</span>
              </NavLink>
            );
          })}
        </div>
      )}
    </div>
  );
}
