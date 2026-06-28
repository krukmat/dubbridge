import React, { useCallback, useEffect, useRef, useState } from "react";

import { createGatewayClient } from "../api/client";
import {
  listNotifications,
  markNotificationsRead,
  type NotificationItem,
} from "../api/notifications";
import {
  type ReviewTaskSummary,
  listReviewQueueForScope,
} from "../api/review";
import { useAuth } from "../auth/AuthProvider";

type OrganizationSummary = {
  id: string;
  name: string;
  viewer_role: "owner" | "admin" | "editor" | "reviewer" | "viewer";
};

type ProjectSummary = {
  id: string;
  org_id: string;
  name: string;
};

export type ViewState =
  | { kind: "loading" }
  | {
      kind: "ready";
      tasks: ReviewTaskSummary[];
      unreadCount: number;
      notificationMessage: string | null;
    }
  | { kind: "empty"; unreadCount: number; notificationMessage: string | null }
  | { kind: "error"; message: string };

type ProjectOutcome =
  | { kind: "ok"; org: OrganizationSummary; projects: ProjectSummary[]; sessionRotation: string | null }
  | { kind: "session_expired" }
  | { kind: "forbidden" }
  | { kind: "error"; message: string };

type QueueOutcome =
  | { kind: "ok"; tasks: ReviewTaskSummary[]; sessionRotation: string | null }
  | { kind: "session_expired" }
  | { kind: "forbidden" }
  | { kind: "error"; message: string };

const CONCURRENCY_CAP = 3;

async function concurrentMap<T, R>(
  items: T[],
  cap: number,
  fn: (item: T) => Promise<R>,
): Promise<R[]> {
  if (items.length === 0) return [];
  const results: R[] = new Array(items.length);
  let next = 0;
  async function worker(): Promise<void> {
    while (next < items.length) {
      const i = next++;
      results[i] = await fn(items[i]);
    }
  }
  await Promise.all(Array.from({ length: Math.min(cap, items.length) }, worker));
  return results;
}

function compareTasks(a: ReviewTaskSummary, b: ReviewTaskSummary): number {
  return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
}

function notificationErrorMessage(kind: "forbidden" | "network" | "http", detail?: string | number): string {
  if (kind === "forbidden") return "Notifications are unavailable for this account.";
  if (kind === "network") return String(detail ?? "Network request failed.");
  return `Notifications request failed with status ${detail}.`;
}

type LoaderDeps = {
  gatewayBaseUrl: string;
  initialTaskId?: string | null;
  onOpenTask: (task: ReviewTaskSummary) => void;
};

type GatewayClient = ReturnType<typeof createGatewayClient>;
type Auth = ReturnType<typeof useAuth>;

async function fetchProjectsForOrg(
  client: GatewayClient,
  auth: Auth,
  org: OrganizationSummary,
): Promise<ProjectOutcome> {
  const result = await client.get<ProjectSummary[]>(`/api/orgs/${org.id}/projects`, auth.sessionRef);
  if (result.ok) return { kind: "ok", org, projects: result.value.data, sessionRotation: result.value.sessionRotation };
  if (result.error.kind === "session_expired") return { kind: "session_expired" };
  if (result.error.kind === "forbidden") return { kind: "forbidden" };
  const message = result.error.kind === "network" ? result.error.message : `Could not load review scopes (${result.error.status}).`;
  return { kind: "error", message };
}

async function fetchOrgsAndProjects(
  client: GatewayClient,
  auth: Auth,
): Promise<{ pairs: { org: OrganizationSummary; project: ProjectSummary }[] } | { abort: true }> {
  const orgResult = await client.get<OrganizationSummary[]>("/api/orgs", auth.sessionRef);
  if (!orgResult.ok) {
    if (orgResult.error.kind === "session_expired") { await auth.logout(); return { abort: true }; }
    return { abort: true };
  }
  await auth.onSessionRotation(orgResult.value.sessionRotation);

  const accessibleOrgs = orgResult.value.data.filter((o) => o.viewer_role !== "viewer");

  const projectOutcomes = await concurrentMap<OrganizationSummary, ProjectOutcome>(
    accessibleOrgs,
    CONCURRENCY_CAP,
    (org) => fetchProjectsForOrg(client, auth, org),
  );

  const pairs: { org: OrganizationSummary; project: ProjectSummary }[] = [];
  for (const outcome of projectOutcomes) {
    if (outcome.kind === "session_expired") { await auth.logout(); return { abort: true }; }
    if (outcome.kind === "forbidden") continue;
    if (outcome.kind === "error") return { abort: true };
    await auth.onSessionRotation(outcome.sessionRotation);
    for (const project of outcome.projects) pairs.push({ org: outcome.org, project });
  }
  return { pairs };
}

async function fetchQueueForPair(
  client: GatewayClient,
  auth: Auth,
  org: OrganizationSummary,
  project: ProjectSummary,
): Promise<QueueOutcome> {
  const result = await listReviewQueueForScope(client, auth.sessionRef, org.id, project.id);
  if (result.ok) return { kind: "ok", tasks: result.value.data.tasks, sessionRotation: result.value.sessionRotation };
  if (result.error.kind === "session_expired") return { kind: "session_expired" };
  if (result.error.kind === "forbidden") return { kind: "forbidden" };
  const message = result.error.kind === "network" ? result.error.message : `Could not load review queue (${result.error.status}).`;
  return { kind: "error", message };
}

