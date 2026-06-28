import { act, cleanup, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import * as notifications from "../src/api/notifications";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { HomeScreen } from "../src/screens/HomeScreen";
import type { AssetSummary } from "../src/screens/AssetListScreen";

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

jest.mock("../src/auth/AuthProvider", () => ({
  useAuth: () => mockAuthValue,
}));

jest.mock("../src/api/client", () => ({
  createGatewayClient: jest.fn(),
}));

jest.mock("../src/api/notifications", () => ({
  listNotifications: jest.fn(),
}));

const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;
const mockListNotifications = notifications.listNotifications as jest.MockedFunction<typeof notifications.listNotifications>;

const ASSET_A: AssetSummary = {
  id: "asset-aaa",
  title: "Track Alpha",
  uploader_id: "user-1",
  status: "finalized",
  created_at: "2026-06-01T10:00:00Z",
  updated_at: "2026-06-01T10:00:00Z",
};

const ASSET_B: AssetSummary = {
  id: "asset-bbb",
  title: "Track Beta",
  uploader_id: "user-1",
  status: "in_review",
  created_at: "2026-06-02T10:00:00Z",
  updated_at: "2026-06-02T10:00:00Z",
};

let mockAuthValue: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };

const noop = () => {};

beforeEach(() => {
  mockClient = {
    get: jest.fn(),
    post: jest.fn(),
    postMultipart: jest.fn(),
  };
  mockCreateGatewayClient.mockReturnValue(mockClient as any);

  mockAuthValue = {
    sessionRef: "tok-abc",
    status: "authed",
    loginError: null,
    login: jest.fn(),
    logout: jest.fn(),
    onSessionRotation: jest.fn().mockResolvedValue(undefined),
  };
});

afterEach(cleanup);

async function renderHome() {
  return render(
    <HomeScreen
      dubbridgeEnv="local"
      gatewayBaseUrl="http://localhost:3000"
      onOpenAssets={noop}
      onOpenUpload={noop}
      onOpenReview={noop}
      onOpenOrganizations={noop}
    />,
  );
}

describe("HomeScreen", () => {
  it("HP-1: shows recent assets and pending review count when data loads", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [ASSET_A, ASSET_B], sessionRotation: null },
    });
    mockListNotifications.mockResolvedValue({
      ok: true,
      value: {
        data: {
          notifications: [
            { id: "n1", kind: "review", ref_entity_type: "review_task", ref_entity_id: "t1", actor_subject_id: null, read_at: null, created_at: "2026-06-01T00:00:00Z" },
            { id: "n2", kind: "review", ref_entity_type: "review_task", ref_entity_id: "t2", actor_subject_id: null, read_at: "2026-06-01T01:00:00Z", created_at: "2026-06-01T00:00:00Z" },
          ],
        },
        sessionRotation: null,
      },
    });

    const { getByTestId, getByText } = await renderHome();

    await waitFor(() => {
      expect(getByText("Track Alpha")).toBeTruthy();
    });

    expect(getByTestId("home-screen")).toBeTruthy();
    expect(getByTestId("home-recent-asset-asset-aaa")).toBeTruthy();
    expect(getByTestId("home-recent-asset-asset-bbb")).toBeTruthy();
    // 1 unread review_task notification
    expect(getByText("1 pending")).toBeTruthy();
    expect(getByTestId("home-pending-review-summary")).toBeTruthy();
    // Quick-action testIDs intact
    expect(getByTestId("home-open-assets")).toBeTruthy();
    expect(getByTestId("home-open-upload")).toBeTruthy();
    expect(getByTestId("home-open-review")).toBeTruthy();
    expect(getByTestId("home-open-organizations")).toBeTruthy();
    expect(getByTestId("home-account-card")).toBeTruthy();
    expect(getByTestId("home-account-icon")).toBeTruthy();
    expect(getByTestId("home-sign-out")).toBeTruthy();
    expect(getByText("Ready")).toBeTruthy();
    expect(getByText("In review")).toBeTruthy();
  });

  it("HP-2: no pending review tasks — pending summary card absent", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [ASSET_A], sessionRotation: null },
    });
    mockListNotifications.mockResolvedValue({
      ok: true,
      value: {
        data: {
          notifications: [
            { id: "n1", kind: "review", ref_entity_type: "review_task", ref_entity_id: "t1", actor_subject_id: null, read_at: "2026-06-01T01:00:00Z", created_at: "2026-06-01T00:00:00Z" },
          ],
        },
        sessionRotation: null,
      },
    });

    const { getByTestId, queryByTestId } = await renderHome();

    await waitFor(() => {
      expect(getByTestId("home-recent-asset-asset-aaa")).toBeTruthy();
    });

    expect(queryByTestId("home-pending-review-summary")).toBeNull();
    expect(getByTestId("home-sign-out")).toBeTruthy();
  });

  it("HP-3: no recent assets — shows empty hint, quick-actions still present", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });
    mockListNotifications.mockResolvedValue({
      ok: true,
      value: { data: { notifications: [] }, sessionRotation: null },
    });

    const { getByTestId, getByText } = await renderHome();

    await waitFor(() => {
      expect(getByText("No recent assets.")).toBeTruthy();
    });

    expect(getByTestId("home-open-assets")).toBeTruthy();
    expect(getByTestId("home-sign-out")).toBeTruthy();
  });

  it("EC-1: asset fetch fails — shows error state, testID home-screen present", async () => {
    mockClient.get.mockResolvedValue({
      ok: false,
      error: { kind: "http", status: 500 },
    });
    mockListNotifications.mockResolvedValue({
      ok: true,
      value: { data: { notifications: [] }, sessionRotation: null },
    });

    const { getByTestId, getByText } = await renderHome();

    await waitFor(() => {
      expect(getByText("Could not load dashboard")).toBeTruthy();
    });

    expect(getByTestId("home-screen")).toBeTruthy();
    // sign-out absent during error (not rendered until ready)
  });

  it("HP-CommunitySlot: community module slot is always present in the ready tree", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [ASSET_A], sessionRotation: null },
    });
    mockListNotifications.mockResolvedValue({
      ok: true,
      value: { data: { notifications: [] }, sessionRotation: null },
    });

    const { getByTestId } = await renderHome();

    await waitFor(() => {
      expect(getByTestId("home-recent-asset-asset-aaa")).toBeTruthy();
    });

    expect(getByTestId("home-community-slot")).toBeTruthy();
  });

  it("EC-2: session expired — auth.logout invoked", async () => {
    mockClient.get.mockResolvedValue({
      ok: false,
      error: { kind: "session_expired" },
    });
    mockListNotifications.mockResolvedValue({
      ok: true,
      value: { data: { notifications: [] }, sessionRotation: null },
    });

    await renderHome();

    await act(async () => {});

    await waitFor(() => {
      expect(mockAuthValue.logout).toHaveBeenCalled();
    });
  });
});
