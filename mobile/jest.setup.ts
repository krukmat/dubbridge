import "react-native-gesture-handler/jestSetup";

jest.mock("expo", () => {
  const actual = jest.requireActual("expo");
  return {
    ...actual,
    useEventListener: jest.fn(),
  };
});

jest.mock("expo-video", () => ({
  VideoView: ({ testID, ...props }: Record<string, unknown>) => {
    const React = require("react");
    const { Text } = require("react-native");
    return React.createElement(
      Text,
      { testID: testID ?? "mock-video-view" },
      JSON.stringify(props),
    );
  },
  useVideoPlayer: jest.fn((_source: unknown, configure?: (player: { loop: boolean }) => void) => {
    const player = { status: "idle", loop: false };
    configure?.(player);
    return player;
  }),
}));
