import { useEffect } from 'react';
import { useParams } from 'react-router';
import { Route } from 'lucide-react';
import { useProjectStore } from '@/stores/projectStore';

export function ProjectTracesPage() {
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
          <Route className="w-6 h-6 text-primary" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-foreground">Traces</h1>
          <p className="text-foreground-muted">
            {currentProject?.name || 'Project'} - Distributed tracing
          </p>
        </div>
      </div>
      <div className="rounded-2xl border border-border bg-surface p-8 text-center">
        <Route className="w-12 h-12 text-foreground-muted mx-auto mb-4 opacity-50" />
        <p className="text-foreground-muted">Trace viewer coming soon...</p>
        <p className="text-sm text-foreground-muted mt-2">
          Distributed tracing with waterfall views and service maps
        </p>
      </div>
    </div>
  );
}
