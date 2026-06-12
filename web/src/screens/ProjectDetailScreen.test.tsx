import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as workspaceClient from '../api/workspaceClient';
import { ProjectDetailScreen } from './ProjectDetailScreen';

const ORG_ID = 'org-1';
const PROJECT_ID = 'proj-a';

const BASE_DETAIL = {
  id: PROJECT_ID,
  org_id: ORG_ID,
  name: 'Alpha',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};

beforeEach(() => {
  vi.restoreAllMocks();
});

describe('ProjectDetailScreen — data-testid', () => {
  it('renders the project-detail-screen testid', async () => {
    vi.spyOn(workspaceClient, 'getProjectDetail').mockResolvedValueOnce({
      ok: true,
      value: { ...BASE_DETAIL, assets: [], target_languages: [] },
    });

    render(<ProjectDetailScreen orgId={ORG_ID} projectId={PROJECT_ID} />);
    await waitFor(() => expect(screen.getByTestId('project-detail-screen')).toBeInTheDocument());
  });
});

describe('ProjectDetailScreen — HP-1: project with assets + languages (SC-PROJECT-1, SC-LANG-1)', () => {
  it('shows linked assets and target languages', async () => {
    const detail = {
      ...BASE_DETAIL,
      assets: [
        { id: 'a1', title: 'Trailer', uploader_id: 'u1', status: 'finalized', created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z' },
        { id: 'a2', title: 'Intro', uploader_id: 'u1', status: 'pending', created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z' },
      ],
      target_languages: [
        { id: 'tl1', project_id: PROJECT_ID, source_lang: 'en', target_lang: 'es-ES', created_at: '2026-01-01T00:00:00Z' },
        { id: 'tl2', project_id: PROJECT_ID, source_lang: 'en', target_lang: 'fr-FR', created_at: '2026-01-01T00:00:00Z' },
      ],
    };

    vi.spyOn(workspaceClient, 'getProjectDetail').mockResolvedValueOnce({ ok: true, value: detail });

    render(<ProjectDetailScreen orgId={ORG_ID} projectId={PROJECT_ID} />);

    await waitFor(() => {
      expect(screen.getAllByTestId('asset-row')).toHaveLength(2);
    });

    expect(screen.getByText('Trailer')).toBeInTheDocument();
    expect(screen.getByText('Intro')).toBeInTheDocument();
    expect(screen.getAllByTestId('language-row')).toHaveLength(2);
    expect(screen.getByText('en → es-ES')).toBeInTheDocument();
    expect(screen.getByText('en → fr-FR')).toBeInTheDocument();
  });
});

describe('ProjectDetailScreen — EC-1: empty project', () => {
  it('shows empty-state sections without errors', async () => {
    vi.spyOn(workspaceClient, 'getProjectDetail').mockResolvedValueOnce({
      ok: true,
      value: { ...BASE_DETAIL, assets: [], target_languages: [] },
    });

    render(<ProjectDetailScreen orgId={ORG_ID} projectId={PROJECT_ID} />);

    await waitFor(() => screen.getByText('Alpha'));

    expect(screen.getByText('No assets linked.')).toBeInTheDocument();
    expect(screen.getByText('No target languages declared.')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});

describe('ProjectDetailScreen — error path', () => {
  it('shows error message on API failure', async () => {
    vi.spyOn(workspaceClient, 'getProjectDetail').mockResolvedValueOnce({
      ok: false,
      error: { kind: 'forbidden' },
    });

    render(<ProjectDetailScreen orgId={ORG_ID} projectId={PROJECT_ID} />);

    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeInTheDocument();
    });
  });
});
