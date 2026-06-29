import { useState } from "react";
import { Pressable, StyleSheet, Text, TextInput, View } from "react-native";

import { formatId, formatRelative, formatStatusLabel, formatTimestamp } from "../format";

import { type ReviewTaskSummary } from "../api/review";
import { ActionBar, ACTION_BAR_CONTENT_HEIGHT } from "../components/ActionBar";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { PlaybackStateView } from "../components/PlaybackStateView";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { usePlaybackLoader } from "../hooks/usePlaybackLoader";
import { color, fieldStyle, radius, space, type } from "../theme";
import { type TaskState, useReviewDetailMutations } from "./useReviewDetailMutations";

type ReviewDetailScreenProps = {
  task: ReviewTaskSummary;
  gatewayBaseUrl: string;
  onBack: () => void;
};

function ReviewScopePanel({ task }: { task: ReviewTaskSummary }) {
  const [techExpanded, setTechExpanded] = useState(false);
  return (
    <Panel>
      <Text style={styles.sectionTitle}>Review scope</Text>
      <View style={styles.comparisonStack}>
        <View style={styles.comparisonPanel}>
          <Text style={styles.comparisonHeading}>Original track</Text>
          <Text style={styles.comparisonMeta}>Created {formatTimestamp(task.created_at)}</Text>
        </View>
        <View style={styles.comparisonPanel}>
          <Text style={styles.comparisonHeading}>Target language</Text>
          <Text style={styles.comparisonMeta}>{formatId(task.target_language_id)}</Text>
        </View>
      </View>
      <Pressable testID="review-tech-details-toggle" onPress={() => setTechExpanded((v) => !v)} accessibilityRole="button" accessibilityLabel="Technical details" accessibilityState={{ expanded: techExpanded }}>
        <Text style={styles.techToggle}>Technical details {techExpanded ? "▲" : "▼"}</Text>
      </Pressable>
      {techExpanded ? (
        <View testID="review-tech-details" style={styles.techGroup}>
          <Text style={styles.metaKey}>Asset ID</Text>
          <Text style={styles.metaVal} numberOfLines={1} ellipsizeMode="tail">{task.asset_id}</Text>
          <Text style={styles.metaKey}>Target language ID</Text>
          <Text style={styles.metaVal} numberOfLines={1} ellipsizeMode="tail">{task.target_language_id}</Text>
          <Text style={styles.metaKey}>Org / Project</Text>
          <Text style={styles.metaVal} numberOfLines={1} ellipsizeMode="tail">{formatId(task.org_id)} / {formatId(task.project_id)}</Text>
        </View>
      ) : null}
    </Panel>
  );
}

type ActionBarsProps = { taskState: TaskState; publishedAt: string | null; isSubmitting: boolean; decide: (v: "approved" | "rejected") => Promise<void>; publish: () => Promise<void> };

function ReviewActionBars({ taskState, publishedAt, isSubmitting, decide, publish }: ActionBarsProps) {
  if (taskState === "pending") {
    return (
      <ActionBar>
        <Button testID="review-approve" label="Approve" onPress={() => void decide("approved")} loading={isSubmitting} disabled={isSubmitting} fullWidth style={styles.actionButton} />
        <Button testID="review-reject" label="Reject" variant="danger" onPress={() => void decide("rejected")} loading={isSubmitting} disabled={isSubmitting} fullWidth style={styles.actionButton} />
      </ActionBar>
    );
  }
  if (taskState === "approved" && !publishedAt) {
    return (
      <ActionBar>
        <Button testID="publish-action" label="Publish" onPress={() => void publish()} loading={isSubmitting} disabled={isSubmitting} fullWidth />
      </ActionBar>
    );
  }
  return null;
}

function readinessLabel(taskState: TaskState, publishedAt: string | null): string | null {
  if (publishedAt) return null;
  if (taskState === "approved") return "Approved — awaiting publication";
  if (taskState === "rejected") return "Rejected";
  return "Pending review";
}

