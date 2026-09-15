import type { GatewayResult } from "../../api/client";
import type { P2PAudienceService } from "../P2PAudienceService";
import type { P2PService } from "../P2PService";
import type { VerifiedP2pPackageHandle } from "../sync/VerifiedPackageHandle";

export type P2PPlaybackSession = Readonly<{
  playbackUrl: string;
  accountScope: string;
  publicationId: string;
  lineageId: string;
}>;

export class P2PPlaybackAuthorizationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "P2PPlaybackAuthorizationError";
  }
}

/** P5 coordinator: authorization + K1 unwrap + verified package -> local loopback URL. */
export class P2PPlaybackController {
  constructor(
    private readonly audience: P2PAudienceService,
    private readonly service: P2PService,
  ) {}

  async start(
    accessToken: string,
    authorizationId: string,
    handle: VerifiedP2pPackageHandle,
  ): Promise<GatewayResult<P2PPlaybackSession>> {
    const authorization = await this.audience.getAuthorization(accessToken, authorizationId);
    if (!authorization.ok) return authorization;
    this.assertAuthorization(handle, authorization.value.data);
    await this.ensureRuntimeReady();

    const keyResult = await this.audience.getTransientContentKey(accessToken, authorizationId);
    if (!keyResult.ok) return keyResult;
    let ckBase64: string | null = keyResult.value.data;
    try {
      const receipt = await this.service.startProductPlayback({
        accountScope: handle.accountScope,
        assetId: handle.assetId,
        externalPublicationId: handle.externalPublicationId,
        lineageId: handle.lineageId,
        manifestDigestSha256: handle.manifestDigestSha256,
        publicationId: handle.publicationId,
        ckBase64,
      });
      return {
        ok: true,
        value: {
          data: {
            playbackUrl: receipt.playback_url,
            accountScope: handle.accountScope,
            publicationId: handle.publicationId,
            lineageId: handle.lineageId,
          },
          sessionRotation: keyResult.value.sessionRotation,
        },
      };
    } finally {
      ckBase64 = null;
    }
  }

  stop(): Promise<void> {
    return this.service.stopProductPlayback();
  }

  private assertAuthorization(
    handle: VerifiedP2pPackageHandle,
    authorization: {
      assetId: string;
      publicationId: string;
      lineageId: string;
      viewerSubjectId: string;
      expiresAtUnix: number;
    },
  ): void {
    if (
      authorization.assetId !== handle.assetId ||
      authorization.publicationId !== handle.publicationId ||
      authorization.lineageId !== handle.lineageId ||
      authorization.viewerSubjectId !== handle.accountScope ||
      authorization.expiresAtUnix <= Math.floor(Date.now() / 1000)
    ) {
      throw new P2PPlaybackAuthorizationError("P2P playback authorization does not match the verified package");
    }
  }

  private async ensureRuntimeReady(): Promise<void> {
    const state = this.service.getSnapshot().runtimeState;
    if (state === "ready") return;
    if (state === "failed") await this.service.shutdown();
    await this.service.initialize();
  }
}
