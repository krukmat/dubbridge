import { useMemo, useState } from "react";
import {
  Button,
  Modal,
  Platform,
  SafeAreaView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";

import { createGatewayClient } from "../../api/client";
import { useAuth } from "../../auth/AuthProvider";
import { readRuntimeConfig } from "../../config/env";
import { P2PAudienceService } from "../P2PAudienceService";
import { useP2PService } from "../P2PProvider";
import { P2PPlaybackController, type P2PPlaybackSession } from "../playback/P2PPlaybackController";
import { P2PPlaybackSessionView } from "../playback/P2PPlaybackSessionView";
import { P2PSyncController } from "../sync/P2PSyncController";
import {
  runP5DeviceCertification,
  type P5DeviceCertificationStage,
} from "./P5DeviceCertification";

type CertificationDependencies = {
  audience: P2PAudienceService;
  sync: P2PSyncController;
  playback: P2PPlaybackController;
};

function createDependencies(
  enabled: boolean,
  service: ReturnType<typeof useP2PService>,
): CertificationDependencies | null {
  if (!enabled || Platform.OS !== "android") return null;
  const config = readRuntimeConfig();
  if (!config.ok) return null;
  const audience = new P2PAudienceService(
    createGatewayClient({ gatewayBaseUrl: config.value.gatewayBaseUrl }),
  );
  return {
    audience,
    sync: new P2PSyncController(service),
    playback: new P2PPlaybackController(audience, service),
  };
}

/**
 * Android-only development surface for P5.T3. It is intentionally absent from
 * normal builds and uses the production claim/sync/verification/playback seams.
 * The invitation token is entered at runtime and is never logged.
 */
export function P5DeviceCertificationHarness({ enabled }: { enabled: boolean }) {
  const auth = useAuth();
  const service = useP2PService();
  const dependencies = useMemo(
    () => createDependencies(enabled, service),
    [enabled, service],
  );
  const [invitationToken, setInvitationToken] = useState("");
  const [stage, setStage] = useState<P5DeviceCertificationStage | "idle" | "playing">("idle");
  const [failureCode, setFailureCode] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [session, setSession] = useState<P2PPlaybackSession | null>(null);

  if (!enabled || Platform.OS !== "android" || auth.status !== "authed") return null;

  async function runCertification(): Promise<void> {
    const token = invitationToken.trim();
    if (!auth.sessionRef || !auth.userId) {
      setFailureCode("AUTH_REQUIRED");
      return;
    }
    if (!dependencies) {
      setFailureCode("CONFIG_INVALID");
      return;
    }
    if (!token) {
      setFailureCode("INVITE_REQUIRED");
      return;
    }

    setBusy(true);
    setFailureCode(null);
    const result = await runP5DeviceCertification(
      {
        accessToken: auth.sessionRef,
        accountScope: auth.userId,
        invitationToken: token,
      },
      dependencies,
      setStage,
    );
    setInvitationToken("");
    setBusy(false);

    if (!result.ok) {
      setFailureCode(result.code);
      setStage("idle");
      return;
    }

    setSession(result.session);
    setStage("playing");
  }

  async function stopCertification(): Promise<void> {
    if (!dependencies) return;
    setBusy(true);
    try {
      await dependencies.playback.stop();
      setFailureCode(null);
    } catch {
      setFailureCode("STOP_FAILED");
    } finally {
      setSession(null);
      setStage("idle");
      setBusy(false);
    }
  }

  return (
    <Modal visible animationType="fade" onRequestClose={() => undefined}>
      <SafeAreaView style={styles.screen}>
        <View style={styles.panel}>
          <Text style={styles.title}>P5 device certification</Text>
          <Text style={styles.note}>
            Development-only. Uses real P3/P4/P5 seams; no mock content key or remote media fallback.
          </Text>

          {session ? (
            <>
              <P2PPlaybackSessionView
                session={session}
                controller={dependencies!.playback}
                testID="p5-cert-player"
              />
              <Text style={styles.status}>stage={stage}</Text>
              <Button title="Stop certification playback" onPress={() => void stopCertification()} disabled={busy} />
            </>
          ) : (
            <>
              <TextInput
                value={invitationToken}
                onChangeText={setInvitationToken}
                placeholder="One-time P2P invitation token"
                autoCapitalize="none"
                autoCorrect={false}
                secureTextEntry
                editable={!busy}
                style={styles.input}
                testID="p5-cert-invite-token"
              />
              <Button
                title={busy ? "Running…" : "Run P5 certification"}
                onPress={() => void runCertification()}
                disabled={busy}
              />
              <Text style={styles.status}>stage={stage}</Text>
            </>
          )}

          {failureCode ? (
            <Text style={styles.failure} testID="p5-cert-failure">
              {failureCode}
            </Text>
          ) : null}
        </View>
      </SafeAreaView>
    </Modal>
  );
}

const styles = StyleSheet.create({
  screen: {
    flex: 1,
    backgroundColor: "#111111",
    justifyContent: "center",
    padding: 24,
  },
  panel: {
    gap: 16,
  },
  title: {
    color: "#ffffff",
    fontSize: 24,
    fontWeight: "700",
  },
  note: {
    color: "#cccccc",
    lineHeight: 20,
  },
  input: {
    backgroundColor: "#ffffff",
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
  },
  status: {
    color: "#cccccc",
  },
  failure: {
    color: "#ff8a80",
    fontWeight: "600",
  },
});
