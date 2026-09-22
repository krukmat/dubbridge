import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";
import * as Clipboard from "expo-clipboard";

import { createGatewayClient } from "../src/api/client";
import type { P2pReadyDescriptor } from "../src/api/p2p";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { canCreateP2pInvite } from "../src/p2p/dashboard/MyContentModel";
import { MyContentScreen } from "../src/screens/MyContentScreen";

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

jest.mock("expo-clipboard", () => ({
  setStringAsync: jest.fn().mockResolvedValue(true),
}));

const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;
const mockSetStringAsync = Clipboard.setStringAsync as jest.MockedFunction<
  typeof Clipboard.setStringAsync
>;

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

function readyOwnerItem() {
  return ownerApiItem({
    assetId: "asset-ready",
    title: "Ready package",
    publicationId: "pub-ready",
    lineageId: "lineage-ready",
    state: "ready",
    descriptor: READY_DESCRIPTOR,
  });
}

function secondReadyOwnerItem() {
  return ownerApiItem({
    assetId: "asset-ready-2",
    title: "Second ready package",
    publicationId: "pub-ready-2",
    lineageId: "lineage-ready-2",
    state: "ready",
    descriptor: {
      ...READY_DESCRIPTOR,
      assetId: "asset-ready-2",
      publicationId: "pub-ready-2",
      lineageId: "lineage-ready-2",
      externalPublicationId: "hyperdrive:ready-2",
      ckWrapRef: "k1:ready-2",
    },
  });
}

function createdInvitation(token = "invite-secret") {
  return {
    ok: true,
    value: {
      data: {
        invitation: {
          id: "invite-1",
          asset_id: "asset-ready",
          publication_id: "pub-ready",
          lineage_id: "lineage-ready",
          owner_subject_id: "owner-1",
          status: "pending",
          expires_at_unix: 2_000_000_000,
          claimed_at_unix: null,
        },
        token,
      },
      sessionRotation: "rotated-session",
    },
  };
}

