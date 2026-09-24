import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { P2pInboxItem } from "../src/api/p2pDashboard";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import {
  projectViewerInboxItem,
  type ViewerProductAction,
  type ViewerProductState,
} from "../src/p2p/dashboard/InvitesModel";
import type { P2pSyncSnapshot } from "../src/p2p/sync/SyncState";
import { InvitesScreen } from "../src/screens/InvitesScreen";

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

jest.mock("../src/p2p/P2PProvider", () => ({
  useP2PSyncController: () => mockSyncController,
}));

jest.mock("../src/p2p/P2PAudienceService", () => ({
  P2PAudienceService: jest.fn().mockImplementation(() => ({
    claimInvitation: (...args: unknown[]) => mockClaimInvitation(...args),
  })),
}));

const mockClaimInvitation = jest.fn();
const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;

let mockAuthValue: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };
let mockSyncController: { getSyncState: jest.Mock };

const DESCRIPTOR = {
  descriptorVersion: "p2p-ready-descriptor-v1",
  assetId: "asset-1",
  publicationId: "pub-1",
  lineageId: "lineage-1",
  manifestVersion: "p2p-manifest-v1",
  manifestDigestSha256: "a".repeat(64),
  externalPublicationId: "b".repeat(64),
  ckWrapRef: "p2p-k1-wrap/pub-1/lineage-1",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-23T00:00:00Z",
} as const;

function inboxItem(overrides: Partial<P2pInboxItem> = {}): P2pInboxItem {
  return {
    invitation: {
      id: "invite-1",
      assetId: "asset-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
      ownerSubjectId: "owner-1",
      status: "claimed",
      expiresAtUnix: 2_000_000_000,
      claimedAtUnix: 1_900_000_000,
    },
    authorization: {
      id: "auth-1",
      invitationId: "invite-1",
      assetId: "asset-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
      viewerSubjectId: "viewer-1",
      deviceId: "device-1",
      expiresAtUnix: 2_000_000_000,
    },
    authorizationActive: true,
    descriptor: DESCRIPTOR,
    ...overrides,
  };
}

function snapshot(
  phase: P2pSyncSnapshot["phase"],
  verified = false,
): P2pSyncSnapshot {
  return {
    schemaVersion: 1,
    identity: {
      accountScope: "viewer-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
    },
    phase,
    progress: verified
      ? { filesCompleted: 2, totalFiles: 2, bytesCompleted: 100, totalBytes: 100 }
      : { filesCompleted: 1, totalFiles: 2, bytesCompleted: 50, totalBytes: 100 },
    manifestVerified: verified,
    packageVerified: verified,
    updatedAtUnixMs: 1,
    lastError: phase === "FAILED" ? "sync_failed" : null,
  };
}

function rawInboxItem({
  viewerSubjectId = "viewer-1",
  authorizationActive = true,
  descriptor = DESCRIPTOR,
}: {
  viewerSubjectId?: string;
  authorizationActive?: boolean;
  descriptor?: typeof DESCRIPTOR | null;
} = {}) {
  return {
    invitation: {
      id: "invite-1",
      asset_id: "asset-1",
      publication_id: "pub-1",
      lineage_id: "lineage-1",
      owner_subject_id: "owner-1",
      status: "claimed",
      expires_at_unix: 2_000_000_000,
      claimed_at_unix: 1_900_000_000,
    },
    authorization: {
      id: "auth-1",
      invitation_id: "invite-1",
      asset_id: "asset-1",
      publication_id: "pub-1",
      lineage_id: "lineage-1",
      viewer_subject_id: viewerSubjectId,
      device_id: "device-1",
      expires_at_unix: 2_000_000_000,
    },
    authorization_active: authorizationActive,
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
  mockSyncController = {
    getSyncState: jest.fn(),
  };
  mockCreateGatewayClient.mockReturnValue(mockClient as never);
  mockClaimInvitation.mockReset();
  mockAuthValue = {
    sessionRef: "session-p6",
    userId: "viewer-1",
    status: "authed",
    loginError: null,
    login: jest.fn(),
    logout: jest.fn(),
    onSessionRotation: jest.fn().mockResolvedValue(undefined),
  };
});

afterEach(cleanup);

function renderInvitesScreen() {
  return render(<InvitesScreen gatewayBaseUrl="http://localhost:3000" />);
}

