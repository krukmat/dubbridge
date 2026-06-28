import { useEffect, useState } from "react";

import { createGatewayClient } from "../api/client";
import { type ReviewTaskSummary, postDecision, publishTask } from "../api/review";
import { useAuth } from "../auth/AuthProvider";

type MutationState =
  | { kind: "idle" }
  | { kind: "submitting" }
  | { kind: "error"; message: string };

export type TaskState = "pending" | "approved" | "rejected";

type Auth = ReturnType<typeof useAuth>;

function decideErrorMessage(error: { kind: string; message?: string; status?: number }): string {
  if (error.kind === "forbidden") return "Insufficient role to submit a decision.";
  if (error.kind === "network") return error.message ?? "Network error.";
  return `Could not submit decision (${error.status}).`;
}

function publishErrorMessage(error: { kind: string; message?: string; status?: number }): string {
  if (error.kind === "forbidden") return "Insufficient role to publish.";
  if (error.kind === "network") return error.message ?? "Network error.";
  return `Could not publish (${error.status}).`;
}

async function submitDecision(
  auth: Auth,
  gatewayBaseUrl: string,
  task: ReviewTaskSummary,
  verdict: "approved" | "rejected",
  comment: string,
): Promise<{ state: TaskState } | { error: string } | { logout: true }> {
  const client = createGatewayClient({ gatewayBaseUrl });
  const normalized = comment.trim();
  const result = await postDecision(client, auth.sessionRef, task, {
    verdict,
    comment: normalized.length > 0 ? normalized : null,
  });
  if (!result.ok) {
    if (result.error.kind === "session_expired") { await auth.logout(); return { logout: true }; }
    return { error: decideErrorMessage(result.error) };
  }
  await auth.onSessionRotation(result.value.sessionRotation);
  const rawState = result.value.data.state;
  if (rawState === "pending" || rawState === "approved" || rawState === "rejected") {
    return { state: rawState };
  }
  console.warn(`[ReviewDetailScreen] unexpected task state from API: ${rawState}`);
  return { state: "pending" };
}

async function submitPublish(
  auth: Auth,
  gatewayBaseUrl: string,
  task: ReviewTaskSummary,
): Promise<{ publishedAt: string } | { error: string } | { logout: true }> {
  const client = createGatewayClient({ gatewayBaseUrl });
  const result = await publishTask(client, auth.sessionRef, task);
  if (!result.ok) {
    if (result.error.kind === "session_expired") { await auth.logout(); return { logout: true }; }
    return { error: publishErrorMessage(result.error) };
  }
  await auth.onSessionRotation(result.value.sessionRotation);
  return { publishedAt: result.value.data.published_at };
}

export function useReviewDetailMutations(task: ReviewTaskSummary, gatewayBaseUrl: string) {
  const auth = useAuth();
  const [taskState, setTaskState] = useState<TaskState>(task.state);
  const [comment, setComment] = useState("");
  const [publishedAt, setPublishedAt] = useState<string | null>(null);
  const [mutation, setMutation] = useState<MutationState>({ kind: "idle" });

  useEffect(() => {
    setTaskState(task.state);
    setComment("");
    setPublishedAt(null);
    setMutation({ kind: "idle" });
  }, [task.id, task.state]);

  async function decide(verdict: "approved" | "rejected"): Promise<void> {
    if (mutation.kind === "submitting") return;
    setMutation({ kind: "submitting" });
    const outcome = await submitDecision(auth, gatewayBaseUrl, task, verdict, comment);
    if ("logout" in outcome) return;
    if ("error" in outcome) { setMutation({ kind: "error", message: outcome.error }); return; }
    setTaskState(outcome.state);
    setComment("");
    setPublishedAt(null);
    setMutation({ kind: "idle" });
  }

  async function publish(): Promise<void> {
    if (mutation.kind === "submitting") return;
    setMutation({ kind: "submitting" });
    const outcome = await submitPublish(auth, gatewayBaseUrl, task);
    if ("logout" in outcome) return;
    if ("error" in outcome) { setMutation({ kind: "error", message: outcome.error }); return; }
    setPublishedAt(outcome.publishedAt);
    setMutation({ kind: "idle" });
  }

  return { taskState, comment, setComment, publishedAt, mutation, decide, publish };
}
