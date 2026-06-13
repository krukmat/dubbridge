import { cleanup, fireEvent, render } from "@testing-library/react-native";
import { StyleSheet, Text } from "react-native";
import { SafeAreaProvider } from "react-native-safe-area-context";

import { Badge, statusTone } from "../src/components/Badge";
import { Button } from "../src/components/Button";
import { Screen } from "../src/components/Screen";
import { StateView } from "../src/components/StateView";
import { color, space } from "../src/theme";

const METRICS = {
  frame: { x: 0, y: 0, width: 390, height: 844 },
  insets: { top: 47, bottom: 34, left: 0, right: 0 },
};

afterEach(cleanup);

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
