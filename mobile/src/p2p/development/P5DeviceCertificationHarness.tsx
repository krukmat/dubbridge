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

import { useAuth, type AuthContextValue } from "../../auth/AuthProvider";
import { useP2PService } from "../P2PProvider";
import type { P2PPlaybackSession } from "../playback/P2PPlaybackController";
import { P2PPlaybackSessionView } from "../playback/P2PPlaybackSessionView";
import {
  createP5CertificationDependencies,
  type P5CertificationDependencies,
} from "./P5DeviceCertificationDependencies";
import {
  runP5DeviceCertification,
  type P5DeviceCertificationStage,
} from "./P5DeviceCertification";

type HarnessControls = {
  clearToken(): void;
  setBusy(value: boolean): void;
  setFailureCode(value: string | null): void;
  setSession(value: P2PPlaybackSession | null): void;
  setStage(value: P5DeviceCertificationStage | "idle" | "playing"): void;
};

type CertificationPanelProps = {
  busy: boolean;
  dependencies: P5CertificationDependencies;
  failureCode: string | null;
  invitationToken: string;
  session: P2PPlaybackSession | null;
  stage: P5DeviceCertificationStage | "idle" | "playing";
  onChangeToken(value: string): void;
  onRun(): void;
  onStop(): void;
};

async function executeCertification(
  auth: Pick<AuthContextValue, "sessionRef" | "userId">,
  dependencies: P5CertificationDependencies | null,
  invitationToken: string,
  controls: HarnessControls,
): Promise<void> {
  const token = invitationToken.trim();
  if (!auth.sessionRef || !auth.userId) {
    controls.setFailureCode("AUTH_REQUIRED");
    return;
  }
  if (!dependencies) {
    controls.setFailureCode("CONFIG_INVALID");
    return;
  }
  if (!token) {
    controls.setFailureCode("INVITE_REQUIRED");
    return;
  }

  controls.setBusy(true);
  controls.setFailureCode(null);
  const result = await runP5DeviceCertification(
    {
      accessToken: auth.sessionRef,
      accountScope: auth.userId,
      invitationToken: token,
    },
    dependencies,
    controls.setStage,
  );
  controls.clearToken();
  controls.setBusy(false);
  if (!result.ok) {
    controls.setFailureCode(result.code);
    controls.setStage("idle");
    return;
  }
  controls.setSession(result.session);
  controls.setStage("playing");
}

async function stopCertification(
  dependencies: P5CertificationDependencies,
  controls: HarnessControls,
): Promise<void> {
  controls.setBusy(true);
  try {
    await dependencies.playback.stop();
    controls.setFailureCode(null);
  } catch {
    controls.setFailureCode("STOP_FAILED");
  } finally {
    controls.setSession(null);
    controls.setStage("idle");
    controls.setBusy(false);
  }
}

function CertificationPanel(props: CertificationPanelProps) {
  const { busy, dependencies, failureCode, invitationToken } = props;
  const { session, stage } = props;
  return (
    <Modal visible animationType="fade" onRequestClose={() => undefined}>
      <SafeAreaView style={styles.screen}>
        <View style={styles.panel}>
          <Text style={styles.title}>P5 device certification</Text>
          <Text style={styles.note}>
            Development-only. Uses real P3/P4/P5 seams; no mock content key or
            remote media fallback.
          </Text>
          {session ? (
            <>
              <P2PPlaybackSessionView
                session={session}
                controller={dependencies.playback}
                testID="p5-cert-player"
              />
              <Text style={styles.status}>stage={stage}</Text>
              <Button
                title="Stop certification playback"
                onPress={props.onStop}
                disabled={busy}
              />
            </>
          ) : (
            <>
              <TextInput
                value={invitationToken}
                onChangeText={props.onChangeToken}
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
                onPress={props.onRun}
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

/** Android-only development surface for P5.T3 using production P3/P4/P5 seams. */
export function P5DeviceCertificationHarness({ enabled }: { enabled: boolean }) {
  const auth = useAuth();
  const service = useP2PService();
  const dependencies = useMemo(
    () => createP5CertificationDependencies(enabled, service),
    [enabled, service],
  );
  const [invitationToken, setInvitationToken] = useState("");
  const [stage, setStage] = useState<
    P5DeviceCertificationStage | "idle" | "playing"
  >("idle");
  const [failureCode, setFailureCode] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [session, setSession] = useState<P2PPlaybackSession | null>(null);
  if (
    !enabled ||
    Platform.OS !== "android" ||
    auth.status !== "authed" ||
    !dependencies
  ) {
    return null;
  }

  const controls: HarnessControls = {
    clearToken: () => setInvitationToken(""),
    setBusy,
    setFailureCode,
    setSession,
    setStage,
  };
  return (
    <CertificationPanel
      busy={busy}
      dependencies={dependencies}
      failureCode={failureCode}
      invitationToken={invitationToken}
      session={session}
      stage={stage}
      onChangeToken={setInvitationToken}
      onRun={() =>
        void executeCertification(auth, dependencies, invitationToken, controls)
      }
      onStop={() => void stopCertification(dependencies, controls)}
    />
  );
}

const styles = StyleSheet.create({
  screen: {
    flex: 1,
    backgroundColor: "#111111",
    justifyContent: "center",
    padding: 24,
  },
  panel: { gap: 16 },
  title: { color: "#ffffff", fontSize: 24, fontWeight: "700" },
  note: { color: "#cccccc", lineHeight: 20 },
  input: {
    backgroundColor: "#ffffff",
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
  },
  status: { color: "#cccccc" },
  failure: { color: "#ff8a80", fontWeight: "600" },
});
