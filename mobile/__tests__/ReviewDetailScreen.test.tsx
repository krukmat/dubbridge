import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { ReviewDetailScreen } from "../src/screens/ReviewDetailScreen";

jest.mock("../src/auth/AuthProvider", () => ({ useAuth: () => mockAuth }));
jest.mock("../src/api/client", () => ({ createGatewayClient: jest.fn() }));
jest.mock("../src/components/VideoPlayer", () => {
  const React = require("react");
  const { Text } = require("react-native");
  const fn = ({ testID, source, ...rest }: any) => {
    if (testID) { mockVideoPlayerProps[testID] = { testID, source, ...rest }; }
    return React.createElement(Text, { testID }, `Video:${source}`);
  };
  return { VideoPlayer: fn };
});
jest.mock("../src/components/Button", () => {
  const React = require("react");
  const { Pressable, Text } = require("react-native");
  return {
    Button: ({ testID, label, onPress, ...props }: any) => {
      if (testID) {
        mockButtonProps[testID] = { testID, label, onPress, ...props };
      }
      return React.createElement(
        Pressable,
        { testID, onPress },
        React.createElement(Text, null, label),
      );
    },
  };
});

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<
  typeof createGatewayClient
>;

const BASE_TASK = {
  id: "task-001",
  org_id: "org-001",
  project_id: "proj-001",
  asset_id: "asset-001",
  target_language_id: "lang-001",
  assignee_subject_id: "reviewer-001",
  state: "pending" as const,
  created_at: "2026-06-13T00:00:00Z",
  updated_at: "2026-06-13T00:00:00Z",
  assigned_at: "2026-06-13T00:00:00Z",
};

let mockAuth: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };
let mockButtonProps: Record<string, any>;
let mockVideoPlayerProps: Record<string, any>;

function playbackGrantSuccess(grantId = "grant-001") {
  return {
    ok: true,
    value: {
      data: { grant_id: grantId },
      sessionRotation: "rot-playback",
    },
  };
}

