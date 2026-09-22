import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { P2pReadyDescriptor } from "../src/api/p2p";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { MyContentScreen, canCreateP2pInvite } from "../src/screens/MyContentScreen";

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

const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;

let mockAuthValue: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };

const READY_DESCRIPTOR: P2pReadyDescriptor = {
  descriptorVersion: "p2p-ready-v1",
  assetId: "asset-ready",
  publicationId: "pub-ready",
  lineageId: "lineage-ready",
  manifestVersion: "p2p-manifest-v1",
  manifestDigestSha256: "a".repeat(64),
  externalPublicationId: "hyperdrive:ready",
  ckWrapRef: "k1:ready",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-22T18:00:00Z",
};

function ownerApiItem({
  assetId,
  title,
  publicationId,
  lineageId,
  state,
  descriptor,
}: {
  assetId: string;
  title: string;
  publicationId: string;
  lineageId: string;
  state: "processing" | "ready" | "failed";
  descriptor: P2pReadyDescriptor | null;
}) {
  return {
    asset_id: assetId,
    title,
    publication_id: publicationId,
    lineage_id: lineageId,
    state,
    descriptor: descriptor
      ? {
          descriptor_version: descriptor.descriptorVersion,
          asset_id: descriptor.assetId,
          publication_id: descriptor.publicationId,
          lineage_id: descriptor.lineageId,
          manifest_version: descriptor.manifestVersion,
          manifest_digest_sha256: descriptor.manifestDigestSha256,
          external_publication_id: descriptor.externalPublicationId,
          ck_wrap_ref: descriptor.ckWrapRef,
          kek_id: descriptor.kekId,
          kek_version: descriptor.kekVersion,
          ready_at: descriptor.readyAt,
        }
      : null,
  };
}

beforeEach(() => {
  mockClient = {
    get: jest.fn(),
    post: jest.fn(),
    postMultipart: jest.fn(),
  };
  mockCreateGatewayClient.mockReturnValue(mockClient as never);
  mockAuthValue = {
    sessionRef: "session-p6",
    userId: "owner-1",
    status: "authed",
    loginError: null,
    login: jest.fn(),
    logout: jest.fn(),
    onSessionRotation: jest.fn().mockResolvedValue(undefined),
  };
});

afterEach(cleanup);

function renderScreen(onCreateInvite = jest.fn()) {
  return {
    onCreateInvite,
    ...render(
      <MyContentScreen
        gatewayBaseUrl="http://localhost:3000"
        onCreateInvite={onCreateInvite}
      />,
    ),
  };
}

describe("MyContentScreen", () => {
  it("renders authoritative processing, ready and failed states and exposes Invite only for exact Ready", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: {
        data: [
          ownerApiItem({
            assetId: "asset-processing",
            title: "Processing package",
            publicationId: "pub-processing",
            lineageId: "lineage-processing",
            state: "processing",
            descriptor: null,
          }),
          ownerApiItem({
            assetId: "asset-ready",
            title: "Ready package",
            publicationId: "pub-ready",
            lineageId: "lineage-ready",
            state: "ready",
            descriptor: READY_DESCRIPTOR,
          }),
          ownerApiItem({
            assetId: "asset-failed",
            title: "Failed package",
            publicationId: "pub-failed",
            lineageId: "lineage-failed",
            state: "failed",
            descriptor: null,
          }),
        ],
        sessionRotation: null,
      },
    });

    const { getByText, getByTestId, queryByTestId, onCreateInvite } = renderScreen();

    await waitFor(() => expect(getByText("Ready package")).toBeTruthy());

    expect(getByText("Processing")).toBeTruthy();
    expect(getByText("Ready")).toBeTruthy();
    expect(getByText("Failed")).toBeTruthy();
    expect(queryByTestId("my-content-create-invite-asset-processing")).toBeNull();
    expect(queryByTestId("my-content-create-invite-asset-failed")).toBeNull();

    fireEvent.press(getByTestId("my-content-create-invite-asset-ready"));
    expect(onCreateInvite).toHaveBeenCalledTimes(1);
    expect(onCreateInvite).toHaveBeenCalledWith(
      expect.objectContaining({
        assetId: "asset-ready",
        publicationId: "pub-ready",
        lineageId: "lineage-ready",
        state: "ready",
      }),
    );
  });

  it("fails closed when Ready carries a mismatched descriptor", async () => {
    const mismatchedDescriptor = { ...READY_DESCRIPTOR, lineageId: "other-lineage" };
    mockClient.get.mockResolvedValue({
      ok: true,
      value: {
        data: [
          ownerApiItem({
            assetId: "asset-ready",
            title: "Drifted package",
            publicationId: "pub-ready",
            lineageId: "lineage-ready",
            state: "ready",
            descriptor: mismatchedDescriptor,
          }),
        ],
        sessionRotation: null,
      },
    });

    const { getByText, queryByTestId } = renderScreen();

    await waitFor(() => expect(getByText("Drifted package")).toBeTruthy());
    expect(queryByTestId("my-content-create-invite-asset-ready")).toBeNull();
  });

  it("shows the empty state when the owner has no P2P publications", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });

    const { getByTestId } = renderScreen();

    await waitFor(() => expect(getByTestId("my-content-empty")).toBeTruthy());
  });

  it("shows a retryable error and reloads owner content", async () => {
    mockClient.get
      .mockResolvedValueOnce({
        ok: false,
        error: { kind: "network", message: "offline" },
      })
      .mockResolvedValueOnce({
        ok: true,
        value: {
          data: [
            ownerApiItem({
              assetId: "asset-ready",
              title: "Recovered package",
              publicationId: "pub-ready",
              lineageId: "lineage-ready",
              state: "ready",
              descriptor: READY_DESCRIPTOR,
            }),
          ],
          sessionRotation: null,
        },
      });

    const { getByTestId, getByText } = renderScreen();

    await waitFor(() => expect(getByTestId("my-content-error")).toBeTruthy());
    await act(async () => {
      fireEvent.press(getByTestId("my-content-error-retry"));
    });
    await waitFor(() => expect(getByText("Recovered package")).toBeTruthy());
    expect(mockClient.get).toHaveBeenCalledTimes(2);
  });

  it("logs out when the dashboard session has expired", async () => {
    mockClient.get.mockResolvedValue({
      ok: false,
      error: { kind: "session_expired" },
    });

    renderScreen();
    await act(async () => {});

    await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
  });
});

describe("canCreateP2pInvite", () => {
  it("requires exact asset, publication and lineage binding", () => {
    const item = {
      assetId: "asset-ready",
      title: "Ready",
      publicationId: "pub-ready",
      lineageId: "lineage-ready",
      state: "ready" as const,
      descriptor: READY_DESCRIPTOR,
    };
    expect(canCreateP2pInvite(item)).toBe(true);
    expect(canCreateP2pInvite({ ...item, state: "processing" })).toBe(false);
    expect(canCreateP2pInvite({ ...item, descriptor: null })).toBe(false);
    expect(
      canCreateP2pInvite({
        ...item,
        descriptor: { ...READY_DESCRIPTOR, publicationId: "other-publication" },
      }),
    ).toBe(false);
  });
});