describe("InvitesModel", () => {
  it.each([
    ["inactive authorization wins over local READY", inboxItem({ authorizationActive: false }), snapshot("READY", true), "expired", "none"],
    ["missing descriptor stays Pending", inboxItem({ descriptor: null }), null, "pending", "refresh"],
    ["IDLE offers Sync", inboxItem(), snapshot("IDLE"), "pending", "sync"],
    ["CANCELLED offers Sync", inboxItem(), snapshot("CANCELLED"), "pending", "sync"],
    ["FAILED offers Retry Sync", inboxItem(), snapshot("FAILED"), "sync_error", "retry_sync"],
    ["DISCOVERING stays Syncing", inboxItem(), snapshot("DISCOVERING"), "syncing", "none"],
    ["DOWNLOADING stays Syncing", inboxItem(), snapshot("DOWNLOADING"), "syncing", "none"],
    ["VERIFYING stays Syncing", inboxItem(), snapshot("VERIFYING"), "syncing", "none"],
    ["RETRYING stays Syncing", inboxItem(), snapshot("RETRYING"), "syncing", "none"],
    ["verified READY becomes Available", inboxItem(), snapshot("READY", true), "available", "play"],
    ["unverified READY cannot become Available", inboxItem(), snapshot("READY"), "syncing", "none"],
  ] as const)(
    "%s",
    (_label, item, localState, expectedState, expectedAction) => {
      const projection = projectViewerInboxItem(item, localState, "viewer-1");
      expect(projection?.state).toBe(expectedState as ViewerProductState);
      expect(projection?.action).toBe(expectedAction as ViewerProductAction);
    },
  );

  it("fails closed on a mismatched descriptor", () => {
    const item = inboxItem({
      descriptor: { ...DESCRIPTOR, lineageId: "other-lineage" },
    });
    expect(projectViewerInboxItem(item, snapshot("READY", true), "viewer-1")).toMatchObject({
      state: "pending",
      action: "refresh",
      snapshot: null,
    });
  });

  it("never projects an invitation belonging to another viewer", () => {
    const item = inboxItem({
      authorization: {
        ...inboxItem().authorization,
        viewerSubjectId: "viewer-2",
      },
    });
    expect(projectViewerInboxItem(item, snapshot("READY", true), "viewer-1")).toBeNull();
  });
});

describe("InvitesScreen T2.B", () => {
  it("joins the authoritative inbox with verified P4 state and renders Available", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: "rotated-session" },
    });
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));

    const { getByText, getByTestId } = await renderInvitesScreen();

    await waitFor(() => expect(getByText("Available")).toBeTruthy());
    expect(getByTestId("invite-row-invite-1")).toBeTruthy();
    expect(mockClient.get).toHaveBeenCalledWith("/api/p2p/inbox", "session-p6");
    expect(mockSyncController.getSyncState).toHaveBeenCalledWith({
      accountScope: "viewer-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
    });
    expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("rotated-session");
  });

  it("treats descriptor absence as Pending without consulting local sync state", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem({ descriptor: null })], sessionRotation: null },
    });

    const { getByText } = await renderInvitesScreen();

    await waitFor(() => expect(getByText("Pending")).toBeTruthy());
    expect(mockSyncController.getSyncState).not.toHaveBeenCalled();
  });

  it("does not display a backend item whose authorization names another viewer", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: {
        data: [rawInboxItem({ viewerSubjectId: "viewer-2" })],
        sessionRotation: null,
      },
    });

    const { getByTestId, queryByTestId } = await renderInvitesScreen();

    await waitFor(() => expect(getByTestId("invites-empty")).toBeTruthy());
    expect(queryByTestId("invite-row-invite-1")).toBeNull();
    expect(mockSyncController.getSyncState).not.toHaveBeenCalled();
  });

  it("renders the explicit empty state", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });

    const { getByTestId } = await renderInvitesScreen();

    await waitFor(() => expect(getByTestId("invites-empty")).toBeTruthy());
  });

  it("shows a retryable API error and reloads the authoritative inbox", async () => {
    mockClient.get
      .mockResolvedValueOnce({
        ok: false,
        error: { kind: "network", message: "offline" },
      })
      .mockResolvedValueOnce({
        ok: true,
        value: { data: [rawInboxItem({ descriptor: null })], sessionRotation: null },
      });

    const { getByTestId, getByText } = await renderInvitesScreen();

    await waitFor(() => expect(getByTestId("invites-error")).toBeTruthy());
    await act(async () => {
      fireEvent.press(getByTestId("invites-error-retry"));
    });
    await waitFor(() => expect(getByText("Pending")).toBeTruthy());
    expect(mockClient.get).toHaveBeenCalledTimes(2);
  });

  it("logs out when the authoritative inbox reports session expiry", async () => {
    mockClient.get.mockResolvedValue({
      ok: false,
      error: { kind: "session_expired" },
    });

    await renderInvitesScreen();

    await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
  });

  it("surfaces local cache read failures as retryable errors", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState.mockRejectedValue(new Error("cache unavailable"));

    const { getByTestId, getByText } = await renderInvitesScreen();

    await waitFor(() => expect(getByTestId("invites-error")).toBeTruthy());
    expect(getByText("Could not read local P2P availability.")).toBeTruthy();
  });
});


