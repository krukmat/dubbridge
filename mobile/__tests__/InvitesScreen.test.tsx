import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { P2pReadyDescriptor } from "../src/api/p2p";
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
  useP2PService: () => mockP2PService,
}));

jest.mock("../src/p2p/P2PAudienceService", () => ({
  P2PAudienceService: jest.fn().mockImplementation(() => ({
    claimInvitation: (...args: unknown[]) => mockClaimInvitation(...args),
  })),
}));

jest.mock("../src/p2p/playback/P2PPlaybackController", () => ({
  P2PPlaybackController: jest.fn().mockImplementation(() => ({
    start: (...args: unknown[]) => mockStartPlayback(...args),
    stop: (...args: unknown[]) => mockStopPlayback(...args),
  })),
}));

jest.mock("../src/p2p/playback/P2PPlaybackSessionView", () => ({
  P2PPlaybackSessionView: (props: Record<string, unknown>) => {
    mockPlaybackView(props);
    return require("react").createElement(require("react-native").View, {
      testID: props.testID,
    });
  },
}));

const mockClaimInvitation = jest.fn();
const mockStartPlayback = jest.fn();
const mockStopPlayback = jest.fn();
const mockPlaybackView = jest.fn();
const mockP2PService = {};
const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;

let mockAuthValue: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };
let mockSyncController: {
  getSyncState: jest.Mock;
  startSync: jest.Mock;
  getVerifiedPackageHandle: jest.Mock;
};

const DESCRIPTOR: P2pReadyDescriptor = {
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
};

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
  descriptor?: P2pReadyDescriptor | null;
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
    startSync: jest.fn(),
    getVerifiedPackageHandle: jest.fn(),
  };
  mockCreateGatewayClient.mockReturnValue(mockClient as never);
  mockClaimInvitation.mockReset();
  mockStartPlayback.mockReset();
  mockStopPlayback.mockReset();
  mockPlaybackView.mockReset();
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

function mockInboxOnce(data: unknown[]) {
  mockClient.get.mockResolvedValueOnce({
    ok: true,
    value: { data, sessionRotation: null },
  });
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

  it.each(["expired", "revoked"] as const)(
    "%s invitation overrides cached READY",
    (status) => {
      const item = inboxItem({
        invitation: { ...inboxItem().invitation, status },
      });
      expect(projectViewerInboxItem(item, snapshot("READY", true), "viewer-1")).toMatchObject({
        state: "expired",
        action: "none",
      });
    },
  );
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

    await act(async () => {
      fireEvent.changeText(getByTestId("invites-claim-token"), "  raw-claim-token  ");
    });
    await waitFor(() =>
      expect(getByTestId("invites-claim-token").props.value).toBe("  raw-claim-token  "),
    );
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
    await act(async () => {
      fireEvent.changeText(getByTestId("invites-claim-token"), "one-token");
    });
    await waitFor(() =>
      expect(getByTestId("invites-claim-token").props.value).toBe("one-token"),
    );

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
    await act(async () => {
      fireEvent.changeText(getByTestId("invites-claim-token"), "bad-token");
    });
    await waitFor(() =>
      expect(getByTestId("invites-claim-token").props.value).toBe("bad-token"),
    );

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
    await act(async () => {
      fireEvent.changeText(getByTestId("invites-claim-token"), "secret-token");
    });
    await waitFor(() =>
      expect(getByTestId("invites-claim-token").props.value).toBe("secret-token"),
    );

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
    await act(async () => {
      fireEvent.changeText(first.getByTestId("invites-claim-token"), "transient-token");
    });
    await waitFor(() =>
      expect(first.getByTestId("invites-claim-token").props.value).toBe("transient-token"),
    );
    await first.unmount();

    const second = await renderInvitesScreen();
    await waitFor(() => expect(second.getByTestId("invites-empty")).toBeTruthy());
    expect(second.getByTestId("invites-claim-token").props.value).toBe("");
    expect(mockClaimInvitation).not.toHaveBeenCalled();
  });
});


