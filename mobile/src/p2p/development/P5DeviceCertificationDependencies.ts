import { Platform } from "react-native";

import { createGatewayClient } from "../../api/client";
import { readRuntimeConfig } from "../../config/env";
import { P2PAudienceService } from "../P2PAudienceService";
import type { P2PService } from "../P2PService";
import { P2PPlaybackController } from "../playback/P2PPlaybackController";
import { P2PSyncController } from "../sync/P2PSyncController";

export type P5CertificationDependencies = {
  audience: P2PAudienceService;
  sync: P2PSyncController;
  playback: P2PPlaybackController;
};

export function createP5CertificationDependencies(
  enabled: boolean,
  service: P2PService,
): P5CertificationDependencies | null {
  if (!enabled || Platform.OS !== "android") return null;
  const config = readRuntimeConfig();
  if (!config.ok) return null;
  const client = createGatewayClient({
    gatewayBaseUrl: config.value.gatewayBaseUrl,
  });
  const audience = new P2PAudienceService(client);
  return {
    audience,
    sync: new P2PSyncController(service),
    playback: new P2PPlaybackController(audience, service),
  };
}
