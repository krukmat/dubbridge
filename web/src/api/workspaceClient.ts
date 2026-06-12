import { gatewayClient } from './gatewayClient';
import type { GatewayResult } from './types';

export type ProjectSummary = {
  id: string;
  org_id: string;
  name: string;
  created_at: string;
  updated_at: string;
};

export type AssetSummary = {
  id: string;
  title: string;
  uploader_id: string;
  status: string;
  created_at: string;
  updated_at: string;
};

export type TargetLanguage = {
  id: string;
  project_id: string;
  source_lang: string;
  target_lang: string;
  created_at: string;
};

export type ProjectDetail = ProjectSummary & {
  assets: AssetSummary[];
  target_languages: TargetLanguage[];
};

export type OrgMember = {
  org_id: string;
  subject_id: string;
  role: 'Owner' | 'Admin' | 'Editor' | 'Reviewer' | 'Viewer';
  joined_at: string;
};

export type AddMemberRequest = {
  subject_id: string;
  role: OrgMember['role'];
};

export function listProjects(orgId: string): Promise<GatewayResult<ProjectSummary[]>> {
  return gatewayClient.get<ProjectSummary[]>(`/api/orgs/${orgId}/projects`);
}

export function getProjectDetail(orgId: string, projectId: string): Promise<GatewayResult<ProjectDetail>> {
  return gatewayClient.get<ProjectDetail>(`/api/orgs/${orgId}/projects/${projectId}`);
}

export function listMembers(orgId: string): Promise<GatewayResult<OrgMember[]>> {
  return gatewayClient.get<OrgMember[]>(`/api/orgs/${orgId}/members`);
}

export function addMember(orgId: string, req: AddMemberRequest): Promise<GatewayResult<OrgMember>> {
  return gatewayClient.post<OrgMember>(`/api/orgs/${orgId}/members`, req);
}
