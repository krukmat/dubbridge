import { StyleSheet, View } from "react-native";

import { space } from "../theme";
import { StateView } from "./StateView";
import { VideoPlayer } from "./VideoPlayer";

export type PlaybackViewState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; source: string }
  | { kind: "not_ready" }
  | { kind: "error"; message: string };

type PlaybackStateViewProps = {
  state: PlaybackViewState;
  testIdPrefix: string;
  testIdPlayer?: string;
  onRetry: () => void;
};

export function PlaybackStateView({ state, testIdPrefix, testIdPlayer, onRetry }: PlaybackStateViewProps) {
  if (state.kind === "loading") {
    return (
      <View style={styles.surface}>
        <StateView
          testID={`${testIdPrefix}-loading`}
          kind="loading"
          title="Loading playback…"
          message="Preparing the original track."
        />
      </View>
    );
  }
  if (state.kind === "not_ready") {
    return (
      <View style={styles.surface}>
        <StateView
          testID={`${testIdPrefix}-empty`}
          kind="empty"
          title="Media not ready yet"
          message="Playback is not available for this asset yet."
        />
      </View>
    );
  }
  if (state.kind === "error") {
    return (
      <View style={styles.surface}>
        <StateView
          testID={`${testIdPrefix}-error`}
          kind="error"
          title="Could not load playback"
          message={state.message}
          onRetry={onRetry}
        />
      </View>
    );
  }
  if (state.kind === "ready") {
    return (
      <VideoPlayer
        testID={testIdPlayer ?? testIdPrefix}
        source={state.source}
        onRetry={onRetry}
      />
    );
  }
  return null;
}

const styles = StyleSheet.create({
  surface: { minHeight: 220, marginTop: space.md },
});
