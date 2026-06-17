import { fireEvent, render, screen, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { ReviewInboxScreen } from "../src/screens/ReviewInboxScreen";

jest.mock("../src/auth/AuthProvider", () => ({ useAuth: () => mockAuth }));
jest.mock("../src/api/client", () => ({ createGatewayClient: jest.fn() }));
jest.mock("../src/components/Screen", () => {
  const React = require("react");
  const { View } = require("react-native");
  return {
    Screen: ({ children, ...props }: any) => {
      mockScreenProps = props;
      return React.createElement(View, props, children);
    },
  };
});
jest.mock("../src/components/Badge", () => {
  const React = require("react");
  const { Text } = require("react-native");
  const actual = jest.requireActual("../src/components/Badge");
  return {
    ...actual,
    Badge: ({ label, tone, testID }: any) => {
      mockBadgeToneCalls.push(tone);
      return React.createElement(Text, { testID }, label);
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

const REVIEW_TASK = {
  id: "task-001",
  org_id: "org-001",
  project_id: "proj-001",
  asset_id: "asset-001",
  target_language_id: "lang-001",
  assignee_subject_id: "reviewer-001",
  state: "pending" as const,
  created_at: "2026-06-13T00:00:00Z",
  updated_at: "2026-06-13T08:00:00Z",
  assigned_at: "2026-06-13T00:00:00Z",
};

let mockAuth: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };
let mockScreenProps: Record<string, unknown> | undefined;
let mockBadgeToneCalls: string[];

describe("ReviewInboxScreen", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockScreenProps = undefined;
    mockBadgeToneCalls = [];
    mockAuth = {
      sessionRef: "opaque-session",
      status: "authed",
      loginError: null,
      login: jest.fn().mockResolvedValue(undefined),
      logout: jest.fn().mockResolvedValue(undefined),
      onSessionRotation: jest.fn().mockResolvedValue(undefined),
    };
    mockClient = { get: jest.fn(), post: jest.fn(), postMultipart: jest.fn() };
    mockCreateGatewayClient.mockReturnValue(
      mockClient as unknown as ReturnType<typeof createGatewayClient>,
    );
  });

  it("HP-1: aggregates review tasks, shows unread badge, and marks notifications read", async () => {
    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return {
            ok: true,
            value: {
              data: [
                { id: "org-001", name: "Acme", viewer_role: "reviewer" },
                { id: "org-002", name: "Ignored", viewer_role: "viewer" },
              ],
              sessionRotation: "rot-orgs",
            },
          };
        case "/api/orgs/org-001/projects":
          return {
            ok: true,
            value: {
              data: [{ id: "proj-001", org_id: "org-001", name: "Trailer" }],
              sessionRotation: "rot-projects",
            },
          };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return {
            ok: true,
            value: {
              data: { org_id: "org-001", project_id: "proj-001", tasks: [REVIEW_TASK] },
              sessionRotation: "rot-queue",
            },
          };
        case "/api/notifications":
          return {
            ok: true,
            value: {
              data: {
                notifications: [
                  {
                    id: "notif-001",
                    kind: "review_task_assigned",
                    ref_entity_type: "review_task",
                    ref_entity_id: REVIEW_TASK.id,
                    actor_subject_id: null,
                    read_at: null,
                    created_at: "2026-06-13T09:00:00Z",
                  },
                ],
              },
              sessionRotation: "rot-notifs",
            },
          };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });
    mockClient.post.mockResolvedValueOnce({
      ok: true,
      value: { data: undefined, sessionRotation: "rot-mark-read" },
    });

    const onOpenTask = jest.fn();
    await render(
      <ReviewInboxScreen
        gatewayBaseUrl="http://gateway"
        onOpenTask={onOpenTask}
      />,
    );

    await waitFor(() => expect(screen.getByTestId(`review-task-card-${REVIEW_TASK.id}`)).toBeTruthy());
    expect(screen.getByText("1 unread notification")).toBeTruthy();
    expect(mockScreenProps?.edges).toEqual(["bottom"]);
    expect(mockBadgeToneCalls).toContain("info");
    expect(mockClient.post).toHaveBeenCalledWith(
      "/api/notifications/mark-read",
      "opaque-session",
      { ids: ["notif-001"] },
    );

    fireEvent.press(screen.getByTestId(`review-task-card-${REVIEW_TASK.id}`));

    expect(onOpenTask).toHaveBeenCalledWith(REVIEW_TASK);
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-orgs");
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-projects");
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-queue");
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-notifs");
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rot-mark-read");
  });

  it("HP-3: initialTaskId opens the matching task after queue resolution", async () => {
    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return {
            ok: true,
            value: {
              data: [{ id: "org-001", name: "Acme", viewer_role: "reviewer" }],
              sessionRotation: null,
            },
          };
        case "/api/orgs/org-001/projects":
          return {
            ok: true,
            value: {
              data: [{ id: "proj-001", org_id: "org-001", name: "Trailer" }],
              sessionRotation: null,
            },
          };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return {
            ok: true,
            value: {
              data: { org_id: "org-001", project_id: "proj-001", tasks: [REVIEW_TASK] },
              sessionRotation: null,
            },
          };
        case "/api/notifications":
          return {
            ok: true,
            value: {
              data: { notifications: [] },
              sessionRotation: null,
            },
          };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    const onOpenTask = jest.fn();
    await render(
      <ReviewInboxScreen
        gatewayBaseUrl="http://gateway"
        initialTaskId={REVIEW_TASK.id}
        onOpenTask={onOpenTask}
      />,
    );

    await waitFor(() => expect(onOpenTask).toHaveBeenCalledWith(REVIEW_TASK));
  });

  it("EC-2: session_expired on review-scope discovery logs out immediately", async () => {
    mockClient.get.mockResolvedValueOnce({
      ok: false,
      error: { kind: "session_expired" },
    });

    await render(
      <ReviewInboxScreen
        gatewayBaseUrl="http://gateway"
        onOpenTask={jest.fn()}
      />,
    );

    await waitFor(() => expect(mockAuth.logout).toHaveBeenCalledTimes(1));
  });

  it("HP-2: review badge tones follow the shared semantic mapping used by the screen", async () => {
    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return {
            ok: true,
            value: {
              data: [{ id: "org-001", name: "Acme", viewer_role: "reviewer" }],
              sessionRotation: null,
            },
          };
        case "/api/orgs/org-001/projects":
          return {
            ok: true,
            value: {
              data: [{ id: "proj-001", org_id: "org-001", name: "Trailer" }],
              sessionRotation: null,
            },
          };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return {
            ok: true,
            value: {
              data: {
                org_id: "org-001",
                project_id: "proj-001",
                tasks: [
                  { ...REVIEW_TASK, id: "task-pending", state: "pending" as const },
                  { ...REVIEW_TASK, id: "task-approved", state: "approved" as const },
                  { ...REVIEW_TASK, id: "task-rejected", state: "rejected" as const },
                  { ...REVIEW_TASK, id: "task-unknown", state: "unknown" as any },
                ],
              },
              sessionRotation: null,
            },
          };
        case "/api/notifications":
          return {
            ok: true,
            value: {
              data: { notifications: [] },
              sessionRotation: null,
            },
          };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    await render(
      <ReviewInboxScreen
        gatewayBaseUrl="http://gateway"
        onOpenTask={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-task-card-task-approved")).toBeTruthy());
    expect(mockBadgeToneCalls).toEqual(
      expect.arrayContaining(["info", "success", "danger", "neutral"]),
    );
  });

  it("EC-1: notification errors are announced accessibly", async () => {
    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return {
            ok: true,
            value: {
              data: [{ id: "org-001", name: "Acme", viewer_role: "reviewer" }],
              sessionRotation: null,
            },
          };
        case "/api/orgs/org-001/projects":
          return {
            ok: true,
            value: {
              data: [{ id: "proj-001", org_id: "org-001", name: "Trailer" }],
              sessionRotation: null,
            },
          };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return {
            ok: true,
            value: {
              data: { org_id: "org-001", project_id: "proj-001", tasks: [REVIEW_TASK] },
              sessionRotation: null,
            },
          };
        case "/api/notifications":
          return {
            ok: false,
            error: { kind: "network", message: "network failure" },
          };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    await render(
      <ReviewInboxScreen
        gatewayBaseUrl="http://gateway"
        onOpenTask={jest.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByTestId("review-notification-message")).toBeTruthy());
    expect(screen.getByTestId("review-notification-message").props.accessibilityRole).toBe("alert");
    expect(screen.getByTestId("review-notification-message").props.accessibilityLiveRegion).toBe("polite");
  });
});