describe("InvitesScreen T2.D Sync / Retry Sync", () => {
  it("delegates Pending Sync to P4 and refreshes into Syncing", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(snapshot("DOWNLOADING"));
    mockSyncController.startSync.mockResolvedValue(snapshot("DOWNLOADING"));

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByTestId("invite-sync-invite-1")).toBeTruthy());

    fireEvent.press(view.getByTestId("invite-sync-invite-1"));

    await waitFor(() => expect(mockSyncController.startSync).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(view.getByText("Syncing")).toBeTruthy());
    expect(mockSyncController.startSync).toHaveBeenCalledWith(DESCRIPTOR, "viewer-1");
  });

  it("delegates FAILED Retry Sync to the same P4 startSync capability", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState
      .mockResolvedValueOnce(snapshot("FAILED"))
      .mockResolvedValueOnce(snapshot("RETRYING"));
    mockSyncController.startSync.mockResolvedValue(snapshot("RETRYING"));

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Retry Sync")).toBeTruthy());

    fireEvent.press(view.getByTestId("invite-sync-invite-1"));

    await waitFor(() => expect(mockSyncController.startSync).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(view.getByText("Syncing")).toBeTruthy());
  });

  it("does not expose Sync for inactive authorization even with an exact descriptor", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem({ authorizationActive: false })], sessionRotation: null },
    });

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Expired")).toBeTruthy());

    expect(view.queryByTestId("invite-sync-invite-1")).toBeNull();
    expect(mockSyncController.startSync).not.toHaveBeenCalled();
  });

  it("does not expose Sync for a mismatched descriptor", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: {
        data: [rawInboxItem({ descriptor: { ...DESCRIPTOR, lineageId: "other-lineage" } })],
        sessionRotation: null,
      },
    });

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Pending")).toBeTruthy());

    expect(view.queryByTestId("invite-sync-invite-1")).toBeNull();
    expect(mockSyncController.startSync).not.toHaveBeenCalled();
  });

  it("locks duplicate Sync starts for the same descriptor while P4 is in flight", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(snapshot("DOWNLOADING"));
    let resolveSync: ((value: P2pSyncSnapshot) => void) | undefined;
    mockSyncController.startSync.mockImplementation(
      () => new Promise<P2pSyncSnapshot>((resolve) => {
        resolveSync = resolve;
      }),
    );

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByTestId("invite-sync-invite-1")).toBeTruthy());

    fireEvent.press(view.getByTestId("invite-sync-invite-1"));
    await waitFor(() =>
      expect(view.getByTestId("invite-sync-invite-1").props.accessibilityState.disabled).toBe(true),
    );
    fireEvent.press(view.getByTestId("invite-sync-invite-1"));
    expect(mockSyncController.startSync).toHaveBeenCalledTimes(1);

    resolveSync?.(snapshot("DOWNLOADING"));
    await waitFor(() => expect(view.getByText("Syncing")).toBeTruthy());
  });

  it("keeps a startSync failure fail-closed and exposes the retryable row error", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(snapshot("FAILED"));
    mockSyncController.startSync.mockRejectedValue(new Error("runtime unavailable"));

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Sync")).toBeTruthy());

    fireEvent.press(view.getByTestId("invite-sync-invite-1"));

    await waitFor(() =>
      expect(view.getByTestId("invite-sync-error-invite-1")).toBeTruthy(),
    );
    expect(view.getByText("Could not sync this invitation.")).toBeTruthy();
    expect(view.getByText("Sync error")).toBeTruthy();
    expect(view.queryByText("Available")).toBeNull();
    expect(view.queryByText("Play")).toBeNull();
  });
});