async function fetchQueues(
  client: GatewayClient,
  auth: Auth,
  pairs: { org: OrganizationSummary; project: ProjectSummary }[],
): Promise<{ tasks: ReviewTaskSummary[] } | { abort: true }> {
  const queueOutcomes = await concurrentMap<{ org: OrganizationSummary; project: ProjectSummary }, QueueOutcome>(
    pairs,
    CONCURRENCY_CAP,
    ({ org, project }) => fetchQueueForPair(client, auth, org, project),
  );

  const tasks: ReviewTaskSummary[] = [];
  for (const outcome of queueOutcomes) {
    if (outcome.kind === "session_expired") { await auth.logout(); return { abort: true }; }
    if (outcome.kind === "forbidden") continue;
    if (outcome.kind === "error") return { abort: true };
    await auth.onSessionRotation(outcome.sessionRotation);
    tasks.push(...outcome.tasks);
  }
  return { tasks };
}

async function fetchNotifications(
  client: GatewayClient,
  auth: Auth,
): Promise<{ unreadCount: number; notificationMessage: string | null; unreadIds: string[] } | { abort: true }> {
  const notifResult = await listNotifications(client, auth.sessionRef);
  if (!notifResult.ok) {
    if (notifResult.error.kind === "session_expired") { await auth.logout(); return { abort: true }; }
    const detail = notifResult.error.kind === "network" ? notifResult.error.message : notifResult.error.kind === "http" ? notifResult.error.status : undefined;
    return { unreadCount: 0, notificationMessage: notificationErrorMessage(notifResult.error.kind, detail), unreadIds: [] };
  }
  await auth.onSessionRotation(notifResult.value.sessionRotation);
  const unread: NotificationItem[] = notifResult.value.data.notifications.filter(
    (n) => n.ref_entity_type === "review_task" && n.read_at === null,
  );
  return { unreadCount: unread.length, notificationMessage: null, unreadIds: unread.map((n) => n.id) };
}

async function markRead(
  client: GatewayClient,
  auth: Auth,
  ids: string[],
): Promise<string | null> {
  if (ids.length === 0) return null;
  const result = await markNotificationsRead(client, auth.sessionRef, ids);
  if (!result.ok) {
    if (result.error.kind === "session_expired") { await auth.logout(); return null; }
    const detail = result.error.kind === "network" ? result.error.message : result.error.kind === "http" ? result.error.status : undefined;
    return notificationErrorMessage(result.error.kind, detail);
  }
  await auth.onSessionRotation(result.value.sessionRotation);
  return null;
}

function resolveNotifMessage(base: string | null, markReadError: string | null): string | null {
  if (markReadError !== null && base === null) return markReadError;
  return base;
}

type InitialTaskResult = { handled: true; navigated: boolean; message: string | null };

function checkInitialTask(
  tasks: ReviewTaskSummary[],
  initialTaskId: string,
  handledRef: React.MutableRefObject<string | null>,
  currentMessage: string | null,
  onOpenTask: (task: ReviewTaskSummary) => void,
): InitialTaskResult {
  if (handledRef.current === initialTaskId) return { handled: false, navigated: false, message: currentMessage } as unknown as InitialTaskResult;
  const match = tasks.find((t) => t.id === initialTaskId);
  handledRef.current = initialTaskId;
  if (match) { onOpenTask(match); return { handled: true, navigated: true, message: currentMessage }; }
  return { handled: true, navigated: false, message: currentMessage ?? "The referenced review task is no longer available." };
}

export function useReviewInboxLoader({ gatewayBaseUrl, initialTaskId = null, onOpenTask }: LoaderDeps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });
  const [refreshing, setRefreshing] = useState(false);
  const initialTaskHandledRef = useRef<string | null>(null);

  const load = useCallback(async (): Promise<void> => {
    const client = createGatewayClient({ gatewayBaseUrl });

    const orgsResult = await fetchOrgsAndProjects(client, auth);
    if ("abort" in orgsResult) return;

    const queuesResult = await fetchQueues(client, auth, orgsResult.pairs);
    if ("abort" in queuesResult) return;

    const sortedTasks = [...queuesResult.tasks].sort(compareTasks);

    const notifResult = await fetchNotifications(client, auth);
    if ("abort" in notifResult) return;

    const markReadError = await markRead(client, auth, notifResult.unreadIds);
    let notificationMessage = resolveNotifMessage(notifResult.notificationMessage, markReadError);

    if (initialTaskId !== null) {
      const r = checkInitialTask(sortedTasks, initialTaskId, initialTaskHandledRef, notificationMessage, onOpenTask);
      if (r.navigated) return;
      notificationMessage = r.message;
    }

    if (sortedTasks.length === 0) {
      setViewState({ kind: "empty", unreadCount: notifResult.unreadCount, notificationMessage });
    } else {
      setViewState({ kind: "ready", tasks: sortedTasks, unreadCount: notifResult.unreadCount, notificationMessage });
    }
  }, [auth, gatewayBaseUrl, initialTaskId, onOpenTask]);

  useEffect(() => { void load(); }, [load]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await load();
    setRefreshing(false);
  }, [load]);

  const unreadCopy =
    (viewState.kind === "ready" || viewState.kind === "empty") && viewState.unreadCount > 0
      ? `${viewState.unreadCount} unread notification${viewState.unreadCount === 1 ? "" : "s"}`
      : undefined;

  const notificationMessage =
    viewState.kind === "ready" || viewState.kind === "empty"
      ? viewState.notificationMessage
      : null;

  return { viewState, refreshing, onRefresh, load, unreadCopy, notificationMessage };
}
