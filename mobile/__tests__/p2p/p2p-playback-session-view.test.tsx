import { cleanup, render } from "@testing-library/react-native";

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

describe("P5 playback session view", () => {
  it("connects the scoped loopback URL and lifecycle callbacks to the existing VideoPlayer", async () => {
    const controller = { stop: jest.fn(async () => undefined) };
    const onRetry = jest.fn();
    await render(
      <P2PPlaybackSessionView
        testID="p2p-player"
        session={session}
        controller={controller}
        onRetry={onRetry}
      />,
    );

    const call = mockVideoPlayer.mock.calls[mockVideoPlayer.mock.calls.length - 1];
    expect(call).toBeDefined();
    expect(call?.[0]).toEqual(expect.objectContaining({
      testID: "p2p-player",
      source: session.playbackUrl,
      onPlaybackError: expect.any(Function),
      onRetry: expect.any(Function),
    }));
  });
});
