import { FlatList, RefreshControl, StyleSheet, Text, View } from "react-native";

import { formatId, formatStatusLabel, formatTimestamp } from "../format";

import { type ReviewTaskSummary } from "../api/review";
import { Badge, statusTone } from "../components/Badge";
import { Card } from "../components/Card";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";
import { useReviewInboxLoader } from "./useReviewInboxLoader";

type ReviewInboxScreenProps = {
  gatewayBaseUrl: string;
  initialTaskId?: string | null;
  onOpenTask: (task: ReviewTaskSummary) => void;
};

function ReviewTaskCard({ task, onPress }: { task: ReviewTaskSummary; onPress: () => void }) {
  return (
    <Card
      testID={`review-task-card-${task.id}`}
      onPress={onPress}
      accessibilityLabel={`Review task ${task.id}, state ${formatStatusLabel(task.state)}`}
      trailing="chevron"
    >
      <View style={styles.cardRow}>
        <Text style={styles.taskId} numberOfLines={1}>Task {formatId(task.id)}</Text>
        <Badge label={formatStatusLabel(task.state)} tone={statusTone(task.state)} />
      </View>
      <Text style={styles.meta}>Asset {formatId(task.asset_id)}</Text>
      <Text style={styles.meta}>
        Project {formatId(task.project_id)} · Updated {formatTimestamp(task.updated_at)}
      </Text>
    </Card>
  );
}

type ReviewTaskListProps = {
  tasks: ReviewTaskSummary[];
  isEmpty: boolean;
  refreshing: boolean;
  onRefresh: () => Promise<void>;
  onOpenTask: (task: ReviewTaskSummary) => void;
};

function ReviewTaskList({ tasks, isEmpty, refreshing, onRefresh, onOpenTask }: ReviewTaskListProps) {
  return (
    <FlatList
      style={styles.scroll}
      contentContainerStyle={isEmpty ? styles.emptyContent : styles.listContent}
      data={tasks}
      keyExtractor={(task) => task.id}
      renderItem={({ item: task }) => (
        <ReviewTaskCard task={task} onPress={() => onOpenTask(task)} />
      )}
      ListEmptyComponent={
        <StateView kind="empty" title="No tasks assigned" message="You have no review tasks at the moment." />
      }
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={() => void onRefresh()} />
      }
    />
  );
}

export function ReviewInboxScreen(props: ReviewInboxScreenProps) {
  const { viewState, refreshing, onRefresh, load, unreadCopy, notificationMessage } = useReviewInboxLoader(props);

  return (
    <Screen testID="review-inbox-screen">
      <ScreenHeader kicker="Review" title="Review inbox" copy={unreadCopy} />

      {notificationMessage ? (
        <Text testID="review-notification-message" style={styles.notificationMessage} accessibilityRole="alert" accessibilityLiveRegion="polite">
          {notificationMessage}
        </Text>
      ) : null}

      {viewState.kind === "loading" ? <StateView kind="loading" title="Loading review queue…" /> : null}

      {viewState.kind === "error" ? (
        <StateView kind="error" title="Could not load review queue" message={viewState.message} onRetry={() => void load()} />
      ) : null}

      {(viewState.kind === "ready" || viewState.kind === "empty") ? (
        <ReviewTaskList
          tasks={viewState.kind === "ready" ? viewState.tasks : []}
          isEmpty={viewState.kind === "empty"}
          refreshing={refreshing}
          onRefresh={onRefresh}
          onOpenTask={props.onOpenTask}
        />
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  scroll: { flex: 1 },
  listContent: { gap: space.md, paddingBottom: space.xl },
  emptyContent: { flexGrow: 1 },
  cardRow: { flexDirection: "row", justifyContent: "space-between", alignItems: "center", gap: space.sm },
  taskId: { ...type.heading, color: color.ink900, flex: 1 },
  meta: { ...type.meta, color: color.ink400 },
  notificationMessage: { ...type.meta, color: color.ink500 },
});
