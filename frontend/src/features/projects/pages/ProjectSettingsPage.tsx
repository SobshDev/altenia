import { useEffect } from 'react';
import { useParams, useNavigate } from 'react-router';
import { Settings, FolderKanban, Database, Key, AlertTriangle } from 'lucide-react';
import { SectionCard } from '@/features/settings/components/SectionCard';
import { useProjectStore } from '@/stores/projectStore';
import { ChangeProjectNameForm } from '../components/ChangeProjectNameForm';
import { ProjectRetentionForm } from '../components/ProjectRetentionForm';
import { ApiKeySection } from '../components/ApiKeySection';
import { DeleteProjectSection } from '../components/DeleteProjectSection';

export function ProjectSettingsPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const { projects, currentProject, setCurrentProject, fetchApiKeys } = useProjectStore();

  useEffect(() => {
    if (projectId) {
      const project = projects.find((p) => p.id === projectId);
      if (project) {
        setCurrentProject(project);
        fetchApiKeys(projectId);
      } else if (projects.length > 0) {
        // Project not found, redirect to dashboard
        navigate('/');
      }
    }
  }, [projectId, projects, setCurrentProject, fetchApiKeys, navigate]);

  if (!currentProject) {
    return (
      <div className="p-8">
        <div className="flex items-center justify-center py-16">
          <div className="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin" />
        </div>
      </div>
    );
  }

  return (
    <div className="p-8">
      {/* Page Header */}
      <div
        className="animate-fade-in-up mb-6"
        style={{ '--stagger': '0ms' } as React.CSSProperties}
      >
        <div className="flex items-center gap-3">
          <div className="p-3 rounded-xl bg-primary/10">
            <Settings className="w-6 h-6 text-primary" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-foreground">
              {currentProject.name}
            </h1>
            <p className="text-foreground-muted">
              Manage project settings and API keys
            </p>
          </div>
        </div>
      </div>

      <div className="flex flex-col lg:flex-row gap-6 lg:items-start">
        {/* Left column */}
        <div className="flex-1 space-y-6">
          <SectionCard
            icon={FolderKanban}
            title="General"
            description="Project name and description"
            staggerDelay={50}
          >
            <ChangeProjectNameForm />
          </SectionCard>

          <SectionCard
            icon={Database}
            title="Data Retention"
            description="Configure how long data is stored"
            staggerDelay={120}
          >
            <ProjectRetentionForm />
          </SectionCard>

          {/* Danger Zone */}
          <SectionCard
            icon={AlertTriangle}
            title="Danger Zone"
            variant="destructive"
            staggerDelay={190}
          >
            <DeleteProjectSection />
          </SectionCard>
        </div>

        {/* Right column */}
        <div className="flex-1 space-y-6">
          <SectionCard
            icon={Key}
            title="API Keys"
            description="Manage keys for data ingestion"
            staggerDelay={80}
          >
            <ApiKeySection />
          </SectionCard>
        </div>
      </div>
    </div>
  );
}
