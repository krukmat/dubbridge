import { cleanup, render } from "@testing-library/react-native";
import { Text } from "react-native";
import { SafeAreaProvider } from "react-native-safe-area-context";

import { ActionBar } from "../src/components/ActionBar";

const METRICS = {
  frame: { x: 0, y: 0, width: 390, height: 844 },
  insets: { top: 47, bottom: 34, left: 0, right: 0 },
};

afterEach(cleanup);

describe("ActionBar", () => {
  it("HP-1: renders children inside the bar", async () => {
    const view = await render(
      <SafeAreaProvider initialMetrics={METRICS}>
        <ActionBar testID="action-bar">
          <Text>Approve</Text>
        </ActionBar>
      </SafeAreaProvider>,
    );

    expect(view.getByTestId("action-bar")).toBeTruthy();
    expect(view.getByText("Approve")).toBeTruthy();
  });

  it("HP-2: renders multiple children", async () => {
    const view = await render(
      <SafeAreaProvider initialMetrics={METRICS}>
        <ActionBar>
          <Text>Approve</Text>
          <Text>Reject</Text>
        </ActionBar>
      </SafeAreaProvider>,
    );

    expect(view.getByText("Approve")).toBeTruthy();
    expect(view.getByText("Reject")).toBeTruthy();
  });

  it("EC-1: renders without SafeAreaProvider (degrades to zero insets)", async () => {
    const view = await render(
      <ActionBar testID="action-bar">
        <Text>Continue</Text>
      </ActionBar>,
    );

    expect(view.getByTestId("action-bar")).toBeTruthy();
    expect(view.getByText("Continue")).toBeTruthy();
  });
});
