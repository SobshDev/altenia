import { useEffect } from 'react';
import { useParams } from 'react-router';
import { Bell } from 'lucide-react';
import { useProjectStore } from '@/stores/projectStore';

export function ProjectAlertsPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const { projects, currentProject, setCurrentProject } = useProjectStore();

  useEffect(() => {
    if (projectId) {
      const project = projects.find((p) => p.id === projectId);
      if (project) {
        setCurrentProject(project);
      }
    }
  }, [projectId, projects, setCurrentProject]);

  return (
    <div className="p-8">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-3 rounded-xl bg-primary/10">
          <Bell className="w-6 h-6 text-primary" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-foreground">Alerts</h1>
          <p className="text-foreground-muted">
            {currentProject?.name || 'Project'} - Alert management
          </p>
        </div>
      </div>
      <div className="rounded-2xl border border-border bg-surface p-8 text-center">
        <Bell className="w-12 h-12 text-foreground-muted mx-auto mb-4 opacity-50" />
        <p className="text-foreground-muted">Alerts management coming soon...</p>
        <p className="text-sm text-foreground-muted mt-2">
          Create alert rules and manage notification channels
        </p>
      </div>
    </div>
  );
}
