import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as workspaceClient from '../api/workspaceClient';
import { MembersScreen } from './MembersScreen';

const ORG_ID = 'org-1';

const MEMBER_OWNER = { org_id: ORG_ID, subject_id: 'user-owner', role: 'Owner' as const, joined_at: '2026-01-01T00:00:00Z' };
const MEMBER_VIEWER = { org_id: ORG_ID, subject_id: 'user-viewer', role: 'Viewer' as const, joined_at: '2026-01-01T00:00:00Z' };

beforeEach(() => {
  vi.restoreAllMocks();
});

describe('MembersScreen — data-testid', () => {
  it('renders the members-screen testid', async () => {
    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({ ok: true, value: [] });
    render(<MembersScreen orgId={ORG_ID} viewerRole="Owner" />);
    await waitFor(() => expect(screen.getByTestId('members-screen')).toBeInTheDocument());
  });
});

describe('MembersScreen — HP-1: owner/admin sees add-member form (SC-MEMBER-1)', () => {
  it('shows add-member button when viewerRole is Owner', async () => {
    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({ ok: true, value: [MEMBER_OWNER] });

    render(<MembersScreen orgId={ORG_ID} viewerRole="Owner" />);

    await waitFor(() => screen.getByTestId('add-member-btn'));
    expect(screen.getByTestId('add-member-btn')).toBeInTheDocument();
  });

  it('shows add-member button when viewerRole is Admin', async () => {
    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({ ok: true, value: [] });

    render(<MembersScreen orgId={ORG_ID} viewerRole="Admin" />);

    await waitFor(() => screen.getByTestId('add-member-btn'));
    expect(screen.getByTestId('add-member-btn')).toBeInTheDocument();
  });

  it('submits the add-member form and appends the new member', async () => {
    const newMember = { org_id: ORG_ID, subject_id: 'user-new', role: 'Reviewer' as const, joined_at: '2026-06-12T00:00:00Z' };

    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({ ok: true, value: [MEMBER_OWNER] });
    const addSpy = vi.spyOn(workspaceClient, 'addMember').mockResolvedValueOnce({ ok: true, value: newMember });

    render(<MembersScreen orgId={ORG_ID} viewerRole="Owner" />);

    await waitFor(() => screen.getByTestId('add-member-btn'));

    await userEvent.type(screen.getByLabelText('Subject ID'), 'user-new');
    await userEvent.selectOptions(screen.getByLabelText('Role'), 'Reviewer');
    await userEvent.click(screen.getByTestId('add-member-btn'));

    expect(addSpy).toHaveBeenCalledWith(ORG_ID, { subject_id: 'user-new', role: 'Reviewer' });

    await waitFor(() => {
      expect(screen.getAllByTestId('member-row')).toHaveLength(2);
    });
  });
});

describe('MembersScreen — EC-2: viewer role hides add-member control (SC-MEMBER-1)', () => {
  it('does not render add-member button when viewerRole is Viewer', async () => {
    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({ ok: true, value: [MEMBER_VIEWER] });

    render(<MembersScreen orgId={ORG_ID} viewerRole="Viewer" />);

    await waitFor(() => screen.getByTestId('member-row'));

    expect(screen.queryByTestId('add-member-btn')).not.toBeInTheDocument();
  });

  it('does not render add-member button for Editor or Reviewer roles', async () => {
    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({ ok: true, value: [] });

    const { rerender } = render(<MembersScreen orgId={ORG_ID} viewerRole="Editor" />);
    await waitFor(() => expect(screen.queryByText('Loading members…')).not.toBeInTheDocument());
    expect(screen.queryByTestId('add-member-btn')).not.toBeInTheDocument();

    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({ ok: true, value: [] });
    rerender(<MembersScreen orgId={ORG_ID} viewerRole="Reviewer" />);
    await waitFor(() => expect(screen.queryByText('Loading members…')).not.toBeInTheDocument());
    expect(screen.queryByTestId('add-member-btn')).not.toBeInTheDocument();
  });
});

describe('MembersScreen — member list rendering', () => {
  it('renders one row per member', async () => {
    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({
      ok: true,
      value: [MEMBER_OWNER, MEMBER_VIEWER],
    });

    render(<MembersScreen orgId={ORG_ID} viewerRole="Owner" />);

    await waitFor(() => {
      expect(screen.getAllByTestId('member-row')).toHaveLength(2);
    });
  });
});

describe('MembersScreen — error path', () => {
  it('shows error message on list API failure', async () => {
    vi.spyOn(workspaceClient, 'listMembers').mockResolvedValueOnce({
      ok: false,
      error: { kind: 'http', status: 500 },
    });

    render(<MembersScreen orgId={ORG_ID} viewerRole="Owner" />);

    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeInTheDocument();
    });
  });
});
