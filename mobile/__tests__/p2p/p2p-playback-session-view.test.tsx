import { act, cleanup, render } from "@testing-library/react-native";

const mockVideoPlayer = jest.fn((_props: Record<string, unknown>) =>
  require("react").createElement(require("react-native").View, { testID: "mock-video-player" }),
);

jest.mock("../../src/components/VideoPlayer", () => ({
  VideoPlayer: (props: Record<string, unknown>) => mockVideoPlayer(props),
}));

import type { P2PPlaybackSession } from "../../src/p2p/playback/P2PPlaybackController";
import { P2PPlaybackSessionView } from "../../src/p2p/playback/P2PPlaybackSessionView";

const session: P2PPlaybackSession = {
  playbackUrl: `http://127.0.0.1:43210/${"c".repeat(32)}/index.m3u8`,
  accountScope: "viewer-1",
  publicationId: "publication-1",
  lineageId: "lineage-1",
};

afterEach(cleanup);
afterEach(() => mockVideoPlayer.mockClear());

function latestPlayerProps() {
  const call = mockVideoPlayer.mock.calls[mockVideoPlayer.mock.calls.length - 1];
  if (!call) throw new Error("VideoPlayer was not rendered");
  return call[0] as {
    source: string;
    onPlaybackError: () => void;
    onRetry?: () => void;
  };
}

describe("P5 playback session view", () => {
  it("connects the scoped loopback URL to the existing VideoPlayer", async () => {
    const controller = { stop: jest.fn(async () => undefined) };
    await render(
      <P2PPlaybackSessionView
        testID="p2p-player"
        session={session}
        controller={controller}
      />,
    );

    expect(latestPlayerProps().source).toBe(session.playbackUrl);
  });

  it("releases the loopback session once when player error and unmount race", async () => {
    const controller = { stop: jest.fn(async () => undefined) };
    const view = await render(
      <P2PPlaybackSessionView session={session} controller={controller} />,
    );
    const player = latestPlayerProps();

    act(() => player.onPlaybackError());
    expect(controller.stop).toHaveBeenCalledTimes(1);

    await view.unmount();
    expect(controller.stop).toHaveBeenCalledTimes(1);
  });

  it("waits for teardown before retrying and reuses the same stop operation", async () => {
    let resolveStop: (() => void) | undefined;
    const stop = jest.fn(
      () => new Promise<void>((resolve) => {
        resolveStop = resolve;
      }),
    );
    const onRetry = jest.fn();
    await render(
      <P2PPlaybackSessionView
        session={session}
        controller={{ stop }}
        onRetry={onRetry}
      />,
    );
    const player = latestPlayerProps();

    act(() => {
      player.onPlaybackError();
      player.onRetry?.();
    });

    expect(stop).toHaveBeenCalledTimes(1);
    expect(onRetry).not.toHaveBeenCalled();

    await act(async () => {
      resolveStop?.();
      await Promise.resolve();
    });

    expect(onRetry).toHaveBeenCalledTimes(1);
    expect(stop).toHaveBeenCalledTimes(1);
  });

  it("keeps retry fail-closed when teardown rejects", async () => {
    const stop = jest.fn(async () => {
      throw new Error("stop failed");
    });
    const onRetry = jest.fn();
    await render(
      <P2PPlaybackSessionView
        session={session}
        controller={{ stop }}
        onRetry={onRetry}
      />,
    );
    const player = latestPlayerProps();

    act(() => player.onRetry?.());
    await act(async () => Promise.resolve());

    expect(stop).toHaveBeenCalledTimes(1);
    expect(onRetry).not.toHaveBeenCalled();
  });
});
