import { useEffect, useState } from 'react';
import { useLocation } from 'react-router';
import { Plus, Loader2, FolderKanban } from 'lucide-react';
import { useOrgStore } from '@/stores/orgStore';
import { useProjectStore } from '@/stores/projectStore';
import { useSidebarStore } from '@/stores/sidebarStore';
import { Tooltip } from '@/shared/components/Tooltip';
import { ProjectItem } from './ProjectItem';
import { CreateProjectModal } from './CreateProjectModal';

interface ProjectListProps {
  collapsed: boolean;
}

export function ProjectList({ collapsed }: ProjectListProps) {
  const location = useLocation();
  const { currentOrg } = useOrgStore();
  const {
    projects,
    isLoading,
    expandedProjects,
    fetchProjects,
    toggleProjectExpanded,
    setProjectExpanded,
    reset,
  } = useProjectStore();
  const { setCollapsed } = useSidebarStore();

  const [showCreateModal, setShowCreateModal] = useState(false);

  // Fetch projects when org changes
  useEffect(() => {
    if (currentOrg?.id) {
      fetchProjects(currentOrg.id);
    } else {
      reset();
    }
  }, [currentOrg?.id, fetchProjects, reset]);

  // Auto-expand project when navigating to its routes
  useEffect(() => {
    const match = location.pathname.match(/^\/projects\/([^/]+)/);
    if (match) {
      const projectId = match[1];
      setProjectExpanded(projectId, true);
    }
  }, [location.pathname, setProjectExpanded]);

  const handleToggleProject = (projectId: string) => {
    if (collapsed) {
      setCollapsed(false);
      setProjectExpanded(projectId, true);
    } else {
      toggleProjectExpanded(projectId);
    }
  };

  if (isLoading && projects.length === 0) {
    return (
      <div className="flex items-center justify-center py-4">
        <Loader2 className="w-5 h-5 animate-spin text-foreground-muted" />
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {/* Section header */}
      {!collapsed && (
        <div className="flex items-center justify-between px-3 py-1">
          <span className="text-xs font-medium text-foreground-muted uppercase tracking-wider">
            Projects
          </span>
          <button
            onClick={() => setShowCreateModal(true)}
            className="p-1 rounded text-foreground-muted hover:text-foreground hover:bg-surface-alt transition-colors"
            aria-label="Create new project"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>
      )}

      {/* Empty state */}
      {projects.length === 0 && !isLoading && (
        <div className={`text-center py-4 ${collapsed ? 'px-1' : 'px-3'}`}>
          {collapsed ? (
            <Tooltip content="No projects" enabled>
              <FolderKanban className="w-5 h-5 text-foreground-muted mx-auto opacity-50" />
            </Tooltip>
          ) : (
            <>
              <FolderKanban className="w-8 h-8 text-foreground-muted mx-auto mb-2 opacity-50" />
              <p className="text-xs text-foreground-muted">No projects yet</p>
              <button
                onClick={() => setShowCreateModal(true)}
                className="mt-2 text-xs text-primary hover:text-primary/80 transition-colors"
              >
                Create your first project
              </button>
            </>
          )}
        </div>
      )}

      {/* Project list */}
      {projects.map((project) => (
        <ProjectItem
          key={project.id}
          project={project}
          collapsed={collapsed}
          isExpanded={expandedProjects.has(project.id)}
          onToggle={() => handleToggleProject(project.id)}
        />
      ))}

      {/* New project button (collapsed) */}
      {collapsed && projects.length > 0 && (
        <Tooltip content="New project" enabled>
          <button
            onClick={() => {
              setCollapsed(false);
              setShowCreateModal(true);
            }}
            className="flex items-center justify-center p-2.5 rounded-lg w-full text-foreground-muted hover:bg-surface-alt hover:text-foreground transition-colors"
          >
            <Plus className="w-5 h-5" />
          </button>
        </Tooltip>
      )}

      {/* Create project modal */}
      <CreateProjectModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
      />
    </div>
  );
}
