import { useEventListener } from "expo";
import { VideoView, useVideoPlayer, type VideoContentFit, type VideoSource } from "expo-video";
import { useEffect, useState } from "react";
import { StyleSheet, Text, View, type StyleProp, type ViewStyle } from "react-native";

import { color, radius, space, type } from "../theme";
import { StateView } from "./StateView";
import {
  createVideoPlayerState,
  createVideoPlayerShellSnapshot,
  reduceVideoPlayerState,
  type VideoPlayerOverlayState,
} from "./video-player-state";

export type VideoPlayerProps = {
  source: VideoSource;
  testID?: string;
  style?: StyleProp<ViewStyle>;
  contentFit?: VideoContentFit;
  accessibilityLabel?: string;
  onRetry?: () => void;
};

/**
 * Video shell for the mobile playback surface. The state machine lives in the
 * pure `video-player-state.ts` module; this component only forwards player
 * events into that reducer and renders the derived overlay state.
 */
export function VideoPlayer({
  source,
  testID,
  style,
  contentFit = "contain",
  accessibilityLabel = "Video player",
  onRetry,
}: VideoPlayerProps) {
  const [playerState, setPlayerState] = useState(() => createVideoPlayerState(source));
  const player = useVideoPlayer(source, (instance) => {
    instance.loop = false;
  });
  const shellSnapshot = createVideoPlayerShellSnapshot(source, playerState);

  useEffect(() => {
    setPlayerState((current) =>
      reduceVideoPlayerState(current, { type: "source_changed", source }),
    );
  }, [source]);

  useEventListener(player, "statusChange", ({ status, error }) => {
    if (status === "loading") {
      setPlayerState((current) => reduceVideoPlayerState(current, { type: "loading" }));
      return;
    }

    if (status === "readyToPlay") {
      setPlayerState((current) => reduceVideoPlayerState(current, { type: "ready" }));
      return;
    }

    if (status === "error") {
      setPlayerState((current) =>
        reduceVideoPlayerState(current, {
          type: "error",
          message: error?.message,
        }),
      );
    }
  });

  useEventListener(player, "playToEnd", () => {
    setPlayerState((current) => reduceVideoPlayerState(current, { type: "ended" }));
  });

  const overlay = shellSnapshot.overlay;
  const showOverlay = overlay.kind !== null;
  const overlayKind = overlay.kind === "end" ? "empty" : overlay.kind;

  return (
    <View
      testID={testID}
      accessibilityLabel={accessibilityLabel}
      style={[styles.container, style]}
    >
      <VideoView
        player={player}
        style={styles.video}
        nativeControls
        contentFit={contentFit}
        fullscreenOptions={{ enable: true }}
      />

      {showOverlay ? (
        <View pointerEvents="box-none" style={styles.overlay}>
          <StateView
            testID={testID ? `${testID}-overlay` : undefined}
            kind={overlayKind ?? "loading"}
            title={overlay.title}
            message={overlay.message}
            onRetry={overlay.kind === "error" ? onRetry : undefined}
          />
        </View>
      ) : null}

      <View pointerEvents="none" style={styles.metaRow}>
        <Text style={styles.metaLabel}>Original track</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    position: "relative",
    overflow: "hidden",
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: color.border,
    backgroundColor: color.ink900,
    minHeight: 220,
  },
  video: {
    width: "100%",
    aspectRatio: 16 / 9,
    backgroundColor: color.ink900,
  },
  overlay: {
    ...StyleSheet.absoluteFill,
    justifyContent: "center",
    backgroundColor: color.ink900,
  },
  metaRow: {
    position: "absolute",
    top: space.md,
    left: space.md,
    paddingHorizontal: space.sm,
    paddingVertical: space.xs,
    borderRadius: radius.pill,
    backgroundColor: color.primarySubtle,
  },
  metaLabel: {
    ...type.label,
    color: color.primaryPressed,
  },
});