describe("ReviewDetailScreen", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockButtonProps = {};
    mockVideoPlayerProps = {};
    mockAuth = {
      sessionRef: "opaque-session",
      status: "authed",
      loginError: null,
      login: jest.fn().mockResolvedValue(undefined),
      logout: jest.fn().mockResolvedValue(undefined),
      onSessionRotation: jest.fn().mockResolvedValue(undefined),
    };
    mockClient = { get: jest.fn(), post: jest.fn(), postMultipart: jest.fn() };
    mockClient.post.mockResolvedValue(playbackGrantSuccess());
    mockCreateGatewayClient.mockReturnValue(
      mockClient as unknown as ReturnType<typeof createGatewayClient>,
    );
  });

  afterEach(() => {
    cleanup();
  });

  it("HP-1: approve posts a scoped decision, rotates session, and reveals publish", async () => {
    mockClient.post
      .mockResolvedValueOnce(playbackGrantSuccess())
      .mockResolvedValueOnce({
        ok: true,
        value: {
          data: { review_task_id: BASE_TASK.id, state: "approved" },
          sessionRotation: "rot-review",
        },
      });

    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    expect(mockButtonProps["review-approve"].fullWidth).toBe(true);
    expect(mockButtonProps["review-reject"].fullWidth).toBe(true);
    await waitFor(() => expect(screen.getByTestId("review-player")).toBeTruthy());
    await waitFor(() => expect(screen.getByTestId("review-detail-screen")).toBeTruthy());
    expect(mockVideoPlayerProps["review-player"].source).toBe(
      "http://gateway/api/assets/asset-001/playback/grant-001/manifest",
    );
    fireEvent(screen.getByTestId("review-comment-input"), "changeText", "Looks good");
    await waitFor(() =>
      expect(screen.getByTestId("review-comment-input").props.value).toBe("Looks good"),
    );
    await act(async () => {
      fireEvent.press(screen.getByTestId("review-approve"));
    });

    await waitFor(() => expect(screen.getByTestId("publish-action")).toBeTruthy());
    expect(mockClient.post).toHaveBeenCalledWith(
      "/api/assets/asset-001/playback-grants",
      "opaque-session",
      {},
    );
    expect(mockClient.post).toHaveBeenCalledWith(
      "/api/orgs/org-001/projects/proj-001/review-tasks/task-001/decision",
      "opaque-session",
      { verdict: "approved", comment: "Looks good" },
    );
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-playback");
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-review");
  });

  it("HP-2: approved task publishes and shows the published timestamp", async () => {
    mockClient.post
      .mockResolvedValueOnce(playbackGrantSuccess("grant-publish"))
      .mockResolvedValueOnce({
        ok: true,
        value: {
          data: {
            review_task_id: BASE_TASK.id,
            status: "published",
            published_by: "reviewer-001",
            published_at: "2026-06-13T10:00:00Z",
          },
          sessionRotation: "rot-publish",
        },
      });

    await render(
      <ReviewDetailScreen
        task={{ ...BASE_TASK, state: "approved" }}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("publish-action")).toBeTruthy());
    expect(screen.getByTestId("review-player")).toBeTruthy();
    await act(async () => {
      fireEvent.press(screen.getByTestId("publish-action"));
    });

    await waitFor(() => expect(screen.getByText(/Published/)).toBeTruthy());
    expect(screen.getByText(/Published/).props.accessibilityLiveRegion).toBe("polite");
    expect(mockClient.post).toHaveBeenCalledWith(
      "/api/orgs/org-001/projects/proj-001/review-tasks/task-001/publish",
      "opaque-session",
      {},
    );
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-publish");
  });

  it("EC-1: pending task hides the publish action", async () => {
    mockClient.post.mockResolvedValueOnce(playbackGrantSuccess("grant-pending"));

    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-player")).toBeTruthy());
    await waitFor(() => expect(screen.getByTestId("review-detail-screen")).toBeTruthy());
    expect(screen.queryByTestId("publish-action")).toBeNull();
  });

  it("EC-1b: same-task rerender with changed state keeps the already-loaded player visible", async () => {
    mockClient.post.mockResolvedValueOnce(playbackGrantSuccess("grant-rerender"));

    const view = await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-player")).toBeTruthy());

    await act(async () => {
      view.rerender(
        <ReviewDetailScreen
          task={{ ...BASE_TASK, state: "approved" }}
          gatewayBaseUrl="http://gateway"
          onBack={jest.fn()}
        />,
      );
    });

    await waitFor(() => expect(screen.getByTestId("review-player")).toBeTruthy());
    expect(screen.queryByTestId("review-player-loading")).toBeNull();
    expect(mockClient.post).toHaveBeenCalledTimes(1);
  });

  it("EC-2: session_expired on decision logs out immediately", async () => {
    mockClient.post
      .mockResolvedValueOnce(playbackGrantSuccess("grant-decision"))
      .mockResolvedValueOnce({
        ok: false,
        error: { kind: "session_expired" },
      });

    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-approve")).toBeTruthy());
    await act(async () => {
      fireEvent.press(screen.getByTestId("review-approve"));
    });

    await waitFor(() => expect(mockAuth.logout).toHaveBeenCalledTimes(1));
  });

  it("EC-3: forbidden publish shows an inline error without logout", async () => {
    mockClient.post
      .mockResolvedValueOnce(playbackGrantSuccess("grant-forbidden-publish"))
      .mockResolvedValueOnce({
        ok: false,
        error: { kind: "forbidden" },
      });

    await render(
      <ReviewDetailScreen
        task={{ ...BASE_TASK, state: "approved" }}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("publish-action")).toBeTruthy());
    await act(async () => {
      fireEvent.press(screen.getByTestId("publish-action"));
    });

    await waitFor(() =>
      expect(screen.getByText("Insufficient role to publish.")).toBeTruthy(),
    );
    expect(screen.getByText("Insufficient role to publish.").props.accessibilityRole).toBe("alert");
    expect(screen.getByText("Insufficient role to publish.").props.accessibilityLiveRegion).toBe("assertive");
    expect(mockAuth.logout).not.toHaveBeenCalled();
  });

  it("EC-3b: generic server error on publish shows inline error and is recoverable without logout", async () => {
    mockClient.post
      .mockResolvedValueOnce(playbackGrantSuccess("grant-publish-500"))
      .mockResolvedValueOnce({
        ok: false,
        error: { kind: "http", status: 500 },
      });

    await render(
      <ReviewDetailScreen
        task={{ ...BASE_TASK, state: "approved" }}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("publish-action")).toBeTruthy());
    await act(async () => {
      fireEvent.press(screen.getByTestId("publish-action"));
    });

    await waitFor(() =>
      expect(screen.getByText("Could not publish (500).")).toBeTruthy(),
    );
    expect(screen.getByText("Could not publish (500).").props.accessibilityRole).toBe("alert");
    // Publish button must still be present for retry
    expect(screen.getByTestId("publish-action")).toBeTruthy();
    expect(mockAuth.logout).not.toHaveBeenCalled();
  });

  it("EC-4: session_expired on playback grant logs out before showing a player", async () => {
    mockClient.post.mockResolvedValueOnce({
      ok: false,
      error: { kind: "session_expired" },
    });

    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(mockAuth.logout).toHaveBeenCalledTimes(1));
    expect(screen.queryByTestId("review-player")).toBeNull();
  });

  it("EC-5: playback denial shows a not-ready empty state and keeps decision controls usable", async () => {
    mockClient.post.mockResolvedValueOnce({
      ok: false,
      error: { kind: "http", status: 422 },
    });

    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByText("Media not ready yet")).toBeTruthy());
    expect(screen.getByTestId("review-approve")).toBeTruthy();
    expect(screen.getByTestId("review-reject")).toBeTruthy();
    expect(screen.queryByTestId("review-player")).toBeNull();
  });

  it("EC-6: playback failure shows an error state and keeps decision controls usable", async () => {
    mockClient.post.mockResolvedValueOnce({
      ok: false,
      error: { kind: "forbidden" },
    });

    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() =>
      expect(screen.getByText("You do not have access to this playback stream.")).toBeTruthy(),
    );
    expect(screen.getByTestId("review-approve")).toBeTruthy();
    expect(screen.getByTestId("review-reject")).toBeTruthy();
    expect(screen.getByTestId("review-player-error-retry")).toBeTruthy();
  });

  it("T2/HP-1: rendered detail panel contains no raw ISO timestamp; technical ids appear in full after expanding accordion", async () => {
    const LONG_TASK = {
      ...BASE_TASK,
      asset_id: "asset-seed-longid",
      target_language_id: "lang-seed-longid",
      org_id: "org-seed-longid",
      project_id: "project-seed-longid",
    };
    await render(
      <ReviewDetailScreen
        task={LONG_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-player")).toBeTruthy());
    await waitFor(() => expect(screen.getByTestId("review-approve")).toBeTruthy());

    // No raw ISO pattern visible anywhere in the tree.
    const treeBefore = JSON.stringify(screen.toJSON());
    expect(treeBefore).not.toMatch(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z/);

    // Expand the technical details accordion so all ids are rendered.
    await fireEvent.press(screen.getByTestId("review-tech-details-toggle"));

    const treeAfter = JSON.stringify(screen.toJSON());
    // Full ids must appear (proves they were not truncated mid-token).
    expect(treeAfter).toContain("asset-seed-longid");
    expect(treeAfter).toContain("lang-seed-longid");
    expect(treeAfter).toContain("org-seed-longid");
    expect(treeAfter).toContain("project-seed-longid");
  });

  it("HP-1b: summary-row ids use single-line tail ellipsis without dropping the full value", async () => {
    const LONG_TASK = {
      ...BASE_TASK,
      id: "review-task-seed-super-long-id",
      asset_id: "asset-seed-super-long-id",
    };

    await render(
      <ReviewDetailScreen
        task={LONG_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-player")).toBeTruthy());
    const summaryTaskId = screen.getAllByText("review-task-seed-super-long-id")[0];
    const summaryAssetId = screen.getAllByText("asset-seed-super-long-id")[0];
    expect(summaryTaskId.props.numberOfLines).toBe(1);
    expect(summaryTaskId.props.ellipsizeMode).toBe("tail");
    expect(summaryAssetId.props.numberOfLines).toBe(1);
    expect(summaryAssetId.props.ellipsizeMode).toBe("tail");
  });

  it("EC-3b: forbidden decision shows an inline error without logout", async () => {
    mockClient.post
      .mockResolvedValueOnce(playbackGrantSuccess("grant-decision-forbidden"))
      .mockResolvedValueOnce({
        ok: false,
        error: { kind: "forbidden" },
      });

    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-approve")).toBeTruthy());
    await act(async () => {
      fireEvent.press(screen.getByTestId("review-approve"));
    });

    await waitFor(() =>
      expect(screen.getByText("Insufficient role to submit a decision.")).toBeTruthy(),
    );
    expect(mockAuth.logout).not.toHaveBeenCalled();
  });

  it("T6/HP-1: editorial summary shows language, project, readiness label, and demoted ids", async () => {
    await render(
      <ReviewDetailScreen
        task={BASE_TASK}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-editorial-summary")).toBeTruthy());
    const summary = screen.getByTestId("review-editorial-summary");
    const summaryText = JSON.stringify(summary);

    // Language and project are visible in the editorial panel.
    expect(summaryText).toContain(BASE_TASK.target_language_id);
    expect(summaryText).toContain(BASE_TASK.project_id);
    // Readiness label present for pending state.
    expect(screen.getByTestId("review-readiness-label")).toBeTruthy();
    expect(screen.getByTestId("review-readiness-label").props.children).toBe("Pending review");
    // Ids appear in summary (demoted but visible).
    expect(summaryText).toContain(BASE_TASK.id);
    expect(summaryText).toContain(BASE_TASK.asset_id);
  });

  it("T6/HP-2: approved task shows explicit readiness label and publish-pending panel", async () => {
    await render(
      <ReviewDetailScreen
        task={{ ...BASE_TASK, state: "approved" }}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-publish-pending-panel")).toBeTruthy());
    expect(screen.getByTestId("review-readiness-label").props.children).toBe("Approved — awaiting publication");
    expect(screen.getByTestId("review-publish-pending-reason")).toBeTruthy();
  });

  it("T6/EC-1: missing target_language_id shows Language TBD fallback in editorial summary", async () => {
    await render(
      <ReviewDetailScreen
        task={{ ...BASE_TASK, target_language_id: "" }}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-editorial-summary")).toBeTruthy());
    expect(screen.getByText("Language TBD")).toBeTruthy();
    // Decision controls remain usable despite missing metadata.
    expect(screen.getByTestId("review-approve")).toBeTruthy();
    expect(screen.getByTestId("review-reject")).toBeTruthy();
  });

  it("T6/EC-2: rejected task shows explicit reason panel instead of silent button absence", async () => {
    await render(
      <ReviewDetailScreen
        task={{ ...BASE_TASK, state: "rejected" }}
        gatewayBaseUrl="http://gateway"
        onBack={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-rejected-panel")).toBeTruthy());
    expect(screen.getByTestId("review-rejected-reason")).toBeTruthy();
    expect(screen.queryByTestId("publish-action")).toBeNull();
    expect(screen.queryByTestId("review-approve")).toBeNull();
    expect(screen.queryByTestId("review-reject")).toBeNull();
  });
});
