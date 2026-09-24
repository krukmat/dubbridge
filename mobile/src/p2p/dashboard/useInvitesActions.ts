import { useCallback, useMemo, useRef, useState } from "react";

import { createGatewayClient } from "../../api/client";
import { useAuth } from "../../auth/AuthProvider";
import { P2PAudienceService } from "../P2PAudienceService";
import { claimInvitationErrorMessage } from "./InvitesModel";

export function useInvitesActions(
  gatewayBaseUrl: string,
  refreshInbox: () => Promise<void>,
) {
  const auth = useAuth();
  const audience = useMemo(
    () => new P2PAudienceService(createGatewayClient({ gatewayBaseUrl })), [gatewayBaseUrl],
  );
  const submitting = useRef(false);
  const [claimToken, setClaimToken] = useState("");
  const [claimError, setClaimError] = useState<string | null>(null);
  const [isClaiming, setIsClaiming] = useState(false);

  const updateClaimToken = useCallback((value: string) => {
    setClaimToken(value);
    setClaimError(null);
  }, []);

  const claim = useCallback(async () => {
    const token = claimToken.trim();
    const accessToken = auth.sessionRef;
    if (!accessToken || token.length === 0 || submitting.current) return;

    submitting.current = true;
    setIsClaiming(true);
    setClaimError(null);
    try {
      const result = await audience.claimInvitation(accessToken, token);
      if (!result.ok) {
        if (result.error.kind === "session_expired") {
          setClaimToken("");
          await auth.logout();
          return;
        }
        setClaimError(claimInvitationErrorMessage(result.error));
        return;
      }

      await auth.onSessionRotation(result.value.sessionRotation);
      setClaimToken("");
      await refreshInbox();
    } catch {
      setClaimError("Could not claim the invitation.");
    } finally {
      submitting.current = false;
      setIsClaiming(false);
    }
  }, [audience, auth.logout, auth.onSessionRotation, auth.sessionRef, claimToken, refreshInbox]);

  return {
    claimToken,
    updateClaimToken,
    claim,
    claimError,
    isClaiming,
    canClaim: claimToken.trim().length > 0 && auth.sessionRef !== null && !isClaiming,
  };
}
