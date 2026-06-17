import type { GatewayClient, GatewayResult } from './client';

export type ReviewTaskSummary = {
  id: string;
  org_id: string;
  project_id: string;
  asset_id: string;
  target_language_id: string;
  assignee_subject_id: string | null;
  state: 'pending' | 'approved' | 'rejected';
  created_at: string;
  updated_at: string;
  assigned_at: string | null;
};

export type DecisionVerdict = 'approved' | 'rejected';

export type ReviewDecisionRequest = {
  verdict: DecisionVerdict;
  comment: string | null;
};

export type ReviewDecisionResponse = {
  review_task_id: string;
  state: string;
};

export type PublishResponse = {
  review_task_id: string;
  status: string;
  published_by: string;
  published_at: string;
};

export type ReviewQueueResponse = {
  org_id: string;
  project_id: string;
  tasks: ReviewTaskSummary[];
};

export function listReviewQueueForScope(
  client: GatewayClient,
  sessionRef: string | null,
  orgId: string,
  projectId: string,
): Promise<GatewayResult<ReviewQueueResponse>> {
  return client.get<ReviewQueueResponse>(
    `/api/orgs/${orgId}/projects/${projectId}/review-tasks`,
    sessionRef,
  );
}

export function postDecision(
  client: GatewayClient,
  sessionRef: string | null,
  task: Pick<ReviewTaskSummary, 'id' | 'org_id' | 'project_id'>,
  body: ReviewDecisionRequest,
): Promise<GatewayResult<ReviewDecisionResponse>> {
  return client.post<ReviewDecisionResponse>(
    `/api/orgs/${task.org_id}/projects/${task.project_id}/review-tasks/${task.id}/decision`,
    sessionRef,
    body,
  );
}

export function publishTask(
  client: GatewayClient,
  sessionRef: string | null,
  task: Pick<ReviewTaskSummary, 'id' | 'org_id' | 'project_id'>,
): Promise<GatewayResult<PublishResponse>> {
  return client.post<PublishResponse>(
    `/api/orgs/${task.org_id}/projects/${task.project_id}/review-tasks/${task.id}/publish`,
    sessionRef,
    {},
  );
}
