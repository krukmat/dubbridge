import { cleanup, fireEvent, render } from "@testing-library/react-native";

import { Select, SelectField } from "../src/components/Select";

const OPTIONS = [
  { label: "Exclusive", value: "exclusive" },
  { label: "Non-exclusive", value: "non_exclusive" },
  { label: "Creative Commons", value: "creative_commons" },
];

afterEach(cleanup);

describe("Select", () => {
  it("HP-1: renders all options", async () => {
    const view = await render(
      <Select options={OPTIONS} value="" onChange={() => {}} testID="sel" />,
    );
    expect(view.getByText("Exclusive")).toBeTruthy();
    expect(view.getByText("Non-exclusive")).toBeTruthy();
    expect(view.getByText("Creative Commons")).toBeTruthy();
  });

  it("HP-2: selected option is marked accessibilityState selected=true", async () => {
    const view = await render(
      <Select options={OPTIONS} value="exclusive" onChange={() => {}} testID="sel" />,
    );
    const selectedPill = view.getByTestId("sel-option-exclusive");
    expect(selectedPill.props.accessibilityState?.selected).toBe(true);

    const unselectedPill = view.getByTestId("sel-option-non_exclusive");
    expect(unselectedPill.props.accessibilityState?.selected).toBe(false);
  });

  it("HP-3: pressing an option calls onChange with the correct value", async () => {
    const onChange = jest.fn();
    const view = await render(
      <Select options={OPTIONS} value="" onChange={onChange} testID="sel" />,
    );
    fireEvent.press(view.getByTestId("sel-option-non_exclusive"));
    expect(onChange).toHaveBeenCalledWith("non_exclusive");
  });

  it("EC-1: renders without crashing when options list is empty", async () => {
    const view = await render(
      <Select options={[]} value="" onChange={() => {}} testID="sel-empty" />,
    );
    expect(view.getByTestId("sel-empty")).toBeTruthy();
  });
});

describe("SelectField", () => {
  it("HP-1: renders label and options", async () => {
    const view = await render(
      <SelectField
        label="License type"
        options={OPTIONS}
        value=""
        onChange={() => {}}
        testID="sf"
      />,
    );
    expect(view.getByText("License type")).toBeTruthy();
    expect(view.getByText("Exclusive")).toBeTruthy();
  });

  it("HP-2: shows error text when error prop is provided", async () => {
    const view = await render(
      <SelectField
        label="License type"
        options={OPTIONS}
        value=""
        onChange={() => {}}
        testID="sf"
        error="Select a license type"
        errorTestID="sf-error"
      />,
    );
    expect(view.getByTestId("sf-error")).toBeTruthy();
    expect(view.getByText("Select a license type")).toBeTruthy();
  });

  it("EC-1: does not render error element when no error prop", async () => {
    const view = await render(
      <SelectField
        label="License type"
        options={OPTIONS}
        value=""
        onChange={() => {}}
        testID="sf"
        errorTestID="sf-error"
      />,
    );
    expect(view.queryByTestId("sf-error")).toBeNull();
  });
});
