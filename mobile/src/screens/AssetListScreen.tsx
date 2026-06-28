import { useCallback, useEffect, useState } from "react";
import {
  FlatList,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { formatStatusLabel } from "../format";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, fieldStyle, space, type } from "../theme";

export type AssetSummary = {
  id: string;
  title: string;
  uploader_id: string;
  status: string;
  created_at: string;
  updated_at: string;
};

type AssetListScreenProps = {
  gatewayBaseUrl: string;
  onOpenAsset: (asset: AssetSummary) => void;
  onOpenUpload?: () => void;
};

type AssetListViewState =
  | { kind: "loading" }
  | { kind: "ready"; assets: AssetSummary[] }
  | { kind: "empty" }
  | { kind: "error"; message: string };

function toAssetListViewState(assets: AssetSummary[]): AssetListViewState {
  return assets.length === 0 ? { kind: "empty" } : { kind: "ready", assets };
}

function assetListErrorMessage(error: { kind: string; message?: string; status?: number }) {
  return error.kind === "network"
    ? error.message ?? "Network request failed."
    : error.kind === "forbidden"
      ? "You do not have access to the asset list."
      : `Request failed with status ${error.status}.`;
}

async function fetchAssetList(
  gatewayBaseUrl: string,
  sessionRef: string | null,
) {
  const client = createGatewayClient({ gatewayBaseUrl });
  return client.get<AssetSummary[]>("/api/assets", sessionRef);
}

function StatusFilterChips({ chips, statusFilter, onSelectStatus }: {
  chips: string[];
  statusFilter: string;
  onSelectStatus: (s: string) => void;
}) {
  return (
    <ScrollView horizontal showsHorizontalScrollIndicator={false} contentContainerStyle={styles.chipRow}>
      {chips.map((chip) => (
        <Button
          key={chip}
          testID={`asset-filter-${chip}`}
          label={chip === "all" ? "All" : formatStatusLabel(chip)}
          size="sm"
          variant={statusFilter === chip ? "primary" : "secondary"}
          onPress={() => onSelectStatus(chip)}
        />
      ))}
    </ScrollView>
  );
}

function LibraryFilterBar({
  query,
  onChangeQuery,
  statusFilter,
  onSelectStatus,
  availableStatuses,
}: {
  query: string;
  onChangeQuery: (q: string) => void;
  statusFilter: string;
  onSelectStatus: (s: string) => void;
  availableStatuses: string[];
}) {
  const chips = ["all", ...availableStatuses];
  return (
    <View style={styles.filterBar}>
      <TextInput
        testID="asset-search"
        style={fieldStyle}
        placeholder="Search assets…"
        value={query}
        onChangeText={onChangeQuery}
        autoCapitalize="none"
        autoCorrect={false}
        clearButtonMode="while-editing"
        accessibilityLabel="Search assets"
      />
      {availableStatuses.length > 0 ? (
        <StatusFilterChips chips={chips} statusFilter={statusFilter} onSelectStatus={onSelectStatus} />
      ) : null}
    </View>
  );
}

function AssetRow({
  asset,
  onOpenAsset,
}: {
  asset: AssetSummary;
  onOpenAsset: (asset: AssetSummary) => void;
}) {
  return (
    <Card
      testID={`asset-card-${asset.id}`}
      onPress={() => onOpenAsset(asset)}
      trailing="chevron"
      mediaTone={statusTone(asset.status)}
    >
      <Text style={styles.assetTitle} numberOfLines={2}>{asset.title}</Text>
      <Badge
        label={formatStatusLabel(asset.status)}
        tone={statusTone(asset.status)}
      />
    </Card>
  );
}

function AssetListBody({
  assets,
  isNoResults,
  refreshing,
  onRefresh,
  onOpenAsset,
  onOpenUpload,
}: {
  assets: AssetSummary[];
  isNoResults: boolean;
  refreshing: boolean;
  onRefresh: () => void;
  onOpenAsset: (asset: AssetSummary) => void;
  onOpenUpload?: () => void;
}) {
  return (
    <FlatList
      style={styles.scroll}
      contentContainerStyle={assets.length === 0 ? styles.emptyContent : styles.listContent}
      data={assets}
      keyExtractor={(asset) => asset.id}
      renderItem={({ item: asset }) => (
        <AssetRow asset={asset} onOpenAsset={onOpenAsset} />
      )}
      ListEmptyComponent={
        isNoResults ? (
          <StateView
            testID="asset-list-no-results"
            kind="empty"
            title="No results"
            message="No assets match your search."
          />
        ) : (
          <StateView
            testID="asset-list-empty-state"
            kind="empty"
            title="No assets yet"
            message="Your workspace does not have any assets to show."
            primaryAction={
              onOpenUpload
                ? { label: "Upload asset", onPress: onOpenUpload, testID: "asset-list-empty-cta" }
                : undefined
            }
          />
        )
      }
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
      }
    />
  );
}