export function ReviewDetailScreen({ task, gatewayBaseUrl, onBack }: ReviewDetailScreenProps) {
  const { taskState, comment, setComment, publishedAt, mutation, decide, publish } =
    useReviewDetailMutations(task, gatewayBaseUrl);
  const [playbackAttempt, setPlaybackAttempt] = useState(0);
  const playbackState = usePlaybackLoader({ assetId: task.asset_id, gatewayBaseUrl, attempt: playbackAttempt });
  const isSubmitting = mutation.kind === "submitting";
  const actionBarHeight = ACTION_BAR_CONTENT_HEIGHT + space.md * 2;
  const readiness = readinessLabel(taskState, publishedAt);

  return (
    <View style={styles.container}>
      <Screen testID="review-detail-screen" scroll extraBottomPadding={actionBarHeight}>
        <ScreenHeader kicker="Review" title="Review task" />
        <Panel testID="review-editorial-summary">
          <View style={styles.row}>
            <Text style={styles.editorialLang} numberOfLines={1}>{formatId(task.target_language_id) || "Language TBD"}</Text>
            <Badge label={formatStatusLabel(taskState)} tone={statusTone(taskState)} />
          </View>
          <Text style={styles.editorialMeta}>
            Project {formatId(task.project_id)} · {formatRelative(task.updated_at)}
          </Text>
          {readiness ? (
            <Text style={styles.readinessLabel} testID="review-readiness-label">
              {readiness}
            </Text>
          ) : null}
          <Text style={styles.techIdMeta} numberOfLines={1} ellipsizeMode="tail">{formatId(task.id)}</Text>
          <Text style={styles.techIdMeta} numberOfLines={1} ellipsizeMode="tail">{formatId(task.asset_id)}</Text>
        </Panel>
        <Panel>
          <Text style={styles.sectionTitle}>Playback</Text>
          <Text style={styles.body}>Watch the original track before submitting the review decision.</Text>
          <PlaybackStateView state={playbackState} testIdPrefix="review-player" onRetry={() => setPlaybackAttempt((a) => a + 1)} />
        </Panel>
        <ReviewScopePanel task={task} />
        <Panel>
          <Text style={styles.sectionTitle}>Decision</Text>
          <TextInput testID="review-comment-input" accessibilityLabel="Comment" value={comment} onChangeText={setComment} placeholder="Add a comment…" multiline numberOfLines={3} style={[fieldStyle, styles.commentInput]} />
          {mutation.kind === "error" ? <Text style={styles.errorText} accessibilityRole="alert" accessibilityLiveRegion="assertive">{mutation.message}</Text> : null}
        </Panel>
        {taskState === "approved" && publishedAt ? (
          <Panel>
            <Text style={styles.sectionTitle}>Publication</Text>
            <Text style={styles.body} accessibilityLiveRegion="polite">Published {formatTimestamp(publishedAt)}</Text>
          </Panel>
        ) : null}
        {taskState === "approved" && !publishedAt ? (
          <Panel testID="review-publish-pending-panel">
            <Text style={styles.sectionTitle}>Publication</Text>
            <Text style={styles.body} testID="review-publish-pending-reason">Ready to publish — use the button below to make this content available.</Text>
          </Panel>
        ) : null}
        {taskState === "rejected" ? (
          <Panel testID="review-rejected-panel">
            <Text style={styles.sectionTitle}>Publication</Text>
            <Text style={styles.body} testID="review-rejected-reason">Not available for publication — this task was rejected.</Text>
          </Panel>
        ) : null}
        <Button label="Back to inbox" variant="secondary" onPress={onBack} />
      </Screen>
      <ReviewActionBars taskState={taskState} publishedAt={publishedAt} isSubmitting={isSubmitting} decide={decide} publish={publish} />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: color.canvas },
  row: { flexDirection: "row", justifyContent: "space-between", alignItems: "center" },
  label: { ...type.label, color: color.ink400 },
  value: { ...type.meta, color: color.ink700, flex: 1, flexShrink: 1, textAlign: "right", marginLeft: space.md },
  editorialLang: { ...type.heading, color: color.ink900, flex: 1 },
  editorialMeta: { ...type.meta, color: color.ink400, marginTop: space.xs },
  readinessLabel: { ...type.body, color: color.ink500, marginTop: space.xs },
  techIdMeta: { ...type.meta, color: color.ink300, marginTop: space.xs },
  sectionTitle: { ...type.heading, color: color.ink900 },
  comparisonStack: { gap: space.md },
  comparisonPanel: { gap: space.xs, padding: space.md, borderRadius: radius.lg, borderWidth: 1, borderColor: color.border, backgroundColor: color.sunken },
  comparisonHeading: { ...type.label, color: color.primary },
  comparisonMeta: { ...type.meta, color: color.ink500 },
  techToggle: { ...type.label, color: color.primary, marginTop: space.xs },
  techGroup: { gap: space.xs, marginTop: space.xs },
  metaKey: { ...type.label, color: color.ink400 },
  metaVal: { ...type.meta, color: color.ink700 },
  body: { ...type.body, color: color.ink500 },
  commentInput: { minHeight: space.xxxl * 2, textAlignVertical: "top" },
  actionButton: { flex: 1 },
  errorText: { ...type.meta, color: color.danger },
});
