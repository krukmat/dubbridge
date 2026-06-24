import { useEffect, useEffectEvent, useState } from "react";
import { StyleSheet, Text, TextInput, View } from "react-native";

import { formatId, formatTimestamp } from "../format";

import { createGatewayClient } from "../api/client";
import { buildManifestUrl, issuePlaybackGrant } from "../api/playback";
import {
  type ReviewTaskSummary,
  postDecision,
  publishTask,
} from "../api/review";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { VideoPlayer } from "../components/VideoPlayer";
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

type PlaybackState =
  | { kind: "loading" }
  | { kind: "ready"; source: string }
  | { kind: "not_ready" }
  | { kind: "error"; message: string };

export function ReviewDetailScreen({
  task,
  gatewayBaseUrl,
  onBack,
}: ReviewDetailScreenProps) {
  const auth = useAuth();
  const handlePlaybackLogout = useEffectEvent(async () => {
    await auth.logout();
  });
  const handlePlaybackSessionRotation = useEffectEvent(async (rotation: string | null) => {
    await auth.onSessionRotation(rotation);
  });
  const [taskState, setTaskState] = useState(task.state);
  const [comment, setComment] = useState("");
  const [publishedAt, setPublishedAt] = useState<string | null>(null);
  const [mutation, setMutation] = useState<MutationState>({ kind: "idle" });
  const [playbackState, setPlaybackState] = useState<PlaybackState>({
    kind: "loading",
  });
  const [playbackAttempt, setPlaybackAttempt] = useState(0);

  useEffect(() => {
    setTaskState(task.state);
    setComment("");
    setPublishedAt(null);
    setMutation({ kind: "idle" });
  }, [task.id, task.state]);

  useEffect(() => {
    let isActive = true;

    async function loadPlayback(): Promise<void> {
      setPlaybackState({ kind: "loading" });

      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await issuePlaybackGrant(client, auth.sessionRef, task.asset_id);

      if (!isActive) {
        return;
      }

      if (!result.ok) {
        if (result.error.kind === "session_expired") {
          await handlePlaybackLogout();
          return;
        }

        if (result.error.kind === "http" && (result.error.status === 409 || result.error.status === 422)) {
          setPlaybackState({ kind: "not_ready" });
          return;
        }

        const message =
          result.error.kind === "forbidden"
            ? "You do not have access to this playback stream."
            : result.error.kind === "network"
              ? result.error.message
              : `Could not load playback (${result.error.status}).`;
        setPlaybackState({ kind: "error", message });
        return;
      }

      await handlePlaybackSessionRotation(result.value.sessionRotation);

      if (!isActive) {
        return;
      }

      setPlaybackState({
        kind: "ready",
        source: buildManifestUrl(gatewayBaseUrl, task.asset_id, result.value.data.grantId),
      });
    }

    void loadPlayback();

    return () => {
      isActive = false;
    };
  }, [auth.sessionRef, gatewayBaseUrl, playbackAttempt, task.asset_id, task.id]);

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
          <Text style={styles.value}>{task.id}</Text>
        </View>
        <View style={styles.row}>
          <Text style={styles.label}>Asset</Text>
          <Text style={styles.value}>{task.asset_id}</Text>
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
        {playbackState.kind === "loading" ? (
          <View style={styles.playbackSurface}>
            <StateView
              testID="review-player-loading"
              kind="loading"
              title="Loading playback…"
              message="Preparing the original track."
            />
          </View>
        ) : null}
        {playbackState.kind === "not_ready" ? (
          <View style={styles.playbackSurface}>
            <StateView
              testID="review-player-empty"
              kind="empty"
              title="Media not ready yet"
              message="Playback is not available for this asset yet."
            />
          </View>
        ) : null}
        {playbackState.kind === "error" ? (
          <View style={styles.playbackSurface}>
            <StateView
              testID="review-player-error"
              kind="error"
              title="Could not load playback"
              message={playbackState.message}
              onRetry={() => setPlaybackAttempt((attempt) => attempt + 1)}
            />
          </View>
        ) : null}
        {playbackState.kind === "ready" ? (
          <VideoPlayer
            testID="review-player"
            source={playbackState.source}
            onRetry={() => setPlaybackAttempt((attempt) => attempt + 1)}
          />
        ) : null}
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
  value: { ...type.meta, color: color.ink700, flex: 1, textAlign: "right" },
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
  playbackSurface: {
    minHeight: 220,
  },
  body: { ...type.body, color: color.ink500 },
  commentInput: { minHeight: space.xxxl * 2, textAlignVertical: "top" },
  actions: { flexDirection: "row", gap: space.sm },
  actionButton: { flex: 1 },
  errorText: { ...type.meta, color: color.danger },
});
