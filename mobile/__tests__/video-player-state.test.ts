import {
  createVideoPlayerShellSnapshot,
  createVideoPlayerState,
  reduceVideoPlayerState,
} from "../src/components/video-player-state";

describe("video-player-state", () => {
  it("HP-1: source_changed moves idle to loading when a source is provided", () => {
    const initial = createVideoPlayerState(null);

    const next = reduceVideoPlayerState(initial, {
      type: "source_changed",
      source: "https://example.com/manifest.m3u8",
    });

    expect(initial).toEqual({ kind: "idle" });
    expect(next).toEqual({ kind: "loading" });
  });

  it("HP-2: ready moves loading to playing and hides the overlay", () => {
    const initial = createVideoPlayerState("https://example.com/manifest.m3u8");

    const next = reduceVideoPlayerState(initial, { type: "ready" });
    const snapshot = createVideoPlayerShellSnapshot(
      "https://example.com/manifest.m3u8",
      next,
    );

    expect(next).toEqual({ kind: "playing" });
    expect(snapshot.overlay).toEqual({ kind: null });
  });

  it("HP-2b: loading can return playing state to a loading overlay during rebuffer", () => {
    const next = reduceVideoPlayerState({ kind: "playing" }, { type: "loading" });
    const snapshot = createVideoPlayerShellSnapshot(
      "https://example.com/manifest.m3u8",
      next,
    );

    expect(next).toEqual({ kind: "loading" });
    expect(snapshot.overlay).toMatchObject({
      kind: "loading",
      title: "Loading video",
    });
  });

  it("HP-2c: ended moves playing to end", () => {
    const next = reduceVideoPlayerState({ kind: "playing" }, { type: "ended" });

    expect(next).toEqual({ kind: "end" });
  });

  it("EC-1: null source stays idle and renders the waiting overlay", () => {
    const initial = createVideoPlayerState(null);
    const next = reduceVideoPlayerState(initial, {
      type: "source_changed",
      source: null,
    });
    const snapshot = createVideoPlayerShellSnapshot(null, next);

    expect(next).toEqual({ kind: "idle" });
    expect(snapshot.overlay).toMatchObject({
      kind: "loading",
      title: "Waiting for media",
    });
  });

  it("EC-2: error event preserves the provided message", () => {
    const initial = createVideoPlayerState("https://example.com/manifest.m3u8");

    const next = reduceVideoPlayerState(initial, {
      type: "error",
      message: "Manifest request failed",
    });
    const snapshot = createVideoPlayerShellSnapshot(
      "https://example.com/manifest.m3u8",
      next,
    );

    expect(next).toEqual({ kind: "error", message: "Manifest request failed" });
    expect(snapshot.overlay).toMatchObject({
      kind: "error",
      title: "Playback error",
      message: "Manifest request failed",
    });
  });
});
