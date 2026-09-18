import type { P2PAudienceService } from "../../src/p2p/P2PAudienceService";
import type { P2PService } from "../../src/p2p/P2PService";
import {
  P2PPlaybackAuthorizationError,
  P2PPlaybackController,
} from "../../src/p2p/playback/P2PPlaybackController";
import type { VerifiedP2pPackageHandle } from "../../src/p2p/sync/VerifiedPackageHandle";

const handle: VerifiedP2pPackageHandle = {
  accountScope: "viewer-1",
  assetId: "asset-1",
  publicationId: "publication-1",
  lineageId: "lineage-1",
  manifestDigestSha256: "a".repeat(64),
  externalPublicationId: "b".repeat(64),
};

type AuthorizationOverrides = Partial<{
  assetId: string;
  publicationId: string;
  lineageId: string;
  viewerSubjectId: string;
  expiresAtUnix: number;
}>;

function createAudience(overrides: AuthorizationOverrides = {}) {
  return {
    getAuthorization: jest.fn(async () => ({
      ok: true as const,
      value: {
        data: {
          id: "auth-1",
          invitationId: "invite-1",
          assetId: "asset-1",
          publicationId: "publication-1",
          lineageId: "lineage-1",
          viewerSubjectId: "viewer-1",
          deviceId: "device-1",
          expiresAtUnix: Math.floor(Date.now() / 1000) + 600,
          ...overrides,
        },
        sessionRotation: null,
      },
    })),
    getTransientContentKey: jest.fn(async () => ({
      ok: true as const,
      value: { data: Buffer.alloc(32, 7).toString("base64"), sessionRotation: null },
    })),
  };
}

function createService() {
  return {
    getSnapshot: jest.fn(() => ({ runtimeState: "ready" as const, lastError: null })),
    initialize: jest.fn(async () => ({
      protocolVersion: 1 as const,
      runtimeVersion: "test",
      capabilities: [],
    })),
    shutdown: jest.fn(async () => undefined),
    startProductPlayback: jest.fn(async () => ({
      capability: "product-playback" as const,
      schema_version: 1 as const,
      playback_url: `http://127.0.0.1:43210/${"c".repeat(32)}/index.m3u8`,
    })),
    stopProductPlayback: jest.fn(async () => undefined),
  };
}

describe("P5 playback controller", () => {
  it("requires O3 authorization, unwraps K1, and starts only the verified P4 package", async () => {
    const audience = createAudience();
    const service = createService();
    const controller = new P2PPlaybackController(
      audience as unknown as P2PAudienceService,
      service as unknown as P2PService,
    );

    const result = await controller.start("access-token", "auth-1", handle);

    expect(result).toEqual({
      ok: true,
      value: {
        data: {
          playbackUrl: `http://127.0.0.1:43210/${"c".repeat(32)}/index.m3u8`,
          accountScope: "viewer-1",
          publicationId: "publication-1",
          lineageId: "lineage-1",
        },
        sessionRotation: null,
      },
    });
    expect(audience.getTransientContentKey).toHaveBeenCalledWith("access-token", "auth-1");
    expect(service.startProductPlayback).toHaveBeenCalledWith(expect.objectContaining({
      accountScope: "viewer-1",
      assetId: "asset-1",
      publicationId: "publication-1",
      lineageId: "lineage-1",
      externalPublicationId: "b".repeat(64),
      manifestDigestSha256: "a".repeat(64),
    }));
  });

  it("fails before K1 unwrap when authorization belongs to another viewer", async () => {
    const audience = createAudience({ viewerSubjectId: "viewer-2" });
    const service = createService();
    const controller = new P2PPlaybackController(
      audience as unknown as P2PAudienceService,
      service as unknown as P2PService,
    );

    await expect(controller.start("access-token", "auth-1", handle)).rejects.toBeInstanceOf(
      P2PPlaybackAuthorizationError,
    );
    expect(audience.getTransientContentKey).not.toHaveBeenCalled();
    expect(service.startProductPlayback).not.toHaveBeenCalled();
  });


  it.each([
    ["asset", { assetId: "asset-2" }],
    ["publication", { publicationId: "publication-2" }],
    ["lineage", { lineageId: "lineage-2" }],
    [
      "expiry",
      { expiresAtUnix: Math.floor(Date.now() / 1000) - 1 },
    ],
  ] as const)(
    "fails before K1 unwrap when authorization %s does not match the verified package",
    async (_caseName, overrides) => {
      const audience = createAudience(overrides);
      const service = createService();
      const controller = new P2PPlaybackController(
        audience as unknown as P2PAudienceService,
        service as unknown as P2PService,
      );

      await expect(
        controller.start("access-token", "auth-1", handle),
      ).rejects.toBeInstanceOf(P2PPlaybackAuthorizationError);

      expect(audience.getTransientContentKey).not.toHaveBeenCalled();
      expect(service.startProductPlayback).not.toHaveBeenCalled();
    },
  );

  it("stops the loopback session explicitly", async () => {
    const service = createService();
    const controller = new P2PPlaybackController(
      createAudience() as unknown as P2PAudienceService,
      service as unknown as P2PService,
    );

    await controller.stop();

    expect(service.stopProductPlayback).toHaveBeenCalledTimes(1);
  });
});