describe("InvitesScreen T2.E Available + Play", () => {
  const verifiedHandle = {
    accountScope: "viewer-1",
    assetId: "asset-1",
    publicationId: "pub-1",
    lineageId: "lineage-1",
    manifestDigestSha256: DESCRIPTOR.manifestDigestSha256,
    externalPublicationId: DESCRIPTOR.externalPublicationId,
  };
  const playbackSession = {
    playbackUrl: `http://127.0.0.1:43210/${"c".repeat(32)}/index.m3u8`,
    accountScope: "viewer-1",
    publicationId: "pub-1",
    lineageId: "lineage-1",
  };

  it("obtains the verified P4 handle before delegating Play to P5", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));
    mockSyncController.getVerifiedPackageHandle.mockResolvedValue(verifiedHandle);
    mockStartPlayback.mockResolvedValue({
      ok: true,
      value: { data: playbackSession, sessionRotation: "play-rotation" },
    });

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Available")).toBeTruthy());

    fireEvent.press(view.getByTestId("invite-play-invite-1"));

    await waitFor(() => expect(mockStartPlayback).toHaveBeenCalledTimes(1));
    expect(mockSyncController.getVerifiedPackageHandle).toHaveBeenCalledWith(
      DESCRIPTOR,
      "viewer-1",
    );
    expect(mockStartPlayback).toHaveBeenCalledWith("session-p6", "auth-1", verifiedHandle);
    expect(mockAuthValue.onSessionRotation).toHaveBeenCalledWith("play-rotation");
    await waitFor(() => expect(view.getByTestId("p2p-player")).toBeTruthy());
    expect(mockPlaybackView).toHaveBeenCalledWith(
      expect.objectContaining({ session: playbackSession }),
    );
  });

  it("does not expose Play for unverified READY", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY"));

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Syncing")).toBeTruthy());

    expect(view.queryByTestId("invite-play-invite-1")).toBeNull();
    expect(mockSyncController.getVerifiedPackageHandle).not.toHaveBeenCalled();
    expect(mockStartPlayback).not.toHaveBeenCalled();
  });

  it("fails closed when the verified P4 handle cannot be produced", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));
    mockSyncController.getVerifiedPackageHandle.mockRejectedValue(
      new Error("package no longer verified"),
    );

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Available")).toBeTruthy());
    fireEvent.press(view.getByTestId("invite-play-invite-1"));

    await waitFor(() => expect(view.getByTestId("invite-play-error-invite-1")).toBeTruthy());
    expect(mockStartPlayback).not.toHaveBeenCalled();
    expect(view.queryByTestId("p2p-player")).toBeNull();
  });

  it("fails closed when P5 rejects current authorization and refreshes authority", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));
    mockSyncController.getVerifiedPackageHandle.mockResolvedValue(verifiedHandle);
    mockStartPlayback.mockResolvedValue({
      ok: false,
      error: { kind: "forbidden" },
    });

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Available")).toBeTruthy());
    fireEvent.press(view.getByTestId("invite-play-invite-1"));

    await waitFor(() => expect(view.getByTestId("invite-play-error-invite-1")).toBeTruthy());
    expect(view.getByText("Playback authorization is no longer available.")).toBeTruthy();
    expect(view.queryByTestId("p2p-player")).toBeNull();
    expect(mockClient.get.mock.calls.length).toBeGreaterThanOrEqual(2);
  });

  it("logs out and creates no player when P5 reports session expiry", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));
    mockSyncController.getVerifiedPackageHandle.mockResolvedValue(verifiedHandle);
    mockStartPlayback.mockResolvedValue({
      ok: false,
      error: { kind: "session_expired" },
    });

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Available")).toBeTruthy());
    fireEvent.press(view.getByTestId("invite-play-invite-1"));

    await waitFor(() => expect(mockAuthValue.logout).toHaveBeenCalledTimes(1));
    expect(view.queryByTestId("p2p-player")).toBeNull();
  });

  it("locks duplicate Play starts while P4/P5 startup is in flight", async () => {
    mockClient.get.mockResolvedValue({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));
    let resolveHandle: ((value: typeof verifiedHandle) => void) | undefined;
    mockSyncController.getVerifiedPackageHandle.mockImplementation(
      () => new Promise((resolve) => {
        resolveHandle = resolve;
      }),
    );
    mockStartPlayback.mockResolvedValue({
      ok: true,
      value: { data: playbackSession, sessionRotation: null },
    });

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByTestId("invite-play-invite-1")).toBeTruthy());

    fireEvent.press(view.getByTestId("invite-play-invite-1"));
    await waitFor(() =>
      expect(view.getByTestId("invite-play-invite-1").props.accessibilityState.disabled).toBe(true),
    );
    fireEvent.press(view.getByTestId("invite-play-invite-1"));
    expect(mockSyncController.getVerifiedPackageHandle).toHaveBeenCalledTimes(1);

    resolveHandle?.(verifiedHandle);
    await waitFor(() => expect(view.getByTestId("p2p-player")).toBeTruthy());
    expect(mockStartPlayback).toHaveBeenCalledTimes(1);
  });
});