function useAssetListState(
  gatewayBaseUrl: string,
  sessionRef: string | null,
  logout: () => Promise<void>,
  onSessionRotation: (rotation: string | null) => Promise<void>,
) {
  const [viewState, setViewState] = useState<AssetListViewState>({ kind: "loading" });
  const [refreshing, setRefreshing] = useState(false);

  const loadAssets = useCallback(async (): Promise<void> => {
    const result = await fetchAssetList(gatewayBaseUrl, sessionRef);

    if (result.ok) {
      await onSessionRotation(result.value.sessionRotation);
      setViewState(toAssetListViewState(result.value.data));
      return;
    }
    if (result.error.kind === "session_expired") {
      await logout();
      return;
    }
    setViewState({ kind: "error", message: assetListErrorMessage(result.error) });
  }, [gatewayBaseUrl, logout, onSessionRotation, sessionRef]);

  useEffect(() => {
    void (async () => {
      setViewState({ kind: "loading" });
      await loadAssets();
    })();
  }, [loadAssets]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadAssets();
    setRefreshing(false);
  }, [loadAssets]);

  const onRetry = useCallback(() => {
    setViewState({ kind: "loading" });
    void loadAssets();
  }, [loadAssets]);

  return { viewState, refreshing, onRefresh, onRetry };
}

export function AssetListScreen({
  gatewayBaseUrl,
  onOpenAsset,
  onOpenUpload,
}: AssetListScreenProps) {
  const auth = useAuth();
  const { viewState, refreshing, onRefresh, onRetry } = useAssetListState(
    gatewayBaseUrl,
    auth.sessionRef,
    auth.logout,
    auth.onSessionRotation,
  );

  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");

  const loadedAssets = viewState.kind === "ready" ? viewState.assets : [];
  const availableStatuses = [...new Set(loadedAssets.map((a) => a.status))];
  const filteredAssets = loadedAssets
    .filter((a) => (query ? a.title.toLowerCase().includes(query.toLowerCase()) : true))
    .filter((a) => (statusFilter !== "all" ? a.status === statusFilter : true));
  const isNoResults = loadedAssets.length > 0 && filteredAssets.length === 0;

  return (
    <Screen testID="asset-list-screen">
      <ScreenHeader kicker="Assets" title="Your library" copy="Browse and manage your media." />

      {viewState.kind === "loading" ? (
        <StateView
          kind="loading"
          title="Loading assets…"
          message="Fetching your assets from the gateway."
        />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load assets"
          message={viewState.message}
          onRetry={onRetry}
        />
      ) : null}

      {viewState.kind === "empty" ? (
        <StateView
          testID="asset-list-empty-state"
          kind="empty"
          title="No assets yet"
          message="Your workspace does not have any assets to show."
          primaryAction={
            onOpenUpload
              ? { label: "Upload asset", onPress: onOpenUpload, testID: "asset-list-empty-cta" }
              : undefined
          }
        />
      ) : null}

      {viewState.kind === "ready" ? (
        <>
          <LibraryFilterBar
            query={query}
            onChangeQuery={setQuery}
            statusFilter={statusFilter}
            onSelectStatus={setStatusFilter}
            availableStatuses={availableStatuses}
          />
          <AssetListBody
            assets={filteredAssets}
            isNoResults={isNoResults}
            refreshing={refreshing}
            onRefresh={onRefresh}
            onOpenAsset={onOpenAsset}
            onOpenUpload={onOpenUpload}
          />
        </>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  scroll: { flex: 1 },
  listContent: { gap: space.md, paddingBottom: space.xl },
  emptyContent: { flexGrow: 1 },
  filterBar: { gap: space.sm },
  chipRow: { flexDirection: "row", gap: space.sm },
  assetTitle: { ...type.heading, color: color.ink900 },
  assetMeta: { ...type.meta, color: color.ink500 },
});
