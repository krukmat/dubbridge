import { useEffect, useState } from 'react';
import { listProjects, type ProjectSummary } from '../api/workspaceClient';

type Props = {
  orgId: string;
  onSelectProject: (projectId: string) => void;
};

export function ProjectsScreen({ orgId, onSelectProject }: Props) {
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    listProjects(orgId).then((result) => {
      if (result.ok) {
        setProjects(result.value);
      } else {
        setError('Failed to load projects');
      }
      setLoading(false);
    });
  }, [orgId]);

  return (
    <div data-testid="projects-screen">
      {loading && <p>Loading projects…</p>}
      {error && <p role="alert">{error}</p>}
      {!loading && !error && projects.length === 0 && (
        <p>No projects found.</p>
      )}
      {projects.map((project) => (
        <div key={project.id} data-testid="project-row">
          <button onClick={() => onSelectProject(project.id)}>
            {project.name}
          </button>
        </div>
      ))}
    </div>
  );
}
