import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { ReviewDetailScreen } from "../src/screens/ReviewDetailScreen";

jest.mock("../src/auth/AuthProvider", () => ({ useAuth: () => mockAuth }));
jest.mock("../src/api/client", () => ({ createGatewayClient: jest.fn() }));
jest.mock("../src/components/VideoPlayer", () => {
  const React = require("react");
  const { Text } = require("react-native");
  return {
    VideoPlayer: ({ testID, source, ...props }: any) => {
      if (testID) {
        mockVideoPlayerProps[testID] = { testID, source, ...props };
      }
      return React.createElement(Text, { testID }, `Video:${source}`);
    },
  };
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

  it("T2/HP-1: rendered detail panel contains no raw ISO timestamp and no mid-token id cut", async () => {
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
    const tree = JSON.stringify(screen.toJSON());
    // No raw ISO pattern.
    expect(tree).not.toMatch(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z/);
    // Full ids must appear (proves they were not truncated mid-token).
    expect(tree).toContain("asset-seed-longid");
    expect(tree).toContain("lang-seed-longid");
    expect(tree).toContain("org-seed-longid");
    expect(tree).toContain("project-seed-longid");
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
});
