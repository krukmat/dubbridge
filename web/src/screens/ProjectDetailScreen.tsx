import { useEffect, useState } from 'react';
import { getProjectDetail, type ProjectDetail } from '../api/workspaceClient';

type Props = {
  orgId: string;
  projectId: string;
};

export function ProjectDetailScreen({ orgId, projectId }: Props) {
  const [detail, setDetail] = useState<ProjectDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    getProjectDetail(orgId, projectId).then((result) => {
      if (result.ok) {
        setDetail(result.value);
      } else {
        setError('Failed to load project');
      }
      setLoading(false);
    });
  }, [orgId, projectId]);

  return (
    <div data-testid="project-detail-screen">
      {loading && <p>Loading project…</p>}
      {error && <p role="alert">{error}</p>}
      {detail && (
        <>
          <h2>{detail.name}</h2>

          <section aria-label="Linked assets">
            <h3>Assets</h3>
            {detail.assets.length === 0 ? (
              <p>No assets linked.</p>
            ) : (
              detail.assets.map((asset) => (
                <div key={asset.id} data-testid="asset-row">
                  {asset.title}
                </div>
              ))
            )}
          </section>

          <section aria-label="Target languages">
            <h3>Target Languages</h3>
            {detail.target_languages.length === 0 ? (
              <p>No target languages declared.</p>
            ) : (
              detail.target_languages.map((lang) => (
                <div key={lang.id} data-testid="language-row">
                  {lang.source_lang} → {lang.target_lang}
                </div>
              ))
            )}
          </section>
        </>
      )}
    </div>
  );
}
