import { act, cleanup, render } from "@testing-library/react-native";

const mockPlayerListeners: Record<string, ((payload?: any) => void) | undefined> = {};

jest.mock("expo", () => {
  const actual = jest.requireActual("expo");
  return {
    ...actual,
    useEventListener: jest.fn(
      (_player: unknown, eventName: string, listener: (payload?: any) => void) => {
        mockPlayerListeners[eventName] = listener;
      },
    ),
  };
});

jest.mock("expo-video", () => ({
  VideoView: () => null,
  useVideoPlayer: jest.fn(() => ({ status: "idle", loop: false })),
}));

import { VideoPlayer } from "../src/components/VideoPlayer";

afterEach(cleanup);
afterEach(() => {
  for (const key of Object.keys(mockPlayerListeners)) delete mockPlayerListeners[key];
});

describe("VideoPlayer lifecycle seam", () => {
  it("forwards native playback errors without changing legacy rendering behavior", async () => {
    const onPlaybackError = jest.fn();
    await render(
      <VideoPlayer
        testID="player"
        source="http://127.0.0.1:43210/session/index.m3u8"
        onPlaybackError={onPlaybackError}
      />,
    );

    await act(async () => {
      mockPlayerListeners.statusChange?.({
        status: "error",
        error: { message: "Loopback request failed" },
      });
    });

    expect(onPlaybackError).toHaveBeenCalledTimes(1);
    expect(onPlaybackError).toHaveBeenCalledWith("Loopback request failed");
  });

  it("does not report loading or ready transitions as playback errors", async () => {
    const onPlaybackError = jest.fn();
    await render(
      <VideoPlayer
        source="https://example.com/review/index.m3u8"
        onPlaybackError={onPlaybackError}
      />,
    );

    await act(async () => {
      mockPlayerListeners.statusChange?.({ status: "loading" });
      mockPlayerListeners.statusChange?.({ status: "readyToPlay" });
    });

    expect(onPlaybackError).not.toHaveBeenCalled();
  });
});
