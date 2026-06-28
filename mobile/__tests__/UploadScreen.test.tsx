import { cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import type { AuthContextValue } from "../src/auth/AuthProvider";
import { UploadScreen } from "../src/screens/UploadScreen";

(
  globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }
).IS_REACT_ACT_ENVIRONMENT = true;

jest.mock("../src/auth/AuthProvider", () => ({
  useAuth: () => mockAuthValue,
}));

jest.mock("../src/api/client", () => ({
  createGatewayClient: jest.fn(),
}));

// E2E mode disabled for all tests in this file.
const originalE2EEnabled = process.env.EXPO_PUBLIC_E2E_ENABLED;

let mockAuthValue: AuthContextValue;

beforeEach(() => {
  jest.clearAllMocks();
  delete process.env.EXPO_PUBLIC_E2E_ENABLED;

  mockAuthValue = {
    sessionRef: "session-abc",
    status: "authed",
    loginError: null,
    login: jest.fn().mockResolvedValue(undefined),
    logout: jest.fn().mockResolvedValue(undefined),
    onSessionRotation: jest.fn().mockResolvedValue(undefined),
  };
});

afterEach(() => {
  if (originalE2EEnabled === undefined) {
    delete process.env.EXPO_PUBLIC_E2E_ENABLED;
  } else {
    process.env.EXPO_PUBLIC_E2E_ENABLED = originalE2EEnabled;
  }
  cleanup();
});

// SC-FORM-1: Incomplete rights form shows a visible validation message
describe("SC-FORM-1: incomplete rights form shows visible validation", () => {
  it("HP-1: tapping Continue with all fields empty shows per-field error messages", async () => {
    const view = await render(
      <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
    );

    // Start on rights_form (E2E disabled).
    await waitFor(() => {
      expect(view.getByTestId("upload-field-owner")).toBeTruthy();
    });

    fireEvent.press(view.getByTestId("upload-submit-rights"));

    await waitFor(() => {
      expect(view.getByTestId("upload-error-owner")).toBeTruthy();
      expect(view.getByTestId("upload-error-license-type")).toBeTruthy();
      expect(view.getByTestId("upload-error-source-type")).toBeTruthy();
      expect(view.getByTestId("upload-error-proof-reference")).toBeTruthy();
    });
  });

  it("HP-2: form does not advance to file step when validation fails", async () => {
    const view = await render(
      <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
    );

    await waitFor(() => {
      expect(view.getByTestId("upload-submit-rights")).toBeTruthy();
    });

    fireEvent.press(view.getByTestId("upload-submit-rights"));

    await waitFor(() => {
      // Still on rights form — pick-file button should NOT be present.
      expect(view.queryByTestId("upload-pick-file")).toBeNull();
    });
  });

  it("EC-1: error for a field clears after the user interacts with it", async () => {
    const view = await render(
      <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
    );

    await waitFor(() => {
      expect(view.getByTestId("upload-field-owner")).toBeTruthy();
    });

    fireEvent.press(view.getByTestId("upload-submit-rights"));

    await waitFor(() => {
      expect(view.getByTestId("upload-error-owner")).toBeTruthy();
    });

    // Typing in the owner field clears the owner error.
    fireEvent.changeText(view.getByTestId("upload-field-owner"), "DubBridge Studios");

    await waitFor(() => {
      expect(view.queryByTestId("upload-error-owner")).toBeNull();
    });
  });
});

// SC-FORM-2: 3-step progress indicator visible and reflects current step
describe("SC-FORM-2: step-progress indicator", () => {
  it("HP-1: step progress indicator is visible on the rights form step", async () => {
    const view = await render(
      <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
    );

    await waitFor(() => {
      expect(view.getByTestId("upload-step-progress")).toBeTruthy();
    });

    // Step labels are rendered.
    expect(view.getByText("Rights")).toBeTruthy();
    expect(view.getByText("File")).toBeTruthy();
    expect(view.getByText("Finalize")).toBeTruthy();
  });

  it("HP-2: completing the rights step advances the progress indicator to File", async () => {
    const view = await render(
      <UploadScreen gatewayBaseUrl="http://127.0.0.1:4000" onSuccess={jest.fn()} />,
    );

    await waitFor(() => {
      expect(view.getByTestId("upload-field-owner")).toBeTruthy();
    });

    // Fill owner.
    fireEvent.changeText(view.getByTestId("upload-field-owner"), "DubBridge Studios");
    // Select license_type.
    fireEvent.press(view.getByTestId("upload-field-license-type-option-exclusive"));
    // Select source_type.
    fireEvent.press(view.getByTestId("upload-field-source-type-option-original"));
    // Fill proof_reference.
    fireEvent.changeText(view.getByTestId("upload-field-proof-reference"), "contract-456");

    fireEvent.press(view.getByTestId("upload-submit-rights"));

    await waitFor(() => {
      // Advanced to file_pending step — pick-file button is visible.
      expect(view.getByTestId("upload-pick-file")).toBeTruthy();
      // Progress indicator still present.
      expect(view.getByTestId("upload-step-progress")).toBeTruthy();
    });
  });
});
