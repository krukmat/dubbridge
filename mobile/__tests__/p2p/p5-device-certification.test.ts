import { runP5DeviceCertification } from "../../src/p2p/development/P5DeviceCertification";

const descriptor = {
  descriptorVersion: "1",
  assetId: "asset-1",
  publicationId: "pub-1",
  lineageId: "lin-1",
  manifestVersion: "1",
  manifestDigestSha256: "a".repeat(64),
  externalPublicationId: "ext-1",
  ckWrapRef: "wrap-1",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-15T12:00:00Z",
};

const claimValue = {
  invitation: {
    id: "invite-1",
    assetId: "asset-1",
    publicationId: "pub-1",
    lineageId: "lin-1",
    ownerSubjectId: "owner-1",
    status: "claimed" as const,
    expiresAtUnix: 4_000_000_000,
    claimedAtUnix: 2_000_000_000,
  },
  authorization: {
    id: "authz-1",
    invitationId: "invite-1",
    assetId: "asset-1",
    publicationId: "pub-1",
    lineageId: "lin-1",
    viewerSubjectId: "viewer-1",
    deviceId: "device-1",
    expiresAtUnix: 4_000_000_000,
  },
  descriptor,
};

const handle = {
  accountScope: "viewer-1",
  assetId: "asset-1",
  publicationId: "pub-1",
  lineageId: "lin-1",
  manifestDigestSha256: "a".repeat(64),
  externalPublicationId: "ext-1",
};

const session = {
  playbackUrl: "http://127.0.0.1:12345/0123456789abcdef0123456789abcdef/index.m3u8",
  accountScope: "viewer-1",
  publicationId: "pub-1",
  lineageId: "lin-1",
};

function createDependencies() {
  return {
    audience: {
      claimInvitation: jest.fn().mockResolvedValue({
        ok: true,
        value: { data: claimValue, sessionRotation: null },
      }),
    },
    sync: {
      startSync: jest.fn().mockResolvedValue({}),
      getVerifiedPackageHandle: jest.fn().mockResolvedValue(handle),
    },
    playback: {
      start: jest.fn().mockResolvedValue({
        ok: true,
        value: { data: session, sessionRotation: null },
      }),
    },
  };
}

describe("P5 device certification orchestration", () => {
  it("uses claim -> sync -> verified handle -> production playback in order", async () => {
    const dependencies = createDependencies();
    const stages: string[] = [];

    const result = await runP5DeviceCertification(
      {
        accessToken: "access-token",
        accountScope: "viewer-1",
        invitationToken: "one-time-invite",
      },
      dependencies,
      (stage) => stages.push(stage),
    );

    expect(result).toEqual({ ok: true, session });
    expect(stages).toEqual(["claim", "sync", "verify", "playback"]);
    expect(dependencies.audience.claimInvitation).toHaveBeenCalledWith(
      "access-token",
      "one-time-invite",
    );
    expect(dependencies.sync.startSync).toHaveBeenCalledWith(descriptor, "viewer-1");
    expect(dependencies.sync.getVerifiedPackageHandle).toHaveBeenCalledWith(
      descriptor,
      "viewer-1",
    );
    expect(dependencies.playback.start).toHaveBeenCalledWith(
      "access-token",
      "authz-1",
      handle,
    );
  });

  it("fails closed before sync when claim is denied", async () => {
    const dependencies = createDependencies();
    dependencies.audience.claimInvitation.mockResolvedValue({
      ok: false,
      error: { kind: "forbidden" },
    });

    await expect(
      runP5DeviceCertification(
        {
          accessToken: "access-token",
          accountScope: "viewer-1",
          invitationToken: "denied-invite",
        },
        dependencies,
      ),
    ).resolves.toEqual({ ok: false, code: "CLAIM_FAILED" });

    expect(dependencies.sync.startSync).not.toHaveBeenCalled();
    expect(dependencies.playback.start).not.toHaveBeenCalled();
  });

  it("stops at sync when replication fails", async () => {
    const dependencies = createDependencies();
    const stages: string[] = [];
    dependencies.sync.startSync.mockRejectedValue(new Error("raw transport detail"));

    await expect(
      runP5DeviceCertification(
        {
          accessToken: "access-token",
          accountScope: "viewer-1",
          invitationToken: "one-time-invite",
        },
        dependencies,
        (stage) => stages.push(stage),
      ),
    ).resolves.toEqual({ ok: false, code: "SYNC_FAILED" });

    expect(stages).toEqual(["claim", "sync"]);
    expect(dependencies.sync.getVerifiedPackageHandle).not.toHaveBeenCalled();
    expect(dependencies.playback.start).not.toHaveBeenCalled();
  });

  it("does not start playback when verified-handle creation fails", async () => {
    const dependencies = createDependencies();
    dependencies.sync.getVerifiedPackageHandle.mockRejectedValue(
      new Error("package is not verified READY"),
    );

    await expect(
      runP5DeviceCertification(
        {
          accessToken: "access-token",
          accountScope: "viewer-1",
          invitationToken: "one-time-invite",
        },
        dependencies,
      ),
    ).resolves.toEqual({ ok: false, code: "VERIFICATION_FAILED" });

    expect(dependencies.playback.start).not.toHaveBeenCalled();
  });

  it("treats an explicit playback denial as fail-closed", async () => {
    const dependencies = createDependencies();
    dependencies.playback.start.mockResolvedValue({
      ok: false,
      error: { kind: "forbidden", message: "raw authorization detail" },
    });

    await expect(
      runP5DeviceCertification(
        {
          accessToken: "access-token",
          accountScope: "viewer-1",
          invitationToken: "one-time-invite",
        },
        dependencies,
      ),
    ).resolves.toEqual({ ok: false, code: "PLAYBACK_FAILED" });
  });

  it("collapses playback failures to a redacted stable code", async () => {
    const dependencies = createDependencies();
    dependencies.playback.start.mockRejectedValue(
      new Error("raw device envelope secret must not leak"),
    );

    await expect(
      runP5DeviceCertification(
        {
          accessToken: "access-token",
          accountScope: "viewer-1",
          invitationToken: "one-time-invite",
        },
        dependencies,
      ),
    ).resolves.toEqual({ ok: false, code: "PLAYBACK_FAILED" });
  });
});
