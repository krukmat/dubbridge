import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";
import * as DocumentPicker from "expo-document-picker";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { AssetDetailScreen } from "../src/screens/AssetDetailScreen";
import {
  AssetListScreen,
  type AssetSummary,
} from "../src/screens/AssetListScreen";
import { UploadScreen } from "../src/screens/UploadScreen";

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
const originalE2EEnabled = process.env.EXPO_PUBLIC_E2E_ENABLED;

const ASSET: AssetSummary = {
  id: "asset-123",
  title: "Test Video",
  uploader_id: "user-123",
  status: "finalized",
  created_at: "2026-06-07T10:00:00Z",
  updated_at: "2026-06-07T10:05:00Z",
};

let mockAuthValue: AuthContextValue;
let mockClient: {
  get: jest.Mock;
  post: jest.Mock;
  postMultipart: jest.Mock;
};

// Await fireEvent.press directly first; RNTL v14 wraps it in its own act scope.
// Then use a separate act() to flush async handler continuations.
function flushAsync() {
  return new Promise<void>(resolve => setImmediate(resolve));
}

describe("asset screens", () => {
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
    if (originalE2EEnabled === undefined) {
      delete process.env.EXPO_PUBLIC_E2E_ENABLED;
    } else {
      process.env.EXPO_PUBLIC_E2E_ENABLED = originalE2EEnabled;
    }
    cleanup();
  });

  // SC-LIST-1 HP-1: populated list renders one card per asset
  describe("SC-LIST-1: authenticated user opens AssetList and assets render", () => {
    it("loads asset list data through the gateway and opens an asset", async () => {
      const onOpenAsset = jest.fn();
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: {
          data: [ASSET],
          sessionRotation: "rotated-session-xyz789",
        },
      });

      const view = await render(
        <AssetListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          onOpenAsset={onOpenAsset}
        />,
      );

      await waitFor(() => {
        expect(view.getByText("Test Video")).toBeTruthy();
      });

      expect(view.getByTestId("asset-card-asset-123")).toBeTruthy();
      expect(mockClient.get).toHaveBeenCalledWith(
        "/api/assets",
        "opaque-session-abc123",
      );
      expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith(
        "rotated-session-xyz789",
      );

      await fireEvent.press(view.getByText("Test Video"));

      expect(onOpenAsset).toHaveBeenCalledWith(ASSET);
      expect(view.getByText("Finalized")).toBeTruthy();
    });
  });

  // SC-LIST-1: asset-list-screen testID is present
  describe("asset-list-screen testID", () => {
    it("renders with the asset-list-screen testID", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: { data: [ASSET], sessionRotation: null },
      });

      const view = await render(
        <AssetListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          onOpenAsset={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(view.getByTestId("asset-list-screen")).toBeTruthy();
      });
    });
  });

  // SC-DETAIL-1 HP-2: asset detail shows title, status, asset id, uploader id
  describe("SC-DETAIL-1: authenticated user opens an asset and sees detail/status", () => {
    it("loads asset detail and shows the available S1 summary", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: {
          data: ASSET,
          sessionRotation: null,
        },
      });

      const view = await render(
        <AssetDetailScreen
          assetId={ASSET.id}
          gatewayBaseUrl="http://127.0.0.1:4000"
        />,
      );

      await waitFor(() => {
        expect(view.getByText("Test Video")).toBeTruthy();
      });

      expect(view.getByTestId("asset-detail-screen")).toBeTruthy();
      expect(mockClient.get).toHaveBeenCalledWith(
        `/api/assets/${ASSET.id}`,
        "opaque-session-abc123",
      );
      expect(view.getByText("Finalized")).toBeTruthy();
      expect(view.getByText("Downstream processing")).toBeTruthy();
      expect(
        view.getByText(
          "Not available yet. S4–S9 product surfaces have not been delivered on this mobile client.",
        ),
      ).toBeTruthy();
    });
  });

  // SC-LIST-2: empty result renders empty state (not an error / not not_available)
  describe("SC-LIST-2: empty asset list renders a friendly empty state", () => {
    it("shows an empty state when the gateway returns no assets", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: true,
        value: {
          data: [],
          sessionRotation: null,
        },
      });

      const view = await render(
        <AssetListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          onOpenAsset={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(view.getByText("No assets yet")).toBeTruthy();
      });

      expect(view.getByTestId("asset-list-empty-state")).toBeTruthy();
      // Must not show the old not_available copy
      expect(view.queryByText("Asset list not available yet")).toBeNull();
    });
  });

  // EC: network error renders error state with retry affordance
  describe("EC: gateway or network failure renders a clear error state with retry", () => {
    it("shows an error state and retries on tap", async () => {
      mockClient.get
        .mockResolvedValueOnce({
          ok: false,
          error: { kind: "network", message: "timeout" },
        })
        .mockResolvedValueOnce({
          ok: true,
          value: { data: [ASSET], sessionRotation: null },
        });

      const view = await render(
        <AssetListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          onOpenAsset={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(view.getByText("Could not load assets")).toBeTruthy();
      });

      expect(view.getByText("timeout")).toBeTruthy();

      // Tap retry — should reload and show the asset
      await fireEvent.press(view.getByText("Retry"));

      await waitFor(() => {
        expect(view.getByText("Test Video")).toBeTruthy();
      });

      expect(mockClient.get).toHaveBeenCalledTimes(2);
    });
  });

  // EC: 401 triggers logout
  describe("EC: session_expired triggers logout", () => {
    it("calls auth.logout when the gateway returns session_expired", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: false,
        error: { kind: "session_expired" },
      });

      await render(
        <AssetListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          onOpenAsset={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(mockAuthValue.logout).toHaveBeenCalledTimes(1);
      });
    });
  });

  // ── UploadScreen ──────────────────────────────────────────────────────────
  // Async handler tests (pick-file, finalize) await fireEvent.press directly,
  // then use act+flushAsync so React commits async continuations before asserts.

  const MOCK_FILE = {
    uri: "file:///movie.mp4",
    name: "movie.mp4",
    mimeType: "video/mp4",
  };

  function mockPickerReturnsFile() {
    (DocumentPicker.getDocumentAsync as jest.Mock).mockResolvedValueOnce({
      canceled: false,
      assets: [MOCK_FILE],
    });
  }

  async function fillRightsForm(view: Awaited<ReturnType<typeof render>>) {
    await fireEvent.changeText(view.getByTestId("upload-field-owner"), "DubBridge Studios");
    await fireEvent.changeText(view.getByTestId("upload-field-license-type"), "exclusive");
    await fireEvent.changeText(view.getByTestId("upload-field-source-type"), "original");
    await fireEvent.changeText(view.getByTestId("upload-field-proof-reference"), "contract-123");
  }

  async function submitRightsAndPickFile(view: Awaited<ReturnType<typeof render>>) {
    await fillRightsForm(view);
    await fireEvent.press(view.getByTestId("upload-submit-rights"));
    await waitFor(() => expect(view.getByTestId("upload-pick-file")).toBeTruthy());

    await fireEvent.press(view.getByTestId("upload-pick-file"));
    await act(async () => {
      await flushAsync();
    });
    await waitFor(() => expect(view.getByTestId("upload-finalize")).toBeTruthy());
  }

  async function pressFinalize(view: Awaited<ReturnType<typeof render>>) {
    await fireEvent.press(view.getByTestId("upload-finalize"));
    await act(async () => {
      await flushAsync();
    });
  }

  // SC-INGEST-1 HP-1
  describe("SC-INGEST-1: rights → file → finalize → onSuccess called", () => {
    it("fires all 3 POSTs in sequence and calls onSuccess", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post
        .mockResolvedValueOnce({ ok: true, value: { data: {}, sessionRotation: null } })
        .mockResolvedValueOnce({ ok: true, value: { data: ASSET, sessionRotation: null } });

      const onSuccess = jest.fn();
      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={onSuccess} />,
      );

      // Step 1: fill rights + continue (synchronous state transition)
      await fillRightsForm(view);
      await fireEvent.press(view.getByTestId("upload-submit-rights"));
      await waitFor(() => expect(view.getByTestId("upload-pick-file")).toBeTruthy());

      // Step 2: pick file (async: DocumentPicker → setViewState)
      await fireEvent.press(view.getByTestId("upload-pick-file"));
      await act(async () => {
        await flushAsync(); // flush DocumentPicker resolution inside a fresh act scope
      });
      expect(view.getByTestId("upload-finalize")).toBeTruthy();

      // Step 3: finalize (async: 3 sequential POSTs → onSuccess)
      await fireEvent.press(view.getByTestId("upload-finalize"));
      await act(async () => {
        await flushAsync(); // flush all client.post resolutions inside a fresh act scope
      });
      await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));

      expect(mockClient.postMultipart).toHaveBeenCalledWith(
        "/api/ingest",
        "opaque-session-abc123",
        expect.any(FormData),
      );
      expect(mockClient.post).toHaveBeenCalledWith(
        "/api/ingest/tok-abc/rights",
        "opaque-session-abc123",
        expect.objectContaining({ owner: "DubBridge Studios", license_type: "exclusive" }),
      );
      expect(mockClient.post).toHaveBeenCalledWith(
        "/api/ingest/tok-abc/finalize",
        "opaque-session-abc123",
        {},
      );
    });
  });

  // SC-INGEST-2 EC-1: 422 on finalize → rights-required error → recovery to rights_form
  describe("SC-INGEST-2: 422 on finalize → rights-required error shown", () => {
    it("shows rights-required message and recovers to rights form", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post
        .mockResolvedValueOnce({ ok: true, value: { data: {}, sessionRotation: null } })
        .mockResolvedValueOnce({ ok: false, error: { kind: "http", status: 422 } });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await fillRightsForm(view);
      await fireEvent.press(view.getByTestId("upload-submit-rights"));
      await waitFor(() => expect(view.getByTestId("upload-pick-file")).toBeTruthy());

      await fireEvent.press(view.getByTestId("upload-pick-file"));
      await act(async () => {
        await flushAsync();
      });
      expect(view.getByTestId("upload-finalize")).toBeTruthy();

      await fireEvent.press(view.getByTestId("upload-finalize"));
      await act(async () => {
        await flushAsync();
      });
      await waitFor(() =>
        expect(view.getByText(/rights are required before finalizing/i)).toBeTruthy(),
      );

      await fireEvent.press(view.getByText("Try again"));
      expect(view.getByTestId("upload-submit-rights")).toBeTruthy();
    });
  });

  // EC-2: 410 during rights → session expired → recovery to rights_form
  describe("EC-2: 410 on rights step → session expired error", () => {
    it("shows expired message and recovers to rights form", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post.mockResolvedValueOnce({
        ok: false,
        error: { kind: "http", status: 410 },
      });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await fillRightsForm(view);
      await fireEvent.press(view.getByTestId("upload-submit-rights"));
      await waitFor(() => expect(view.getByTestId("upload-pick-file")).toBeTruthy());

      await fireEvent.press(view.getByTestId("upload-pick-file"));
      await act(async () => {
        await flushAsync();
      });

      await fireEvent.press(view.getByTestId("upload-finalize"));
      await act(async () => {
        await flushAsync();
      });
      await waitFor(() =>
        expect(view.getByText(/ingest session expired/i)).toBeTruthy(),
      );

      await fireEvent.press(view.getByText("Try again"));
      expect(view.getByTestId("upload-submit-rights")).toBeTruthy();
    });
  });

  // EC-3: session rotation at each step is persisted
  describe("EC-3: session rotation persisted from each step", () => {
    it("calls onSessionRotation after each successful POST", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: "rot-1" },
      });
      mockClient.post
        .mockResolvedValueOnce({ ok: true, value: { data: {}, sessionRotation: "rot-2" } })
        .mockResolvedValueOnce({ ok: true, value: { data: ASSET, sessionRotation: "rot-3" } });

      const onSuccess = jest.fn();
      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={onSuccess} />,
      );

      await fillRightsForm(view);
      await fireEvent.press(view.getByTestId("upload-submit-rights"));
      await waitFor(() => expect(view.getByTestId("upload-pick-file")).toBeTruthy());

      await fireEvent.press(view.getByTestId("upload-pick-file"));
      await act(async () => {
        await flushAsync();
      });

      await fireEvent.press(view.getByTestId("upload-finalize"));
      await act(async () => {
        await flushAsync();
      });
      await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));

      expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("rot-1");
      expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("rot-2");
      expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("rot-3");
    });
  });

  // EC-4: Continue blocked when a rights field is empty
  describe("EC-4: Continue blocked when rights fields are incomplete", () => {
    it("stays on rights_form when a field is empty", async () => {
      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      // Fill only 3 of 4 fields
      await fireEvent.changeText(view.getByTestId("upload-field-owner"), "DubBridge Studios");
      await fireEvent.changeText(view.getByTestId("upload-field-license-type"), "exclusive");
      await fireEvent.changeText(view.getByTestId("upload-field-source-type"), "original");
      // upload-field-proof-reference left empty

      await fireEvent.press(view.getByTestId("upload-submit-rights"));

      expect(view.getByTestId("upload-submit-rights")).toBeTruthy();
      expect(view.queryByTestId("upload-pick-file")).toBeNull();
    });
  });

  // EC-5: cancel file picker stays at file_pending
  describe("EC-5: cancel file picker stays at file_pending step", () => {
    it("does not advance when user cancels the picker", async () => {
      (DocumentPicker.getDocumentAsync as jest.Mock).mockResolvedValueOnce({
        canceled: true,
        assets: [],
      });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await fillRightsForm(view);
      await fireEvent.press(view.getByTestId("upload-submit-rights"));
      await waitFor(() => expect(view.getByTestId("upload-pick-file")).toBeTruthy());

      await fireEvent.press(view.getByTestId("upload-pick-file"));
      await act(async () => {
        await flushAsync();
      });

      // Canceled → stays at file_pending
      expect(view.getByTestId("upload-pick-file")).toBeTruthy();
      expect(view.queryByTestId("upload-finalize")).toBeNull();
      expect(mockClient.postMultipart).not.toHaveBeenCalled();
    });
  });

  // EC-6: picker returns no asset
  describe("EC-6: empty picker result stays at file_pending step", () => {
    it("does not advance when the picker returns no asset", async () => {
      (DocumentPicker.getDocumentAsync as jest.Mock).mockResolvedValueOnce({
        canceled: false,
        assets: [],
      });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await fillRightsForm(view);
      await fireEvent.press(view.getByTestId("upload-submit-rights"));
      await waitFor(() => expect(view.getByTestId("upload-pick-file")).toBeTruthy());

      await fireEvent.press(view.getByTestId("upload-pick-file"));
      await act(async () => {
        await flushAsync();
      });

      expect(view.getByTestId("upload-pick-file")).toBeTruthy();
      expect(view.queryByTestId("upload-finalize")).toBeNull();
      expect(mockClient.postMultipart).not.toHaveBeenCalled();
    });
  });

  // EC-7: upload metadata defaults
  describe("EC-7: picker asset missing metadata uses defaults", () => {
    it("uploads with fallback file name and MIME type", async () => {
      (DocumentPicker.getDocumentAsync as jest.Mock).mockResolvedValueOnce({
        canceled: false,
        assets: [{ uri: "file:///untitled" }],
      });
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post
        .mockResolvedValueOnce({ ok: true, value: { data: {}, sessionRotation: null } })
        .mockResolvedValueOnce({ ok: true, value: { data: ASSET, sessionRotation: null } });

      const onSuccess = jest.fn();
      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={onSuccess} />,
      );

      await submitRightsAndPickFile(view);
      expect(view.getByText("file")).toBeTruthy();

      await pressFinalize(view);
      await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
    });
  });

  describe("EC-7b: E2E mode bypasses the native document picker", () => {
    it("starts directly at the file-pending step when EXPO_PUBLIC_E2E_ENABLED is true", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      expect(view.getByTestId("upload-pick-file")).toBeTruthy();
      expect(view.queryByTestId("upload-submit-rights")).toBeNull();
    });

    it("uses the seeded upload asset when EXPO_PUBLIC_E2E_ENABLED is true", async () => {
      process.env.EXPO_PUBLIC_E2E_ENABLED = "true";
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post
        .mockResolvedValueOnce({ ok: true, value: { data: {}, sessionRotation: null } })
        .mockResolvedValueOnce({ ok: true, value: { data: ASSET, sessionRotation: null } });

      const onSuccess = jest.fn();
      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={onSuccess} />,
      );

      await fireEvent.press(view.getByTestId("upload-pick-file"));
      await waitFor(() =>
        expect(view.getByText("dubbridge-e2e-upload.mov")).toBeTruthy(),
      );

      expect(DocumentPicker.getDocumentAsync).not.toHaveBeenCalled();

      await pressFinalize(view);
      await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
    });
  });

  // EC-8: ingest step errors
  describe("EC-8: ingest upload errors are surfaced", () => {
    it("shows file-too-large message and recovers to ready", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: false,
        error: { kind: "http", status: 413 },
      });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await submitRightsAndPickFile(view);
      await pressFinalize(view);

      await waitFor(() =>
        expect(view.getByText(/file too large/i)).toBeTruthy(),
      );
      await fireEvent.press(view.getByText("Try again"));
      expect(view.getByTestId("upload-finalize")).toBeTruthy();
    });

    it("logs out when ingest upload returns session_expired", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: false,
        error: { kind: "session_expired" },
      });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await submitRightsAndPickFile(view);
      await pressFinalize(view);

      await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
    });
  });

  // EC-9: rights step errors
  describe("EC-9: rights step errors are surfaced", () => {
    it("shows permission error and recovers to ready", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post.mockResolvedValueOnce({
        ok: false,
        error: { kind: "forbidden" },
      });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await submitRightsAndPickFile(view);
      await pressFinalize(view);

      await waitFor(() =>
        expect(view.getByText(/do not have permission/i)).toBeTruthy(),
      );
      await fireEvent.press(view.getByText("Try again"));
      expect(view.getByTestId("upload-finalize")).toBeTruthy();
    });

    it("logs out when rights step returns session_expired", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post.mockResolvedValueOnce({
        ok: false,
        error: { kind: "session_expired" },
      });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await submitRightsAndPickFile(view);
      await pressFinalize(view);

      await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
    });
  });

  // EC-10: finalize step errors
  describe("EC-10: finalize step errors are surfaced", () => {
    it("logs out when finalize returns session_expired", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post
        .mockResolvedValueOnce({ ok: true, value: { data: {}, sessionRotation: null } })
        .mockResolvedValueOnce({ ok: false, error: { kind: "session_expired" } });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await submitRightsAndPickFile(view);
      await pressFinalize(view);

      await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
    });

    it("shows generic HTTP error and recovers to ready", async () => {
      mockPickerReturnsFile();
      mockClient.postMultipart.mockResolvedValueOnce({
        ok: true,
        value: { data: { ingest_token: "tok-abc" }, sessionRotation: null },
      });
      mockClient.post
        .mockResolvedValueOnce({ ok: true, value: { data: {}, sessionRotation: null } })
        .mockResolvedValueOnce({ ok: false, error: { kind: "http", status: 500 } });

      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );

      await submitRightsAndPickFile(view);
      await pressFinalize(view);

      await waitFor(() =>
        expect(view.getByText(/request failed \(500\)/i)).toBeTruthy(),
      );
      await fireEvent.press(view.getByText("Try again"));
      expect(view.getByTestId("upload-finalize")).toBeTruthy();
    });
  });

  // testIDs
  describe("upload-screen testID present on mount", () => {
    it("renders with upload-screen testID", async () => {
      const view = await render(
        <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
      );
      expect(view.getByTestId("upload-screen")).toBeTruthy();
    });
  });
});