describe("InvitesScreen T2.F fail-closed lifecycle", () => {
  function switchAccount(userId: string, sessionRef: string) {
    mockAuthValue = {
      ...mockAuthValue,
      userId,
      sessionRef,
      status: "authed",
      logout: jest.fn(),
      onSessionRotation: jest.fn().mockResolvedValue(undefined),
    };
  }

  it("invalidates a visible Available item and playback session immediately on account change", async () => {
    mockInboxOnce([rawInboxItem()]);
    mockInboxOnce([]);
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));
    mockSyncController.getVerifiedPackageHandle.mockResolvedValue({
      accountScope: "viewer-1",
      assetId: "asset-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
      manifestDigestSha256: DESCRIPTOR.manifestDigestSha256,
      externalPublicationId: DESCRIPTOR.externalPublicationId,
    });
    mockStartPlayback.mockResolvedValue({
      ok: true,
      value: {
        data: {
          playbackUrl: `http://127.0.0.1:43210/${"d".repeat(32)}/index.m3u8`,
          accountScope: "viewer-1",
          publicationId: "pub-1",
          lineageId: "lineage-1",
        },
        sessionRotation: null,
      },
    });

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByText("Available")).toBeTruthy());
    fireEvent.press(view.getByTestId("invite-play-invite-1"));
    await waitFor(() => expect(view.getByTestId("p2p-player")).toBeTruthy());

    switchAccount("viewer-2", "session-p6-2");
    await act(async () => {
      await view.rerender(<InvitesScreen gatewayBaseUrl="http://localhost:3000" />);
    });

    await waitFor(() => expect(view.getByTestId("invites-empty")).toBeTruthy());
    expect(view.queryByTestId("invite-row-invite-1")).toBeNull();
    expect(view.queryByTestId("p2p-player")).toBeNull();
  });

  it("discards a late inbox response from the previous account", async () => {
    let resolveOldInbox: ((value: unknown) => void) | undefined;
    mockClient.get.mockImplementationOnce(
      () => new Promise((resolve) => {
        resolveOldInbox = resolve;
      }),
    );
    mockInboxOnce([]);

    const view = await renderInvitesScreen();
    await waitFor(() => expect(mockClient.get).toHaveBeenCalledTimes(1));

    switchAccount("viewer-2", "session-p6-2");
    await act(async () => {
      await view.rerender(<InvitesScreen gatewayBaseUrl="http://localhost:3000" />);
    });
    await waitFor(() => expect(view.getByTestId("invites-empty")).toBeTruthy());

    resolveOldInbox?.({
      ok: true,
      value: { data: [rawInboxItem()], sessionRotation: null },
    });
    await act(async () => Promise.resolve());

    expect(view.queryByTestId("invite-row-invite-1")).toBeNull();
    expect(view.getByTestId("invites-empty")).toBeTruthy();
  });

  it("discards a verified-handle completion from the previous account before P5 startup", async () => {
    mockInboxOnce([rawInboxItem()]);
    mockInboxOnce([]);
    mockSyncController.getSyncState.mockResolvedValue(snapshot("READY", true));
    let resolveHandle: ((value: unknown) => void) | undefined;
    mockSyncController.getVerifiedPackageHandle.mockImplementation(
      () => new Promise((resolve) => {
        resolveHandle = resolve;
      }),
    );

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByTestId("invite-play-invite-1")).toBeTruthy());
    fireEvent.press(view.getByTestId("invite-play-invite-1"));
    await waitFor(() =>
      expect(mockSyncController.getVerifiedPackageHandle).toHaveBeenCalledTimes(1),
    );

    switchAccount("viewer-2", "session-p6-2");
    await act(async () => {
      await view.rerender(<InvitesScreen gatewayBaseUrl="http://localhost:3000" />);
    });
    await waitFor(() => expect(view.getByTestId("invites-empty")).toBeTruthy());

    resolveHandle?.({
      accountScope: "viewer-1",
      assetId: "asset-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
      manifestDigestSha256: DESCRIPTOR.manifestDigestSha256,
      externalPublicationId: DESCRIPTOR.externalPublicationId,
    });
    await act(async () => Promise.resolve());

    expect(mockStartPlayback).not.toHaveBeenCalled();
    expect(view.queryByTestId("p2p-player")).toBeNull();
  });

  it("clears a raw Claim token when the authenticated account changes", async () => {
    mockInboxOnce([]);
    mockInboxOnce([]);

    const view = await renderInvitesScreen();
    await waitFor(() => expect(view.getByTestId("invites-empty")).toBeTruthy());
    await act(async () => {
      fireEvent.changeText(view.getByTestId("invites-claim-token"), "account-one-secret");
    });
    await waitFor(() =>
      expect(view.getByTestId("invites-claim-token").props.value).toBe("account-one-secret"),
    );

    switchAccount("viewer-2", "session-p6-2");
    await act(async () => {
      await view.rerender(<InvitesScreen gatewayBaseUrl="http://localhost:3000" />);
    });

    await waitFor(() =>
      expect(view.getByTestId("invites-claim-token").props.value).toBe(""),
    );
    expect(mockClaimInvitation).not.toHaveBeenCalled();
  });
});
