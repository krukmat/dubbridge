import { useCallback, useEffect, useRef, useState } from "react";
import { RefreshControl, StyleSheet, Text, View } from "react-native";

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
import { Badge, statusTone } from "../components/Badge";
import { Card } from "../components/Card";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";

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

type ViewState =
  | { kind: "loading" }
  | {
      kind: "ready";
      tasks: ReviewTaskSummary[];
      unreadCount: number;
      notificationMessage: string | null;
    }
  | { kind: "empty"; unreadCount: number; notificationMessage: string | null }
  | { kind: "error"; message: string };

type ReviewInboxScreenProps = {
  gatewayBaseUrl: string;
  initialTaskId?: string | null;
  onOpenTask: (task: ReviewTaskSummary) => void;
};

function stateLabel(state: ReviewTaskSummary["state"]): string {
  if (state === "approved") return "Approved";
  if (state === "rejected") return "Rejected";
  return "Pending";
}

function compareTasks(a: ReviewTaskSummary, b: ReviewTaskSummary): number {
  return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
}

function notificationErrorMessage(notificationError: { kind: "forbidden" | "network" | "http"; detail?: string | number }): string {
  if (notificationError.kind === "forbidden") {
    return "Notifications are unavailable for this account.";
  }
  if (notificationError.kind === "network") {
    return String(notificationError.detail ?? "Network request failed.");
  }
  return `Notifications request failed with status ${notificationError.detail}.`;
}

