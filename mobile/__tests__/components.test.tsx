import { act, cleanup, fireEvent, render } from "@testing-library/react-native";
import { StyleSheet, Text } from "react-native";
import { SafeAreaProvider } from "react-native-safe-area-context";

const mockPlayerListeners: Record<string, ((payload?: unknown) => void) | undefined> = {};

jest.mock("expo", () => {
  const actual = jest.requireActual("expo");
  return {
    ...actual,
    useEventListener: jest.fn(
      (_player: unknown, eventName: string, listener: (payload?: unknown) => void) => {
        mockPlayerListeners[eventName] = listener;
      },
    ),
  };
});

jest.mock("expo-video", () => ({
  VideoView: ({ testID, ...props }: Record<string, unknown>) =>
    require("react").createElement(
      require("react-native").Text,
      { testID: testID ?? "mock-video-view" },
      JSON.stringify(props),
    ),
  useVideoPlayer: jest.fn(() => ({
    status: "idle",
    loop: false,
  })),
}));

import { Badge, statusTone } from "../src/components/Badge";
import { Button } from "../src/components/Button";
import { Card } from "../src/components/Card";
import { IconBadge } from "../src/components/IconBadge";
import { Screen } from "../src/components/Screen";
import { ScreenHeader } from "../src/components/ScreenHeader";
import { StateView } from "../src/components/StateView";
import { VideoPlayer } from "../src/components/VideoPlayer";
import { color, space } from "../src/theme";

const METRICS = {
  frame: { x: 0, y: 0, width: 390, height: 844 },
  insets: { top: 47, bottom: 34, left: 0, right: 0 },
};

afterEach(cleanup);
afterEach(() => {
  for (const key of Object.keys(mockPlayerListeners)) {
    delete mockPlayerListeners[key];
  }
});

describe("Button", () => {
  it("HP-1: primary renders its label, fires onPress, and exposes button role", async () => {
    const onPress = jest.fn();
    const view = await render(
      <Button testID="btn" label="Continue" onPress={onPress} />,
    );

    expect(view.getByText("Continue")).toBeTruthy();
    expect(view.getByTestId("btn").props.accessibilityRole).toBe("button");

    fireEvent.press(view.getByTestId("btn"));
    expect(onPress).toHaveBeenCalledTimes(1);
  });

  it("EC-1: disabled does not fire onPress and reports disabled a11y state", async () => {
    const onPress = jest.fn();
    const view = await render(
      <Button testID="btn" label="Continue" onPress={onPress} disabled />,
    );

    fireEvent.press(view.getByTestId("btn"));
    expect(onPress).not.toHaveBeenCalled();
    expect(view.getByTestId("btn").props.accessibilityState).toMatchObject({
      disabled: true,
    });
  });

  it("EC-1b: loading blocks press and marks busy", async () => {
    const onPress = jest.fn();
    const view = await render(
      <Button testID="btn" label="Continue" onPress={onPress} loading />,
    );

    fireEvent.press(view.getByTestId("btn"));
    expect(onPress).not.toHaveBeenCalled();
    expect(view.getByTestId("btn").props.accessibilityState).toMatchObject({
      busy: true,
    });
  });
});