beforeEach(() => {
  mockClient = {
    get: jest.fn(),
    post: jest.fn(),
    postMultipart: jest.fn(),
  };
  mockCreateGatewayClient.mockReturnValue(mockClient as never);
  mockSetStringAsync.mockClear();
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

async function renderScreen() {
  const result = await render(<MyContentScreen gatewayBaseUrl="http://localhost:3000" />);
  return result;
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
          readyOwnerItem(),
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

    const { getByText, getByTestId, queryByTestId } = await renderScreen();

    await waitFor(() => expect(getByText("Ready package")).toBeTruthy());

    expect(getByText("Processing")).toBeTruthy();
    expect(getByText("Ready")).toBeTruthy();
    expect(getByText("Failed")).toBeTruthy();
    expect(queryByTestId("my-content-create-invite-asset-processing")).toBeNull();
    expect(queryByTestId("my-content-create-invite-asset-failed")).toBeNull();
    expect(getByTestId("my-content-create-invite-asset-ready")).toBeTruthy();
  });

  it("creates through P3, exposes the raw token transiently, and copies it", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [readyOwnerItem()], sessionRotation: null },
    });
    mockClient.post.mockResolvedValue(createdInvitation());

    const { getByTestId, getByText, queryByText } = await renderScreen();
    await waitFor(() => expect(getByText("Ready package")).toBeTruthy());

    await act(async () => {
      fireEvent.press(getByTestId("my-content-create-invite-asset-ready"));
    });

    await waitFor(() => expect(getByText("invite-secret")).toBeTruthy());
    expect(mockClient.post).toHaveBeenCalledWith(
      "/api/assets/asset-ready/p2p/invitations",
      "session-p6",
      {},
    );
    expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("rotated-session");

    await act(async () => {
      fireEvent.press(getByTestId("my-content-copy-invite"));
    });
    expect(mockSetStringAsync).toHaveBeenCalledWith("invite-secret");
    expect(getByText("Copied")).toBeTruthy();

    await act(async () => {
      fireEvent.press(getByTestId("my-content-dismiss-invite"));
    });
    await waitFor(() => expect(queryByText("invite-secret")).toBeNull());
  });

  it("does not reconstruct a raw invite token after the screen remounts", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [readyOwnerItem()], sessionRotation: null },
    });
    mockClient.post.mockResolvedValue(createdInvitation("one-time-secret"));

    const first = await renderScreen();
    await waitFor(() => expect(first.getByText("Ready package")).toBeTruthy());
    await act(async () => {
      fireEvent.press(first.getByTestId("my-content-create-invite-asset-ready"));
    });
    await waitFor(() => expect(first.getByText("one-time-secret")).toBeTruthy());
    await act(async () => {
      first.unmount();
    });

    const second = await renderScreen();
    await waitFor(() => expect(second.getByText("Ready package")).toBeTruthy());
    expect(second.queryByText("one-time-secret")).toBeNull();
    expect(mockClient.post).toHaveBeenCalledTimes(1);
  });

  it.each([
    ["403", { kind: "forbidden" }, "Invite creation is not allowed for this content."],
    ["404", { kind: "http", status: 404 }, "This content is no longer eligible for an invite."],
    ["409", { kind: "http", status: 409 }, "This content is no longer eligible for an invite."],
  ])(
    "refreshes authoritative content and fails closed when P3 rejects stale Ready state with %s",
    async (_label, error, expectedMessage) => {
      mockClient.get
        .mockResolvedValueOnce({
          ok: true,
          value: { data: [readyOwnerItem()], sessionRotation: null },
        })
        .mockResolvedValueOnce({
          ok: true,
          value: {
            data: [
              ownerApiItem({
                assetId: "asset-ready",
                title: "Ready package",
                publicationId: "pub-ready",
                lineageId: "lineage-ready",
                state: "processing",
                descriptor: null,
              }),
            ],
            sessionRotation: null,
          },
        });
      mockClient.post.mockResolvedValue({ ok: false, error });

      const { getByTestId, getByText, queryByTestId, queryByText } = await renderScreen();
      await waitFor(() => expect(getByText("Ready package")).toBeTruthy());

      await act(async () => {
        fireEvent.press(getByTestId("my-content-create-invite-asset-ready"));
      });

      await waitFor(() => expect(getByTestId("my-content-invite-error")).toBeTruthy());
      expect(getByText(expectedMessage)).toBeTruthy();
      expect(mockClient.get).toHaveBeenCalledTimes(2);
      expect(queryByTestId("my-content-create-invite-asset-ready")).toBeNull();
      expect(queryByText("invite-secret")).toBeNull();
    },
  );

  it("locks other Create actions while a one-time raw token is visible", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: {
        data: [readyOwnerItem(), secondReadyOwnerItem()],
        sessionRotation: null,
      },
    });
    mockClient.post.mockResolvedValue(createdInvitation("locked-token"));

    const { getByTestId, getByText } = await renderScreen();
    await waitFor(() => expect(getByText("Second ready package")).toBeTruthy());

    await act(async () => {
      fireEvent.press(getByTestId("my-content-create-invite-asset-ready"));
    });
    await waitFor(() => expect(getByText("locked-token")).toBeTruthy());

    const secondCreate = getByTestId("my-content-create-invite-asset-ready-2");
    expect(secondCreate.props.accessibilityState.disabled).toBe(true);
    fireEvent.press(secondCreate);
    expect(mockClient.post).toHaveBeenCalledTimes(1);
  });

  it("logs out when P3 reports the create session expired", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [readyOwnerItem()], sessionRotation: null },
    });
    mockClient.post.mockResolvedValue({
      ok: false,
      error: { kind: "session_expired" },
    });

    const { getByTestId, getByText } = await renderScreen();
    await waitFor(() => expect(getByText("Ready package")).toBeTruthy());
    await act(async () => {
      fireEvent.press(getByTestId("my-content-create-invite-asset-ready"));
    });

    await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
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

    const { getByText, queryByTestId } = await renderScreen();

    await waitFor(() => expect(getByText("Drifted package")).toBeTruthy());
    expect(queryByTestId("my-content-create-invite-asset-ready")).toBeNull();
  });

  it("shows the empty state when the owner has no P2P publications", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });

    const { getByTestId } = await renderScreen();

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

    const { getByTestId, getByText } = await renderScreen();

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

    await renderScreen();
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