export function ReviewInboxScreen({
  gatewayBaseUrl,
  initialTaskId = null,
  onOpenTask,
}: ReviewInboxScreenProps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });
  const [refreshing, setRefreshing] = useState(false);
  const initialTaskHandledRef = useRef<string | null>(null);

  const load = useCallback(async (): Promise<void> => {
    const client = createGatewayClient({ gatewayBaseUrl });

    const orgResult = await client.get<OrganizationSummary[]>("/api/orgs", auth.sessionRef);
    if (!orgResult.ok) {
      if (orgResult.error.kind === "session_expired") {
        await auth.logout();
        return;
      }
      const message =
        orgResult.error.kind === "network"
          ? orgResult.error.message
          : orgResult.error.kind === "forbidden"
            ? "You do not have access to the review queue."
            : `Could not load review scopes (${orgResult.error.status}).`;
      setViewState({ kind: "error", message });
      return;
    }

    await auth.onSessionRotation(orgResult.value.sessionRotation);

    const accessibleOrganizations = orgResult.value.data.filter(
      (organization) => organization.viewer_role !== "viewer",
    );

    const tasks: ReviewTaskSummary[] = [];
    for (const organization of accessibleOrganizations) {
      const projectResult = await client.get<ProjectSummary[]>(
        `/api/orgs/${organization.id}/projects`,
        auth.sessionRef,
      );

      if (!projectResult.ok) {
        if (projectResult.error.kind === "session_expired") {
          await auth.logout();
          return;
        }
        if (projectResult.error.kind === "forbidden") {
          continue;
        }
        const message =
          projectResult.error.kind === "network"
            ? projectResult.error.message
            : `Could not load review scopes (${projectResult.error.status}).`;
        setViewState({ kind: "error", message });
        return;
      }

      await auth.onSessionRotation(projectResult.value.sessionRotation);

      for (const project of projectResult.value.data) {
        const queueResult = await listReviewQueueForScope(
          client,
          auth.sessionRef,
          organization.id,
          project.id,
        );

        if (!queueResult.ok) {
          if (queueResult.error.kind === "session_expired") {
            await auth.logout();
            return;
          }
          if (queueResult.error.kind === "forbidden") {
            continue;
          }
          const message =
            queueResult.error.kind === "network"
              ? queueResult.error.message
              : `Could not load review queue (${queueResult.error.status}).`;
          setViewState({ kind: "error", message });
          return;
        }

        await auth.onSessionRotation(queueResult.value.sessionRotation);
        tasks.push(...queueResult.value.data.tasks);
      }
    }

    const sortedTasks = [...tasks].sort(compareTasks);

    const notifResult = await listNotifications(client, auth.sessionRef);
    let unreadCount = 0;
    let notificationMessage: string | null = null;
    let unreadNotifications: NotificationItem[] = [];

    if (!notifResult.ok) {
      if (notifResult.error.kind === "session_expired") {
        await auth.logout();
        return;
      }
      notificationMessage = notificationErrorMessage({
        kind: notifResult.error.kind,
        detail:
          notifResult.error.kind === "network"
            ? notifResult.error.message
            : notifResult.error.kind === "http"
              ? notifResult.error.status
              : undefined,
      });
    } else {
      await auth.onSessionRotation(notifResult.value.sessionRotation);
      unreadNotifications = notifResult.value.data.notifications.filter(
        (notification) =>
          notification.ref_entity_type === "review_task" && notification.read_at === null,
      );
      unreadCount = unreadNotifications.length;
    }

    if (unreadNotifications.length > 0) {
      const markReadResult = await markNotificationsRead(
        client,
        auth.sessionRef,
        unreadNotifications.map((notification) => notification.id),
      );
      if (!markReadResult.ok) {
        if (markReadResult.error.kind === "session_expired") {
          await auth.logout();
          return;
        }
        if (notificationMessage === null) {
          notificationMessage = notificationErrorMessage({
            kind: markReadResult.error.kind,
            detail:
              markReadResult.error.kind === "network"
                ? markReadResult.error.message
                : markReadResult.error.kind === "http"
                  ? markReadResult.error.status
                  : undefined,
          });
        }
      } else {
        await auth.onSessionRotation(markReadResult.value.sessionRotation);
      }
    }

    if (
      initialTaskId !== null &&
      initialTaskHandledRef.current !== initialTaskId
    ) {
      const matchedTask = sortedTasks.find((task) => task.id === initialTaskId);
      initialTaskHandledRef.current = initialTaskId;
      if (matchedTask) {
        onOpenTask(matchedTask);
        return;
      }
      notificationMessage =
        notificationMessage ?? "The referenced review task is no longer available.";
    }

    if (sortedTasks.length === 0) {
      setViewState({ kind: "empty", unreadCount, notificationMessage });
    } else {
      setViewState({
        kind: "ready",
        tasks: sortedTasks,
        unreadCount,
        notificationMessage,
      });
    }
  }, [auth, gatewayBaseUrl, initialTaskId, onOpenTask]);

  useEffect(() => {
    void load();
  }, [load]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await load();
    setRefreshing(false);
  }, [load]);

  const unreadCopy =
    viewState.kind === "ready" || viewState.kind === "empty"
      ? viewState.unreadCount > 0
        ? `${viewState.unreadCount} unread notification${viewState.unreadCount === 1 ? "" : "s"}`
        : undefined
      : undefined;

  const notificationMessage =
    viewState.kind === "ready" || viewState.kind === "empty"
      ? viewState.notificationMessage
      : null;

  return (
    <Screen
      testID="review-inbox-screen"
      scroll
      edges={["bottom"]}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={() => void onRefresh()} />
      }
    >
      <ScreenHeader
        kicker="Review"
        title="Review inbox"
        copy={unreadCopy}
      />

      {notificationMessage ? (
        <Text
          testID="review-notification-message"
          style={styles.notificationMessage}
          accessibilityRole="alert"
          accessibilityLiveRegion="polite"
        >
          {notificationMessage}
        </Text>
      ) : null}

      {viewState.kind === "loading" ? (
        <StateView kind="loading" title="Loading review queue…" />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load review queue"
          message={viewState.message}
          onRetry={() => void load()}
        />
      ) : null}

      {viewState.kind === "empty" ? (
        <StateView
          kind="empty"
          title="No tasks assigned"
          message="You have no review tasks at the moment."
        />
      ) : null}

      {viewState.kind === "ready"
        ? viewState.tasks.map((task) => (
            <Card
              key={task.id}
              testID={`review-task-card-${task.id}`}
              onPress={() => onOpenTask(task)}
              accessibilityLabel={`Review task ${task.id}, state ${stateLabel(task.state)}`}
            >
              <View style={styles.cardRow}>
                <Text style={styles.taskId} numberOfLines={1}>
                  Task {task.id.slice(0, 8)}
                </Text>
                <Badge label={stateLabel(task.state)} tone={statusTone(task.state)} />
              </View>
              <Text style={styles.meta}>Asset {task.asset_id.slice(0, 8)}</Text>
              <Text style={styles.meta}>
                Project {task.project_id.slice(0, 8)} · Updated{" "}
                {new Date(task.updated_at).toLocaleDateString()}
              </Text>
            </Card>
          ))
        : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  cardRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    gap: space.sm,
  },
  taskId: { ...type.heading, color: color.ink900, flex: 1 },
  meta: { ...type.meta, color: color.ink400 },
  notificationMessage: { ...type.meta, color: color.ink500 },
});