describe("StateView", () => {
  it("HP-2: error with onRetry renders the message and a working retry control", async () => {
    const onRetry = jest.fn();
    const view = await render(
      <StateView
        testID="state"
        kind="error"
        title="Could not load"
        message="Network failed"
        onRetry={onRetry}
      />,
    );

    expect(view.getByText("Network failed")).toBeTruthy();
    fireEvent.press(view.getByTestId("state-retry"));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("EC-2: error without onRetry renders no retry control", async () => {
    const view = await render(
      <StateView testID="state" kind="error" message="Network failed" />,
    );

    expect(view.queryByTestId("state-retry")).toBeNull();
  });

  it("loading renders the provided message", async () => {
    const view = await render(
      <StateView kind="loading" title="Loading assets…" message="Fetching…" />,
    );
    expect(view.getByText("Loading assets…")).toBeTruthy();
    expect(view.getByText("Fetching…")).toBeTruthy();
  });

  it("EC-2b: inverse appearance switches text to dark-surface-safe foreground colors", async () => {
    const view = await render(
      <StateView
        kind="loading"
        title="Loading assets…"
        message="Fetching…"
        appearance="inverse"
      />,
    );

    expect(StyleSheet.flatten(view.getByText("Loading assets…").props.style).color).toBe(color.onPrimary);
    expect(StyleSheet.flatten(view.getByText("Fetching…").props.style).color).toBe(color.canvas);
  });
});

describe("Badge", () => {
  it("HP-3: status maps to semantic tone", () => {
    expect(statusTone("grant")).toBe("success");
    expect(statusTone("active")).toBe("success");
    expect(statusTone("finalized")).toBe("success");
    expect(statusTone("revoke")).toBe("danger");
    expect(statusTone("pending")).toBe("info");
  });

  it("renders the success tone background for an active badge", async () => {
    const view = await render(<Badge testID="b" label="Active" tone="success" />);
    const flat = StyleSheet.flatten(view.getByTestId("b").props.style);
    expect(flat.backgroundColor).toBe(color.successSubtle);
    expect(view.getByText("Active")).toBeTruthy();
  });

  it("EC-3: unknown status falls back to neutral and never throws", async () => {
    expect(statusTone("totally-unknown")).toBe("neutral");
    expect(statusTone(null)).toBe("neutral");
    const view = await render(
      // @ts-expect-error intentionally passing an unsupported tone value
      <Badge testID="b" label="???" tone="made-up" />,
    );
    const flat = StyleSheet.flatten(view.getByTestId("b").props.style);
    expect(flat.backgroundColor).toBe(color.sunken);
  });
});

describe("IconBadge", () => {
  it("renders a restrained monogram icon with the requested tone", async () => {
    const view = await render(<IconBadge testID="icon" symbol="RV" tone="info" />);
    const flat = StyleSheet.flatten(view.getByTestId("icon").props.style);
    expect(flat.backgroundColor).toBe(color.infoSubtle);
    expect(view.getByText("RV")).toBeTruthy();
  });
});

describe("Screen", () => {
  it("HP-4: applies safe-area top inset on top of base padding when a provider is present", async () => {
    const view = await render(
      <SafeAreaProvider initialMetrics={METRICS}>
        <Screen testID="screen">
          <Text>Hello</Text>
        </Screen>
      </SafeAreaProvider>,
    );

    expect(view.getByText("Hello")).toBeTruthy();
    const flat = StyleSheet.flatten(view.getByTestId("screen").props.style);
    expect(flat.paddingTop).toBe(space.xxl + METRICS.insets.top);
    expect(flat.backgroundColor).toBe(color.canvas);
  });

  it("degrades to zero insets (base padding only) without a provider", async () => {
    const view = await render(
      <Screen testID="screen">
        <Text>Hello</Text>
      </Screen>,
    );

    expect(view.getByText("Hello")).toBeTruthy();
    const flat = StyleSheet.flatten(view.getByTestId("screen").props.style);
    expect(flat.paddingTop).toBe(space.xxl);
  });
});

describe("VideoPlayer", () => {
  it("HP-5: renders the expo-video shell inside the tokenized player container", async () => {
    const view = await render(
      <VideoPlayer testID="player" source="https://example.com/manifest.m3u8" />,
    );

    expect(view.getByTestId("player")).toBeTruthy();
    expect(view.getByText("Original track")).toBeTruthy();
    expect(view.getByText("Loading video")).toBeTruthy();
    expect(StyleSheet.flatten(view.getByText("Loading video").props.style).color).toBe(color.onPrimary);
    expect(
      StyleSheet.flatten(view.getByText("Preparing the player for playback.").props.style).color,
    ).toBe(color.canvas);
  });

  it("HP-6: binds an error overlay through StateView without embedding reducer logic", async () => {
    const onRetry = jest.fn();
    const view = await render(
      <VideoPlayer
        testID="player"
        source="https://example.com/manifest.m3u8"
        onRetry={onRetry}
      />,
    );

    await act(async () => {
      mockPlayerListeners.statusChange?.({
        status: "error",
        error: { message: "Manifest request failed" },
      });
    });

    expect(view.getByText("Playback error")).toBeTruthy();
    expect(view.getByText("Manifest request failed")).toBeTruthy();
    fireEvent.press(view.getByTestId("player-overlay-retry"));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("HP-7: statusChange ready hides the overlay and loading shows it again", async () => {
    const view = await render(
      <VideoPlayer testID="player" source="https://example.com/manifest.m3u8" />,
    );

    await act(async () => {
      mockPlayerListeners.statusChange?.({ status: "readyToPlay" });
    });

    expect(view.queryByTestId("player-overlay")).toBeNull();

    await act(async () => {
      mockPlayerListeners.statusChange?.({ status: "loading" });
    });

    expect(view.getByText("Loading video")).toBeTruthy();
  });

  it("HP-8: playToEnd shows the end overlay after playback is ready", async () => {
    const view = await render(
      <VideoPlayer testID="player" source="https://example.com/manifest.m3u8" />,
    );

    await act(async () => {
      mockPlayerListeners.statusChange?.({ status: "readyToPlay" });
    });

    await act(async () => {
      mockPlayerListeners.playToEnd?.();
    });

    expect(view.getByText("Playback finished")).toBeTruthy();
  });

  it("EC-4: null source keeps the shell safe and shows a waiting overlay", async () => {
    const view = await render(<VideoPlayer testID="player" source={null} />);

    expect(view.getByTestId("player")).toBeTruthy();
    expect(view.getByText("Waiting for media")).toBeTruthy();
    expect(
      view.getByText("A playback source is required before the player can start."),
    ).toBeTruthy();
  });
});

describe("Card", () => {
  it("renders a leading adornment alongside title content", async () => {
    const view = await render(
      <Card
        title="Review inbox"
        subtitle="2 pending"
        leadingAdornment={<IconBadge testID="card-icon" symbol="RV" tone="info" />}
      />,
    );

    expect(view.getByTestId("card-icon")).toBeTruthy();
    expect(view.getByText("Review inbox")).toBeTruthy();
    expect(view.getByText("2 pending")).toBeTruthy();
  });
});

describe("Card", () => {
  it("HP-1: title-mode renders title and subtitle without mediaTone", async () => {
    const view = await render(
      <Card title="Track Alpha" subtitle="Ready" />,
    );
    expect(view.getByText("Track Alpha")).toBeTruthy();
    expect(view.getByText("Ready")).toBeTruthy();
  });

  it("HP-2: mediaTone renders without crash alongside title", async () => {
    const view = await render(
      <Card testID="card" title="Track Alpha" mediaTone="success" />,
    );
    expect(view.getByText("Track Alpha")).toBeTruthy();
    expect(view.getByTestId("card")).toBeTruthy();
  });

  it("HP-3: children-mode without mediaTone preserves original children layout", async () => {
    const view = await render(
      <Card testID="card">
        <Text>Child content</Text>
      </Card>,
    );
    expect(view.getByText("Child content")).toBeTruthy();
  });

  it("HP-4: children-mode with mediaTone renders tile alongside children without crash", async () => {
    const view = await render(
      <Card testID="card" mediaTone="info">
        <Text>Track Beta</Text>
      </Card>,
    );
    expect(view.getByText("Track Beta")).toBeTruthy();
    expect(view.getByTestId("card")).toBeTruthy();
  });
});

describe("ScreenHeader", () => {
  it("HP-1: full variant renders kicker, title, and optional copy", async () => {
    const view = await render(
      <ScreenHeader kicker="Assets" title="Asset list" copy="Browse your uploads." />,
    );
    expect(view.getByText("Assets")).toBeTruthy();
    expect(view.getByText("Asset list")).toBeTruthy();
    expect(view.getByText("Browse your uploads.")).toBeTruthy();
  });

  it("HP-2: full variant without copy omits the copy text", async () => {
    const view = await render(<ScreenHeader title="Asset list" />);
    expect(view.getByText("Asset list")).toBeTruthy();
    expect(view.queryByText("Browse your uploads.")).toBeNull();
  });

  it("HP-3: compact variant with kicker renders only the kicker", async () => {
    const view = await render(<ScreenHeader kicker="Asset" title="Asset detail" compact />);
    expect(view.getByText("Asset")).toBeTruthy();
    expect(view.queryByText("Asset detail")).toBeNull();
  });

  it("EC-1: compact variant without kicker renders nothing", async () => {
    const view = await render(<ScreenHeader title="Asset detail" compact />);
    expect(view.queryByText("Asset detail")).toBeNull();
  });
});
