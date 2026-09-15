import type { P2PAudienceService } from "../P2PAudienceService";
import type { P2PPlaybackController, P2PPlaybackSession } from "../playback/P2PPlaybackController";
import type { P2PSyncController } from "../sync/P2PSyncController";

export type P5DeviceCertificationStage =
  | "claim"
  | "sync"
  | "verify"
  | "playback";

export type P5DeviceCertificationFailureCode =
  | "CLAIM_FAILED"
  | "SYNC_FAILED"
  | "VERIFICATION_FAILED"
  | "PLAYBACK_FAILED";

export type P5DeviceCertificationResult =
  | { ok: true; session: P2PPlaybackSession }
  | { ok: false; code: P5DeviceCertificationFailureCode };

type P5DeviceCertificationDependencies = {
  audience: Pick<P2PAudienceService, "claimInvitation">;
  sync: Pick<P2PSyncController, "startSync" | "getVerifiedPackageHandle">;
  playback: Pick<P2PPlaybackController, "start">;
};

type P5DeviceCertificationInput = {
  accessToken: string;
  accountScope: string;
  invitationToken: string;
};

/**
 * Development-only P5 device certification orchestration. It deliberately
 * consumes the production P3/P4/P5 seams rather than introducing a mock CK or
 * alternate playback path. Raw gateway/runtime errors are collapsed to stable
 * failure codes so secrets and transport details never become harness output.
 */
export async function runP5DeviceCertification(
  input: P5DeviceCertificationInput,
  dependencies: P5DeviceCertificationDependencies,
  onStage: (stage: P5DeviceCertificationStage) => void = () => undefined,
): Promise<P5DeviceCertificationResult> {
  onStage("claim");
  let claim;
  try {
    claim = await dependencies.audience.claimInvitation(
      input.accessToken,
      input.invitationToken,
    );
  } catch {
    return { ok: false, code: "CLAIM_FAILED" };
  }
  if (!claim.ok) return { ok: false, code: "CLAIM_FAILED" };

  onStage("sync");
  try {
    await dependencies.sync.startSync(
      claim.value.data.descriptor,
      input.accountScope,
    );
  } catch {
    return { ok: false, code: "SYNC_FAILED" };
  }

  onStage("verify");
  let handle;
  try {
    handle = await dependencies.sync.getVerifiedPackageHandle(
      claim.value.data.descriptor,
      input.accountScope,
    );
  } catch {
    return { ok: false, code: "VERIFICATION_FAILED" };
  }

  onStage("playback");
  try {
    const playback = await dependencies.playback.start(
      input.accessToken,
      claim.value.data.authorization.id,
      handle,
    );
    if (!playback.ok) return { ok: false, code: "PLAYBACK_FAILED" };
    return { ok: true, session: playback.value.data };
  } catch {
    return { ok: false, code: "PLAYBACK_FAILED" };
  }
}
