import { useEffect, useState } from 'react';
import { addMember, listMembers, type AddMemberRequest, type OrgMember } from '../api/workspaceClient';

type Props = {
  orgId: string;
  /** The role of the currently-authenticated user in this org. */
  viewerRole: OrgMember['role'];
};

const WRITE_ROLES: OrgMember['role'][] = ['Owner', 'Admin'];

function canWrite(role: OrgMember['role']): boolean {
  return WRITE_ROLES.includes(role);
}

export function MembersScreen({ orgId, viewerRole }: Props) {
  const [members, setMembers] = useState<OrgMember[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // Add-member form state (only relevant for owner/admin)
  const [subjectId, setSubjectId] = useState('');
  const [role, setRole] = useState<OrgMember['role']>('Viewer');
  const [addError, setAddError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    listMembers(orgId).then((result) => {
      if (result.ok) {
        setMembers(result.value);
      } else {
        setError('Failed to load members');
      }
      setLoading(false);
    });
  }, [orgId]);

  async function handleAddMember(e: React.FormEvent) {
    e.preventDefault();
    setAddError(null);
    const req: AddMemberRequest = { subject_id: subjectId, role };
    const result = await addMember(orgId, req);
    if (result.ok) {
      setMembers((prev) => [...prev, result.value]);
      setSubjectId('');
      setRole('Viewer');
    } else {
      setAddError('Failed to add member');
    }
  }

  return (
    <div data-testid="members-screen">
      {loading && <p>Loading members…</p>}
      {error && <p role="alert">{error}</p>}
      {!loading && !error && members.length === 0 && (
        <p>No members yet.</p>
      )}
      {members.map((member) => (
        <div key={member.subject_id} data-testid="member-row">
          {member.subject_id} — {member.role}
        </div>
      ))}

      {canWrite(viewerRole) && (
        <form onSubmit={handleAddMember} aria-label="Add member">
          {addError && <p role="alert">{addError}</p>}
          <input
            aria-label="Subject ID"
            value={subjectId}
            onChange={(e) => setSubjectId(e.target.value)}
            placeholder="Subject ID"
            required
          />
          <select
            aria-label="Role"
            value={role}
            onChange={(e) => setRole(e.target.value as OrgMember['role'])}
          >
            <option value="Owner">Owner</option>
            <option value="Admin">Admin</option>
            <option value="Editor">Editor</option>
            <option value="Reviewer">Reviewer</option>
            <option value="Viewer">Viewer</option>
          </select>
          <button type="submit" data-testid="add-member-btn">Add Member</button>
        </form>
      )}
    </div>
  );
}
