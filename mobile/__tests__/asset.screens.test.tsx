import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { AssetDetailScreen } from "../src/screens/AssetDetailScreen";
import {
  AssetListScreen,
  type AssetSummary,
} from "../src/screens/AssetListScreen";

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
};

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
    };

    mockCreateGatewayClient.mockReturnValue(
      mockClient as unknown as ReturnType<typeof createGatewayClient>,
    );
  });

  afterEach(() => {
    cleanup();
  });

  describe("T4 HP-1: authenticated user opens AssetList and assets render", () => {
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

      expect(mockClient.get).toHaveBeenCalledWith(
        "/api/assets?view=mobile",
        "opaque-session-abc123",
      );
      expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith(
        "rotated-session-xyz789",
      );

      fireEvent.press(view.getByText("Test Video"));

      expect(onOpenAsset).toHaveBeenCalledWith(ASSET);
      expect(view.getByText("Finalized")).toBeTruthy();
    });
  });

  describe("T4 HP-2: authenticated user opens an asset and sees detail/status", () => {
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

  describe("T4 EC-1: empty asset list renders a friendly empty state", () => {
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
    });
  });

  describe("T4 EC-2: gateway or network failure renders a clear error state", () => {
    it("shows an error state when the gateway request fails", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: false,
        error: {
          kind: "network",
          message: "timeout",
        },
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
    });
  });

  describe("T4 EC-3: unavailable surfaces render not available yet", () => {
    it("shows a not-available state when the mobile asset list endpoint is not live", async () => {
      mockClient.get.mockResolvedValueOnce({
        ok: false,
        error: {
          kind: "http",
          status: 404,
        },
      });

      const view = await render(
        <AssetListScreen
          gatewayBaseUrl="http://127.0.0.1:4000"
          onOpenAsset={jest.fn()}
        />,
      );

      await waitFor(() => {
        expect(view.getByText("Asset list not available yet")).toBeTruthy();
      });
    });
  });
});
