import { act, cleanup, render, waitFor } from "@testing-library/react-native";
import type { ReactNode } from "react";
import { Platform, Text } from "react-native";

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

import {
  P2PProvider,
  useP2PRuntimeSnapshot,
  useP2PService,
} from "../../src/p2p/P2PProvider";
import { AndroidBareRuntimeProbe } from "../../src/p2p/AndroidBareRuntimeProbe";
import { P2PService } from "../../src/p2p/P2PService";
import type { RuntimeHandshake } from "../../src/p2p/runtime/protocol";

let capturedService: P2PService | undefined;
let renderCount = 0;
const handshake: RuntimeHandshake = {
  protocolVersion: 1,
  runtimeVersion: "test",
  capabilities: ["ping"],
};
const platformOS = Object.getOwnPropertyDescriptor(Platform, "OS");

function Consumer() {
  capturedService = useP2PService();
  const snapshot = useP2PRuntimeSnapshot();
  renderCount += 1;
  return <Text testID="p2p-state">{snapshot.runtimeState}</Text>;
}

function withP2PProvider(children: ReactNode) {
  return <P2PProvider>{children}</P2PProvider>;
}

describe("P2PProvider", () => {
  beforeEach(() => {
    capturedService = undefined;
    renderCount = 0;
  });

  afterEach(cleanup);

  afterEach(() => {
    if (platformOS) Object.defineProperty(Platform, "OS", platformOS);
    jest.restoreAllMocks();
  });

  it("HP-F2 owns one stable inert service across rerenders", async () => {
    const view = await render(withP2PProvider(<Consumer />));
    const firstService = capturedService;

    view.rerender(withP2PProvider(<Consumer />));

    expect(capturedService).toBe(firstService);
    expect(firstService?.getSnapshot()).toEqual({ runtimeState: "stopped", lastError: null });
    expect(view.getByTestId("p2p-state").props.children).toBe("stopped");
  });

  it("EC-F2 updates only a status subscriber after an explicit service operation", async () => {
    await render(withP2PProvider(<Consumer />));
    const service = capturedService;
    expect(service).toBeDefined();

    await act(async () => {
      await expect(service?.ping()).rejects.toMatchObject({ code: "INVALID_STATE" });
    });

    expect(service?.getSnapshot().lastError).toBe("Cannot ping while runtime is stopped");
    expect(renderCount).toBeGreaterThan(1);
  });

  it("HP-F2 runs the Android P0 probe through the stable service and shuts it down", async () => {
    Object.defineProperty(Platform, "OS", { configurable: true, value: "android" });
    const initialize = jest.spyOn(P2PService.prototype, "initialize").mockResolvedValue(handshake);
    const ping = jest.spyOn(P2PService.prototype, "ping").mockResolvedValue("pong");
    const shutdown = jest.spyOn(P2PService.prototype, "shutdown").mockResolvedValue(undefined);
    jest.spyOn(console, "warn").mockImplementation(() => undefined);

    await act(async () => {
      await render(withP2PProvider(<AndroidBareRuntimeProbe enabled />));
    });

    await waitFor(() => {
      expect(initialize).toHaveBeenCalledTimes(1);
      expect(ping).toHaveBeenCalledTimes(1);
      expect(shutdown).toHaveBeenCalledTimes(1);
    });
  });

  it("EC-F2 stops the probe after unmount while initialization is pending", async () => {
    Object.defineProperty(Platform, "OS", { configurable: true, value: "android" });
    let resolveInitialization: (value: RuntimeHandshake) => void = () => undefined;
    const pendingInitialization = new Promise<RuntimeHandshake>((resolve) => {
      resolveInitialization = resolve;
    });
    jest.spyOn(P2PService.prototype, "initialize").mockReturnValue(pendingInitialization);
    const ping = jest.spyOn(P2PService.prototype, "ping").mockResolvedValue("pong");
    const shutdown = jest.spyOn(P2PService.prototype, "shutdown").mockResolvedValue(undefined);

    const view = await render(withP2PProvider(<AndroidBareRuntimeProbe enabled />));
    await waitFor(() => expect(P2PService.prototype.initialize).toHaveBeenCalledTimes(1));
    await act(async () => {
      view.unmount();
    });
    resolveInitialization(handshake);
    await act(async () => {
      await Promise.resolve();
    });

    expect(ping).not.toHaveBeenCalled();
    expect(shutdown).toHaveBeenCalledTimes(1);
  });

  it("EC-F2 reports probe initialization and shutdown failures without leaking work", async () => {
    Object.defineProperty(Platform, "OS", { configurable: true, value: "android" });
    const error = jest.spyOn(console, "error").mockImplementation(() => undefined);
    jest.spyOn(P2PService.prototype, "initialize").mockRejectedValue(new Error("initialization failed"));
    jest.spyOn(P2PService.prototype, "shutdown").mockRejectedValue(new Error("shutdown failed"));

    await render(withP2PProvider(<AndroidBareRuntimeProbe enabled />));

    await waitFor(() => {
      expect(error).toHaveBeenCalledWith("[Bare runtime probe] initialization failed");
      expect(error).toHaveBeenCalledWith("[Bare runtime probe] shutdown failed");
    });
  });

  it("EC-F2 reports a rejected cleanup shutdown", async () => {
    Object.defineProperty(Platform, "OS", { configurable: true, value: "android" });
    let resolveInitialization: (value: RuntimeHandshake) => void = () => undefined;
    const pendingInitialization = new Promise<RuntimeHandshake>((resolve) => {
      resolveInitialization = resolve;
    });
    jest.spyOn(P2PService.prototype, "initialize").mockReturnValue(pendingInitialization);
    jest.spyOn(P2PService.prototype, "shutdown").mockRejectedValue(new Error("cleanup failed"));
    const error = jest.spyOn(console, "error").mockImplementation(() => undefined);

    const view = await render(withP2PProvider(<AndroidBareRuntimeProbe enabled />));
    await waitFor(() => expect(P2PService.prototype.initialize).toHaveBeenCalledTimes(1));
    await act(async () => {
      view.unmount();
    });
    resolveInitialization(handshake);

    await waitFor(() => expect(error).toHaveBeenCalledWith("[Bare runtime probe] cleanup failed"));
  });
});
