import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as workspaceClient from '../api/workspaceClient';
import { ProjectsScreen } from './ProjectsScreen';

const ORG_ID = 'org-1';

const PROJECT_A = { id: 'proj-a', org_id: ORG_ID, name: 'Alpha', created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z' };
const PROJECT_B = { id: 'proj-b', org_id: ORG_ID, name: 'Beta', created_at: '2026-01-02T00:00:00Z', updated_at: '2026-01-02T00:00:00Z' };

beforeEach(() => {
  vi.restoreAllMocks();
});

describe('ProjectsScreen — data-testid', () => {
  it('renders the projects-screen testid', async () => {
    vi.spyOn(workspaceClient, 'listProjects').mockResolvedValueOnce({ ok: true, value: [] });
    render(<ProjectsScreen orgId={ORG_ID} onSelectProject={vi.fn()} />);
    await waitFor(() => expect(screen.getByTestId('projects-screen')).toBeInTheDocument());
  });
});

describe('ProjectsScreen — HP-1: org with projects', () => {
  it('renders one row per project (SC-PROJECT-1)', async () => {
    vi.spyOn(workspaceClient, 'listProjects').mockResolvedValueOnce({
      ok: true,
      value: [PROJECT_A, PROJECT_B],
    });

    render(<ProjectsScreen orgId={ORG_ID} onSelectProject={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getAllByTestId('project-row')).toHaveLength(2);
    });

    expect(screen.getByText('Alpha')).toBeInTheDocument();
    expect(screen.getByText('Beta')).toBeInTheDocument();
  });

  it('calls onSelectProject with the project id when a row is clicked', async () => {
    vi.spyOn(workspaceClient, 'listProjects').mockResolvedValueOnce({
      ok: true,
      value: [PROJECT_A],
    });

    const onSelect = vi.fn();
    render(<ProjectsScreen orgId={ORG_ID} onSelectProject={onSelect} />);

    await waitFor(() => screen.getByText('Alpha'));
    await userEvent.click(screen.getByText('Alpha'));

    expect(onSelect).toHaveBeenCalledWith('proj-a');
  });
});

describe('ProjectsScreen — EC-1: empty org', () => {
  it('renders empty-state with no error when list is empty', async () => {
    vi.spyOn(workspaceClient, 'listProjects').mockResolvedValueOnce({ ok: true, value: [] });

    render(<ProjectsScreen orgId={ORG_ID} onSelectProject={vi.fn()} />);

    await waitFor(() => {
      expect(screen.queryByTestId('project-row')).not.toBeInTheDocument();
    });

    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(screen.getByText('No projects found.')).toBeInTheDocument();
  });
});

describe('ProjectsScreen — error path', () => {
  it('shows an error message on API failure', async () => {
    vi.spyOn(workspaceClient, 'listProjects').mockResolvedValueOnce({
      ok: false,
      error: { kind: 'http', status: 500 },
    });

    render(<ProjectsScreen orgId={ORG_ID} onSelectProject={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeInTheDocument();
    });
  });
});
