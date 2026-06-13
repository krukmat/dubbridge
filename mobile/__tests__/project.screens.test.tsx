import { cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { ProjectDetailScreen } from "../src/screens/ProjectDetailScreen";
import { ProjectListScreen } from "../src/screens/ProjectListScreen";

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

const mockCreateGatewayClient =
  createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;

const PROJECT_A = {
  id: "proj-001",
  org_id: "org-abc",
  name: "Dubbing Alpha",
  created_at: "2026-06-01T00:00:00Z",
};

const PROJECT_B = {
  id: "proj-002",
  org_id: "org-abc",
  name: "Dubbing Beta",
  created_at: "2026-06-02T00:00:00Z",
};

const ASSET_A = { id: "asset-111", title: "Scene 1", status: "finalized" };
const ASSET_B = { id: "asset-222", title: "Scene 2", status: "processing" };

const PROJECT_DETAIL = {
  id: PROJECT_A.id,
  org_id: PROJECT_A.org_id,
  name: PROJECT_A.name,
  assets: [ASSET_A, ASSET_B],
  target_languages: [
    {
      id: "lang-001",
      project_id: PROJECT_A.id,
      source_lang: "en",
      target_lang: "es-ES",
      created_at: "2026-06-02T00:00:00Z",
    },
  ],
  created_at: PROJECT_A.created_at,
  updated_at: PROJECT_A.created_at,
};

let mockAuthValue: AuthContextValue;
let mockClient: {
  get: jest.Mock;
  post: jest.Mock;
  postMultipart: jest.Mock;
};

describe("project screens", () => {
  beforeEach(() => {
    jest.clearAllMocks();

    mockAuthValue = {
      sessionRef: "opaque-session-abc123",
      status: "authed",
      loginError: null,
      login: jest.fn().mockResolvedValue(undefined),
      logout: jest.fn().mockResolvedValue(undefined),
      onSessionRotation: jest.fn().mockResolvedValue(undefined),
    };

    mockClient = {
      get: jest.fn(),
      post: jest.fn(),
      postMultipart: jest.fn(),
    };

    mockCreateGatewayClient.mockReturnValue(
      mockClient as unknown as ReturnType<typeof createGatewayClient>,
    );
  });

  afterEach(() => {
    cleanup();
  });

  // ── ProjectListScreen ────────────────────────────────────────────────────────

  // HP-1: 2 projects → 2 cards; testID present; onOpenProject called on tap
  describe("HP-1: populated project list renders one card per project", () => {
    it("renders project-list-screen testID and one card per project, calls onOpenProject on tap", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: { data: [PROJECT_A, PROJECT_B], sessionRotation: "rotated-xyz" },
      });

      const onOpenProject = jest.fn();
      const view = await render(
        <ProjectListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          orgId="org-abc"
          onOpenProject={onOpenProject}
        />,
      );

      await waitFor(() => {
        expect(view.getByText("Dubbing Alpha")).toBeTruthy();
      });

      expect(view.getByTestId("project-list-screen")).toBeTruthy();
      expect(view.getByTestId("project-card-proj-001")).toBeTruthy();
      expect(view.getByTestId("project-card-proj-002")).toBeTruthy();
      expect(mockClient.get).toHaveBeenCalledWith(
        "/api/orgs/org-abc/projects",
        "opaque-session-abc123",
      );
      expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("rotated-xyz");

      fireEvent.press(view.getByText("Dubbing Alpha"));
      expect(onOpenProject).toHaveBeenCalledWith(PROJECT_A);
    });
  });

  // EC-1: empty list → empty-state, no error
  describe("EC-1: empty project list shows empty state without error", () => {
    it("renders the empty-state panel when the org has no projects", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: { data: [], sessionRotation: null },
      });

      const view = await render(
        <ProjectListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          orgId="org-abc"
          onOpenProject={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(view.getByTestId("project-list-empty-state")).toBeTruthy();
      });

      expect(view.queryByText("Could not load projects")).toBeNull();
    });
  });

  // EC-2a: session_expired on ProjectListScreen → auth.logout()
  describe("EC-2a: session_expired on ProjectListScreen triggers logout", () => {
    it("calls auth.logout when gateway returns session_expired", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: false,
        error: { kind: "session_expired" },
      });

      render(
        <ProjectListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          orgId="org-abc"
          onOpenProject={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(mockAuthValue.logout).toHaveBeenCalledTimes(1);
      });
    });
  });

  // EC-2b: 401 is mapped to session_expired by the client — same logout path
  describe("EC-2b: network error on ProjectListScreen shows error state", () => {
    it("renders error state for a network failure on the project list", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: false,
        error: { kind: "network", message: "timeout" },
      });

      const view = await render(
        <ProjectListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          orgId="org-abc"
          onOpenProject={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(view.getByText("Could not load projects")).toBeTruthy();
      });

      expect(view.getByText("timeout")).toBeTruthy();
    });
  });

  // ── ProjectDetailScreen ──────────────────────────────────────────────────────

  // HP-1: detail with 2 assets → 2 asset-row nodes; tap navigates to AssetDetail
  describe("HP-1: project detail shows linked assets and tapping opens AssetDetail", () => {
    it("renders project-detail-screen testID, lists asset rows, calls onOpenAsset on tap", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: { data: PROJECT_DETAIL, sessionRotation: null },
      });

      const onOpenAsset = jest.fn();
      const view = await render(
        <ProjectDetailScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          orgId="org-abc"
          projectId="proj-001"
          onOpenAsset={onOpenAsset}
        />,
      );

      await waitFor(() => {
        expect(view.getByText("Scene 1")).toBeTruthy();
      });

      expect(view.getByTestId("project-detail-screen")).toBeTruthy();
      expect(view.getByTestId("asset-row-asset-111")).toBeTruthy();
      expect(view.getByTestId("asset-row-asset-222")).toBeTruthy();
      expect(view.getByTestId("target-language-lang-001")).toBeTruthy();
      expect(view.getByText("en to es-ES")).toBeTruthy();
      expect(mockClient.get).toHaveBeenCalledWith(
        "/api/orgs/org-abc/projects/proj-001",
        "opaque-session-abc123",
      );

      fireEvent.press(view.getByText("Scene 1"));
      expect(onOpenAsset).toHaveBeenCalledWith("asset-111", "Scene 1");
    });
  });

  // EC-1: project with no assets → empty-assets state, no error
  describe("EC-1: project detail with no assets shows empty-assets state", () => {
    it("renders the empty-assets panel when the project has no linked assets", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: {
          data: { ...PROJECT_DETAIL, assets: [], target_languages: [] },
          sessionRotation: null,
        },
      });

      const view = await render(
        <ProjectDetailScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          orgId="org-abc"
          projectId="proj-001"
          onOpenAsset={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(view.getByTestId("project-detail-empty-assets")).toBeTruthy();
      });

      expect(view.queryByText("Could not load project")).toBeNull();
      expect(view.getByTestId("project-detail-empty-languages")).toBeTruthy();
    });
  });

  // EC-2: session_expired on ProjectDetailScreen → auth.logout()
  describe("EC-2: session_expired on ProjectDetailScreen triggers logout", () => {
    it("calls auth.logout when gateway returns session_expired on project detail", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: false,
        error: { kind: "session_expired" },
      });

      render(
        <ProjectDetailScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          orgId="org-abc"
          projectId="proj-001"
          onOpenAsset={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(mockAuthValue.logout).toHaveBeenCalledTimes(1);
      });
    });
  });
});
