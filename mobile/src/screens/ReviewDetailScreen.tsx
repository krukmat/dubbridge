import { useEffect, useState } from "react";
import { StyleSheet, Text, TextInput, View } from "react-native";

import { formatId, formatTimestamp } from "../format";

import { createGatewayClient } from "../api/client";
import {
  type ReviewTaskSummary,
  postDecision,
  publishTask,
} from "../api/review";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { PlaybackStateView } from "../components/PlaybackStateView";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { usePlaybackLoader } from "../hooks/usePlaybackLoader";

import { color, fieldStyle, radius, space, type } from "../theme";

type ReviewDetailScreenProps = {
  task: ReviewTaskSummary;
  gatewayBaseUrl: string;
  onBack: () => void;
};

type MutationState =
  | { kind: "idle" }
  | { kind: "submitting" }
  | { kind: "error"; message: string };


export function ReviewDetailScreen({
  task,
  gatewayBaseUrl,
  onBack,
}: ReviewDetailScreenProps) {
  const auth = useAuth();
  const [taskState, setTaskState] = useState(task.state);
  const [comment, setComment] = useState("");
  const [publishedAt, setPublishedAt] = useState<string | null>(null);
  const [mutation, setMutation] = useState<MutationState>({ kind: "idle" });
  const [playbackAttempt, setPlaybackAttempt] = useState(0);
  const playbackState = usePlaybackLoader({
    assetId: task.asset_id,
    gatewayBaseUrl,
    attempt: playbackAttempt,
  });

  useEffect(() => {
    setTaskState(task.state);
    setComment("");
    setPublishedAt(null);
    setMutation({ kind: "idle" });
  }, [task.id, task.state]);

  async function decide(verdict: "approved" | "rejected"): Promise<void> {
    if (mutation.kind === "submitting") return;
    setMutation({ kind: "submitting" });

    const client = createGatewayClient({ gatewayBaseUrl });
    const normalizedComment = comment.trim();
    const result = await postDecision(client, auth.sessionRef, task, {
      verdict,
      comment: normalizedComment.length > 0 ? normalizedComment : null,
    });

    if (!result.ok) {
      if (result.error.kind === "session_expired") {
        await auth.logout();
        return;
      }
      const message =
        result.error.kind === "forbidden"
          ? "Insufficient role to submit a decision."
          : result.error.kind === "network"
            ? result.error.message
            : `Could not submit decision (${result.error.status}).`;
      setMutation({ kind: "error", message });
      return;
    }

    await auth.onSessionRotation(result.value.sessionRotation);
    setTaskState(result.value.data.state as ReviewTaskSummary["state"]);
    setComment("");
    setPublishedAt(null);
    setMutation({ kind: "idle" });
  }

  async function publish(): Promise<void> {
    if (mutation.kind === "submitting") return;
    setMutation({ kind: "submitting" });

    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await publishTask(client, auth.sessionRef, task);

    if (!result.ok) {
      if (result.error.kind === "session_expired") {
        await auth.logout();
        return;
      }
      const message =
        result.error.kind === "forbidden"
          ? "Insufficient role to publish."
          : result.error.kind === "network"
            ? result.error.message
            : `Could not publish (${result.error.status}).`;
      setMutation({ kind: "error", message });
      return;
    }

    await auth.onSessionRotation(result.value.sessionRotation);
    setPublishedAt(result.value.data.published_at);
    setMutation({ kind: "idle" });
  }

  const isSubmitting = mutation.kind === "submitting";

  return (
    <Screen testID="review-detail-screen" scroll edges={["bottom"]}>
      <ScreenHeader kicker="Review" title="Review task" />

      <Panel>
        <View style={styles.row}>
          <Text style={styles.label}>Task ID</Text>
          <Text style={styles.value} numberOfLines={1} ellipsizeMode="tail">
            {formatId(task.id)}
          </Text>
        </View>
        <View style={styles.row}>
          <Text style={styles.label}>Asset</Text>
          <Text style={styles.value} numberOfLines={1} ellipsizeMode="tail">
            {formatId(task.asset_id)}
          </Text>
        </View>
        <View style={styles.row}>
          <Text style={styles.label}>State</Text>
          <Badge
            label={taskState.charAt(0).toUpperCase() + taskState.slice(1)}
            tone={statusTone(taskState)}
          />
        </View>
      </Panel>

      <Panel>
        <Text style={styles.sectionTitle}>Playback</Text>
        <Text style={styles.body}>
          Watch the original track before submitting the review decision.
        </Text>
        <PlaybackStateView
          state={playbackState}
          testIdPrefix="review-player"
          onRetry={() => setPlaybackAttempt((attempt) => attempt + 1)}
        />
      </Panel>

      <Panel>
        <Text style={styles.sectionTitle}>Original vs. Derived</Text>
        <View style={styles.comparisonStack}>
          <View style={styles.comparisonPanel}>
            <Text style={styles.comparisonHeading}>Original</Text>
            <Text style={styles.comparisonBody}>Asset {formatId(task.asset_id)}</Text>
            <Text style={styles.comparisonMeta}>Created {formatTimestamp(task.created_at)}</Text>
          </View>
          <View style={styles.comparisonPanel}>
            <Text style={styles.comparisonHeading}>Derived</Text>
            <Text style={styles.comparisonBody}>Target {formatId(task.target_language_id)}</Text>
            <Text style={styles.comparisonMeta}>
              Scope {formatId(task.org_id)} / {formatId(task.project_id)}
            </Text>
          </View>
        </View>
      </Panel>

      <Panel>
        <Text style={styles.sectionTitle}>Decision</Text>
        <TextInput
          testID="review-comment-input"
          accessibilityLabel="Comment"
          value={comment}
          onChangeText={setComment}
          placeholder="Add a comment…"
          multiline
          numberOfLines={3}
          style={[fieldStyle, styles.commentInput]}
        />
        <View style={styles.actions}>
          <Button
            testID="review-approve"
            label="Approve"
            onPress={() => void decide("approved")}
            loading={isSubmitting}
            disabled={isSubmitting}
            fullWidth
            style={styles.actionButton}
          />
          <Button
            testID="review-reject"
            label="Reject"
            variant="danger"
            onPress={() => void decide("rejected")}
            loading={isSubmitting}
            disabled={isSubmitting}
            fullWidth
            style={styles.actionButton}
          />
        </View>
        {mutation.kind === "error" ? (
          <Text
            style={styles.errorText}
            accessibilityRole="alert"
            accessibilityLiveRegion="assertive"
          >
            {mutation.message}
          </Text>
        ) : null}
      </Panel>

      {taskState === "approved" ? (
        <Panel>
          <Text style={styles.sectionTitle}>Publication</Text>
          <Text style={styles.body}>
            This task is approved. You may publish the derived output.
          </Text>
          <Button
            testID="publish-action"
            label="Publish"
            onPress={() => void publish()}
            loading={isSubmitting}
            disabled={isSubmitting}
          />
          {publishedAt ? (
            <Text style={styles.body} accessibilityLiveRegion="polite">
              Published {new Date(publishedAt).toLocaleString()}
            </Text>
          ) : null}
        </Panel>
      ) : null}

      <Button
        label="Back to inbox"
        variant="secondary"
        onPress={onBack}
      />
    </Screen>
  );
}

const styles = StyleSheet.create({
  row: { flexDirection: "row", justifyContent: "space-between", alignItems: "center" },
  label: { ...type.label, color: color.ink400 },
  value: { ...type.meta, color: color.ink700, flex: 1, flexShrink: 1, textAlign: "right", marginLeft: space.md },
  sectionTitle: { ...type.heading, color: color.ink900 },
  comparisonStack: { gap: space.md },
  comparisonPanel: {
    gap: space.xs,
    padding: space.md,
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: color.border,
    backgroundColor: color.sunken,
  },
  comparisonHeading: { ...type.label, color: color.primary },
  comparisonBody: { ...type.bodyStrong, color: color.ink900 },
  comparisonMeta: { ...type.meta, color: color.ink500 },
  body: { ...type.body, color: color.ink500 },
  commentInput: { minHeight: space.xxxl * 2, textAlignVertical: "top" },
  actions: { flexDirection: "row", gap: space.sm },
  actionButton: { flex: 1 },
  errorText: { ...type.meta, color: color.danger },
});
