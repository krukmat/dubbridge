import type { VideoSource } from "expo-video";

export type VideoPlayerOverlayKind = "loading" | "error" | "end" | null;

export type VideoPlayerOverlayState = {
  kind: VideoPlayerOverlayKind;
  title?: string;
  message?: string;
};

export type VideoPlayerShellSnapshot = {
  hasSource: boolean;
  overlay: VideoPlayerOverlayState;
};

export type VideoPlayerState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "playing" }
  | { kind: "error"; message: string }
  | { kind: "end" };

export type VideoPlayerEvent =
  | { type: "source_changed"; source: VideoSource }
  | { type: "loading" }
  | { type: "ready" }
  | { type: "error"; message?: string }
  | { type: "ended" };

const WAITING_OVERLAY: VideoPlayerOverlayState = {
  kind: "loading",
  title: "Waiting for media",
  message: "A playback source is required before the player can start.",
};

const LOADING_OVERLAY: VideoPlayerOverlayState = {
  kind: "loading",
  title: "Loading video",
  message: "Preparing the player for playback.",
};

export const VIDEO_PLAYER_END_OVERLAY: VideoPlayerOverlayState = {
  kind: "end",
  title: "Playback finished",
  message: "Replay controls will be wired in the next task.",
};

export const VIDEO_PLAYER_ERROR_OVERLAY: VideoPlayerOverlayState = {
  kind: "error",
  title: "Playback error",
  message: "The video could not be played right now.",
};

export function createVideoPlayerState(source: VideoSource): VideoPlayerState {
  return source == null ? { kind: "idle" } : { kind: "loading" };
}

export function reduceVideoPlayerState(
  state: VideoPlayerState,
  event: VideoPlayerEvent,
): VideoPlayerState {
  switch (event.type) {
    case "source_changed":
      return createVideoPlayerState(event.source);
    case "loading":
      if (state.kind === "idle") {
        return state;
      }
      return { kind: "loading" };
    case "ready":
      if (state.kind === "idle") {
        return state;
      }
      return { kind: "playing" };
    case "error":
      if (state.kind === "idle") {
        return state;
      }
      return {
        kind: "error",
        message: event.message?.trim() || VIDEO_PLAYER_ERROR_OVERLAY.message || "",
      };
    case "ended":
      if (state.kind !== "playing") {
        return state;
      }
      return { kind: "end" };
  }
}

export function createVideoPlayerShellSnapshot(
  source: VideoSource,
  state: VideoPlayerState,
): VideoPlayerShellSnapshot {
  return {
    hasSource: source != null,
    overlay: overlayForState(source, state),
  };
}

function overlayForState(
  source: VideoSource,
  state: VideoPlayerState,
): VideoPlayerOverlayState {
  switch (state.kind) {
    case "idle":
      return source == null ? WAITING_OVERLAY : LOADING_OVERLAY;
    case "loading":
      return LOADING_OVERLAY;
    case "playing":
      return { kind: null };
    case "error":
      return {
        ...VIDEO_PLAYER_ERROR_OVERLAY,
        message: state.message,
      };
    case "end":
      return VIDEO_PLAYER_END_OVERLAY;
  }
}