describe("InvitesScreen T2.C manual Claim", () => {
  it("delegates a trimmed token to P3, rotates session, clears the token and refreshes inbox", async () => {
    mockClient.get
      .mockResolvedValueOnce({
        ok: true,
        value: { data: [], sessionRotation: null },
      })
      .mockResolvedValueOnce({
        ok: true,
        value: { data: [rawInboxItem()], sessionRotation: null },
      });
    mockSyncController.getSyncState.mockResolvedValue(null);
    mockClaimInvitation.mockResolvedValue({
      ok: true,
      value: {
        data: {
          invitation: inboxItem().invitation,
          authorization: inboxItem().authorization,
          descriptor: DESCRIPTOR,
        },
        sessionRotation: "claim-rotation",
      },
    });

    const { getByTestId, getByText, queryByText } = await renderInvitesScreen();
    await waitFor(() => expect(getByTestId("invites-empty")).toBeTruthy());

    fireEvent.changeText(getByTestId("invites-claim-token"), "  raw-claim-token  ");
    fireEvent.press(getByTestId("invites-claim-submit"));

    await waitFor(() => expect(mockClaimInvitation).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(getByText("Pending")).toBeTruthy());
    expect(mockClaimInvitation).toHaveBeenCalledWith("session-p6", "raw-claim-token");
    expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("claim-rotation");
    expect(mockClient.get).toHaveBeenCalledTimes(2);
    expect(getByTestId("invites-claim-token").props.value).toBe("");
    expect(queryByText("Play")).toBeNull();
  });

  it("keeps Claim disabled for blank input and never delegates to P3", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });

    const { getByTestId } = await renderInvitesScreen();
    await waitFor(() => expect(getByTestId("invites-empty")).toBeTruthy());

    const submit = getByTestId("invites-claim-submit");
    expect(submit.props.accessibilityState.disabled).toBe(true);
    fireEvent.press(submit);
    expect(mockClaimInvitation).not.toHaveBeenCalled();
  });

  it("locks duplicate submits while a claim is in flight", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });
    let resolveClaim: ((value: unknown) => void) | undefined;
    mockClaimInvitation.mockImplementation(
      () => new Promise((resolve) => {
        resolveClaim = resolve;
      }),
    );

    const { getByTestId } = await renderInvitesScreen();
    await waitFor(() => expect(getByTestId("invites-empty")).toBeTruthy());
    fireEvent.changeText(getByTestId("invites-claim-token"), "one-token");

    fireEvent.press(getByTestId("invites-claim-submit"));
    await waitFor(() =>
      expect(getByTestId("invites-claim-submit").props.accessibilityState.disabled).toBe(true),
    );
    fireEvent.press(getByTestId("invites-claim-submit"));
    expect(mockClaimInvitation).toHaveBeenCalledTimes(1);

    resolveClaim?.({
      ok: false,
      error: { kind: "network", message: "offline" },
    });
    await waitFor(() =>
      expect(getByTestId("invites-claim-error")).toBeTruthy(),
    );
  });

  it.each([
    [404, "Invitation token is invalid or no longer available."],
    [409, "Invitation has already been claimed."],
    [410, "Invitation has expired."],
  ])("fails closed for claim HTTP %s without fabricating an inbox item", async (status, message) => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });
    mockClaimInvitation.mockResolvedValue({
      ok: false,
      error: { kind: "http", status },
    });

    const { getByTestId, getByText } = await renderInvitesScreen();
    await waitFor(() => expect(getByTestId("invites-empty")).toBeTruthy());
    fireEvent.changeText(getByTestId("invites-claim-token"), "bad-token");

    fireEvent.press(getByTestId("invites-claim-submit"));

    await waitFor(() => expect(getByText(message)).toBeTruthy());
    expect(getByTestId("invites-empty")).toBeTruthy();
    expect(mockClient.get).toHaveBeenCalledTimes(1);
  });

  it("logs out and clears the raw token when P3 reports session expiry", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });
    mockClaimInvitation.mockResolvedValue({
      ok: false,
      error: { kind: "session_expired" },
    });

    const { getByTestId } = await renderInvitesScreen();
    await waitFor(() => expect(getByTestId("invites-empty")).toBeTruthy());
    fireEvent.changeText(getByTestId("invites-claim-token"), "secret-token");

    fireEvent.press(getByTestId("invites-claim-submit"));

    await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
    expect(getByTestId("invites-claim-token").props.value).toBe("");
    expect(mockClient.get).toHaveBeenCalledTimes(1);
  });

  it("does not recover the raw claim token after unmount/remount", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [], sessionRotation: null },
    });

    const first = await renderInvitesScreen();
    await waitFor(() => expect(first.getByTestId("invites-empty")).toBeTruthy());
    fireEvent.changeText(first.getByTestId("invites-claim-token"), "transient-token");
    expect(first.getByTestId("invites-claim-token").props.value).toBe("transient-token");
    await first.unmount();

    const second = await renderInvitesScreen();
    await waitFor(() => expect(second.getByTestId("invites-empty")).toBeTruthy());
    expect(second.getByTestId("invites-claim-token").props.value).toBe("");
    expect(mockClaimInvitation).not.toHaveBeenCalled();
  });
});
