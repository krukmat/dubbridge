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

  it("T2/HP-1: rendered inbox card contains no raw ISO timestamp and no mid-token id cut", async () => {
    const LONG_TASK = {
      ...REVIEW_TASK,
      id: "task-seed-longid",
      asset_id: "asset-seed-longid",
      project_id: "project-seed-longid",
      updated_at: "2026-06-13T08:00:00Z",
    };
    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return { ok: true, value: { data: [{ id: "org-001", name: "Acme", viewer_role: "reviewer" }], sessionRotation: null } };
        case "/api/orgs/org-001/projects":
          return { ok: true, value: { data: [{ id: "proj-001", org_id: "org-001", name: "Trailer" }], sessionRotation: null } };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return { ok: true, value: { data: { org_id: "org-001", project_id: "proj-001", tasks: [LONG_TASK] }, sessionRotation: null } };
        case "/api/notifications":
          return { ok: true, value: { data: { notifications: [] }, sessionRotation: null } };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    await render(
      <ReviewInboxScreen gatewayBaseUrl="http://gateway" onOpenTask={jest.fn()} />,
    );
    await waitFor(() => expect(screen.getByTestId(`review-task-card-${LONG_TASK.id}`)).toBeTruthy());

    const card = screen.getByTestId(`review-task-card-${LONG_TASK.id}`);
    const cardText = JSON.stringify(card);
    // No raw ISO pattern.
    expect(cardText).not.toMatch(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z/);
    // Full ids must appear in the card (proves they were not truncated mid-token).
    expect(cardText).toContain("task-seed-longid");
    expect(cardText).toContain("asset-seed-longid");
    expect(cardText).toContain("project-seed-longid");
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

  it("T7b/HP-1: tasks from two orgs and multiple projects are all aggregated", async () => {
    const TASK_A = { ...REVIEW_TASK, id: "task-a", project_id: "proj-001" };
    const TASK_B = { ...REVIEW_TASK, id: "task-b", project_id: "proj-002" };
    const TASK_C = { ...REVIEW_TASK, id: "task-c", org_id: "org-002", project_id: "proj-003" };

    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return { ok: true, value: { data: [
            { id: "org-001", name: "Acme", viewer_role: "reviewer" },
            { id: "org-002", name: "Beta", viewer_role: "reviewer" },
          ], sessionRotation: null } };
        case "/api/orgs/org-001/projects":
          return { ok: true, value: { data: [
            { id: "proj-001", org_id: "org-001", name: "Alpha" },
            { id: "proj-002", org_id: "org-001", name: "Bravo" },
          ], sessionRotation: null } };
        case "/api/orgs/org-002/projects":
          return { ok: true, value: { data: [
            { id: "proj-003", org_id: "org-002", name: "Gamma" },
          ], sessionRotation: null } };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return { ok: true, value: { data: { org_id: "org-001", project_id: "proj-001", tasks: [TASK_A] }, sessionRotation: null } };
        case "/api/orgs/org-001/projects/proj-002/review-tasks":
          return { ok: true, value: { data: { org_id: "org-001", project_id: "proj-002", tasks: [TASK_B] }, sessionRotation: null } };
        case "/api/orgs/org-002/projects/proj-003/review-tasks":
          return { ok: true, value: { data: { org_id: "org-002", project_id: "proj-003", tasks: [TASK_C] }, sessionRotation: null } };
        case "/api/notifications":
          return { ok: true, value: { data: { notifications: [] }, sessionRotation: null } };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    await render(<ReviewInboxScreen gatewayBaseUrl="http://gateway" onOpenTask={jest.fn()} />);
    await waitFor(() => expect(screen.getByTestId("review-task-card-task-a")).toBeTruthy());
    expect(screen.getByTestId("review-task-card-task-b")).toBeTruthy();
    expect(screen.getByTestId("review-task-card-task-c")).toBeTruthy();
  });

  it("T7b/EC-1: forbidden project scope is skipped; other projects still contribute tasks", async () => {
    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return { ok: true, value: { data: [{ id: "org-001", name: "Acme", viewer_role: "reviewer" }], sessionRotation: null } };
        case "/api/orgs/org-001/projects":
          return { ok: true, value: { data: [
            { id: "proj-001", org_id: "org-001", name: "Forbidden" },
            { id: "proj-002", org_id: "org-001", name: "Allowed" },
          ], sessionRotation: null } };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return { ok: false, error: { kind: "forbidden" } };
        case "/api/orgs/org-001/projects/proj-002/review-tasks":
          return { ok: true, value: { data: { org_id: "org-001", project_id: "proj-002", tasks: [REVIEW_TASK] }, sessionRotation: null } };
        case "/api/notifications":
          return { ok: true, value: { data: { notifications: [] }, sessionRotation: null } };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    await render(<ReviewInboxScreen gatewayBaseUrl="http://gateway" onOpenTask={jest.fn()} />);
    await waitFor(() => expect(screen.getByTestId(`review-task-card-${REVIEW_TASK.id}`)).toBeTruthy());
  });

  it("T7b/EC-2: session_expired during queue fetch logs out exactly once", async () => {
    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return { ok: true, value: { data: [{ id: "org-001", name: "Acme", viewer_role: "reviewer" }], sessionRotation: null } };
        case "/api/orgs/org-001/projects":
          return { ok: true, value: { data: [{ id: "proj-001", org_id: "org-001", name: "Trailer" }], sessionRotation: null } };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          return { ok: false, error: { kind: "session_expired" } };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    await render(<ReviewInboxScreen gatewayBaseUrl="http://gateway" onOpenTask={jest.fn()} />);
    await waitFor(() => expect(mockAuth.logout).toHaveBeenCalledTimes(1));
    expect(mockAuth.logout).toHaveBeenCalledTimes(1);
  });

  it("T7b/EC-3: second project queue request starts before slow first project resolves", async () => {
    let proj1CallCount = 0;
    let proj2CallCount = 0;
    let resolveProj1!: (v: unknown) => void;

    mockClient.get.mockImplementation(async (path: string) => {
      switch (path) {
        case "/api/orgs":
          return { ok: true, value: { data: [{ id: "org-001", name: "Acme", viewer_role: "reviewer" }], sessionRotation: null } };
        case "/api/orgs/org-001/projects":
          return { ok: true, value: { data: [
            { id: "proj-001", org_id: "org-001", name: "Slow" },
            { id: "proj-002", org_id: "org-001", name: "Fast" },
          ], sessionRotation: null } };
        case "/api/orgs/org-001/projects/proj-001/review-tasks":
          proj1CallCount++;
          return new Promise((res) => { resolveProj1 = res; });
        case "/api/orgs/org-001/projects/proj-002/review-tasks":
          proj2CallCount++;
          return { ok: true, value: { data: { org_id: "org-001", project_id: "proj-002", tasks: [] }, sessionRotation: null } };
        case "/api/notifications":
          return { ok: true, value: { data: { notifications: [] }, sessionRotation: null } };
        default:
          throw new Error(`Unexpected GET ${path}`);
      }
    });

    render(<ReviewInboxScreen gatewayBaseUrl="http://gateway" onOpenTask={jest.fn()} />);

    // Both queue requests must be in-flight concurrently before the first resolves.
    await waitFor(() => {
      expect(proj1CallCount).toBe(1);
      expect(proj2CallCount).toBe(1);
    });

    // Resolve the slow first project and let the screen settle.
    resolveProj1({ ok: true, value: { data: { org_id: "org-001", project_id: "proj-001", tasks: [] }, sessionRotation: null } });
    await waitFor(() => expect(screen.getByText("No tasks assigned")).toBeTruthy());
  });
});
