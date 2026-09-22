import { cleanup, render, waitFor } from "@testing-library/react-native";
import { Text } from "react-native";

import { useAuth, type AuthContextValue } from "../src/auth/AuthProvider";
import { P2PProvider } from "../src/p2p/P2PProvider";
import { P2PService } from "../src/p2p/P2PService";
import { P2PSyncController } from "../src/p2p/sync/P2PSyncController";

jest.mock("../src/auth/AuthProvider", () => ({
  useAuth: jest.fn(),
}));

jest.mock("../src/p2p/P2PService", () => ({
  P2PService: jest.fn(),
}));

jest.mock("../src/p2p/sync/P2PSyncController", () => ({
  P2PSyncController: jest.fn(),
}));

const mockUseAuth = useAuth as jest.MockedFunction<typeof useAuth>;
const mockP2PService = P2PService as jest.MockedClass<typeof P2PService>;
const mockP2PSyncController = P2PSyncController as jest.MockedClass<typeof P2PSyncController>;
const clearAccount = jest.fn(async () => undefined);

function authValue(userId: string | null): AuthContextValue {
  return {
    sessionRef: userId === null ? null : `token-${userId}`,
    userId,
    status: userId === null ? "unauthed" : "authed",
    loginError: null,
    login: jest.fn(async () => undefined),
    logout: jest.fn(async () => undefined),
    onSessionRotation: jest.fn(async () => undefined),
  };
}

describe("P2PProvider account lifecycle", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    clearAccount.mockClear();
    mockUseAuth.mockReturnValue(authValue("viewer-a"));
    mockP2PService.mockImplementation(() => ({} as P2PService));
    mockP2PSyncController.mockImplementation(
      () =>
        ({
          clearAccount,
        }) as unknown as P2PSyncController,
    );
  });

  afterEach(() => cleanup());

  it("clears the previous P2P account when authenticated identity changes", async () => {
    const view = render(
      <P2PProvider>
        <Text>child</Text>
      </P2PProvider>,
    );

    expect(clearAccount).not.toHaveBeenCalled();

    mockUseAuth.mockReturnValue(authValue("viewer-b"));
    view.rerender(
      <P2PProvider>
        <Text>child</Text>
      </P2PProvider>,
    );

    await waitFor(() => {
      expect(clearAccount).toHaveBeenCalledTimes(1);
    });
    expect(clearAccount).toHaveBeenCalledWith("viewer-a");

    view.rerender(
      <P2PProvider>
        <Text>child</Text>
      </P2PProvider>,
    );

    await waitFor(() => {
      expect(clearAccount).toHaveBeenCalledTimes(1);
    });
  });

  it("clears the signed-out account when auth transitions to unauthenticated", async () => {
    const view = render(
      <P2PProvider>
        <Text>child</Text>
      </P2PProvider>,
    );

    mockUseAuth.mockReturnValue(authValue(null));
    view.rerender(
      <P2PProvider>
        <Text>child</Text>
      </P2PProvider>,
    );

    await waitFor(() => {
      expect(clearAccount).toHaveBeenCalledWith("viewer-a");
    });
  });
});
