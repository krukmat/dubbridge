import { useEffect, useState } from "react";
import { StyleSheet, Text } from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, type } from "../theme";
import type { AssetSummary } from "./AssetListScreen";

type AssetDetailScreenProps = {
  assetId: string;
  gatewayBaseUrl: string;
  onOpenCompliance: () => void;
};

type AssetDetailViewState =
  | { kind: "loading" }
  | { kind: "ready"; asset: AssetSummary }
  | { kind: "error"; message: string }
  | { kind: "not_available" };

function formatStatus(status: string): string {
  return status
    .split("_")
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

export function AssetDetailScreen({
  assetId,
  gatewayBaseUrl,
  onOpenCompliance,
}: AssetDetailScreenProps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<AssetDetailViewState>({
    kind: "loading",
  });

  useEffect(() => {
    let isActive = true;

    async function loadAssetDetail(): Promise<void> {
      setViewState({ kind: "loading" });

      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.get<AssetSummary>(
        `/api/assets/${assetId}`,
        auth.sessionRef,
      );

      if (!isActive) {
        return;
      }

      if (!result.ok) {
        if (result.error.kind === "session_expired") {
          await auth.logout();
          return;
        }

        if (result.error.kind === "http" && result.error.status === 404) {
          setViewState({ kind: "not_available" });
          return;
        }

        const message =
          result.error.kind === "network"
            ? result.error.message
            : result.error.kind === "forbidden"
              ? "You do not have access to this asset."
              : `Gateway request failed with status ${result.error.status}.`;
        setViewState({ kind: "error", message });
        return;
      }

      await auth.onSessionRotation(result.value.sessionRotation);

      if (!isActive) {
        return;
      }

      setViewState({ kind: "ready", asset: result.value.data });
    }

    void loadAssetDetail();

    return () => {
      isActive = false;
    };
  }, [assetId, auth, gatewayBaseUrl]);

  return (
    <Screen testID="asset-detail-screen" scroll edges={["bottom"]}>
      <ScreenHeader kicker="Asset" title="Asset detail" />

      {viewState.kind === "loading" ? (
        <StateView kind="loading" title="Loading asset detail…" />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load asset detail"
          message={viewState.message}
        />
      ) : null}

      {viewState.kind === "not_available" ? (
        <StateView
          kind="empty"
          title="Asset detail not available yet"
          message="This asset detail surface is not available on the current backend."
        />
      ) : null}

      {viewState.kind === "ready" ? (
        <>
          <Panel>
            <Text style={styles.assetTitle}>{viewState.asset.title}</Text>
            <Text style={styles.metaLabel}>Status</Text>
            <Badge
              label={formatStatus(viewState.asset.status)}
              tone={statusTone(viewState.asset.status)}
            />
            <Text style={styles.metaLabel}>Asset ID</Text>
            <Text style={styles.metaValue}>{viewState.asset.id}</Text>
            <Text style={styles.metaLabel}>Uploader ID</Text>
            <Text style={styles.metaValue}>{viewState.asset.uploader_id}</Text>
          </Panel>

          <Panel>
            <Text style={styles.panelTitle}>Compliance and consent</Text>
            <Text style={styles.panelCopy}>
              Review the immutable audit trail, rights evidence, and voice consent ledger.
            </Text>
            <Button
              testID="asset-open-compliance"
              label="Open compliance center"
              onPress={onOpenCompliance}
            />
          </Panel>
        </>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  assetTitle: { ...type.title, color: color.ink900 },
  metaLabel: { ...type.label, color: color.ink400 },
  metaValue: { ...type.meta, color: color.ink700 },
  panelTitle: { ...type.heading, color: color.ink900 },
  panelCopy: { ...type.body, color: color.ink500 },
});
