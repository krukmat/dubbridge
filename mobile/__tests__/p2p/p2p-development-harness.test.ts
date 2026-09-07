import { act, cleanup, render, waitFor } from "@testing-library/react-native";
import { createElement } from "react";
import { Platform } from "react-native";

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

import { P2PProvider } from "../../src/p2p/P2PProvider";
import { P2PDevelopmentHarness } from "../../src/p2p/development/P2PDevelopmentHarness";
import { P2PService } from "../../src/p2p/P2PService";
import type { RuntimeHandshake } from "../../src/p2p/runtime/protocol";

function createHandshake(): RuntimeHandshake {
  return {
    protocolVersion: 1,
    runtimeVersion: "test",
    capabilities: ["ping"],
  };
}

function createPendingHandshake() {
  let complete: (value: RuntimeHandshake) => void = () => undefined;
  const promise = new Promise<RuntimeHandshake>((resolve) => {
    complete = resolve;
  });

  return { complete, promise };
}

const platformOS = Object.getOwnPropertyDescriptor(Platform, "OS");

function withP2PProvider(enabled = true) {
  return createElement(
    P2PProvider,
    null,
    createElement(P2PDevelopmentHarness, { enabled }),
  );
}

describe("P2P development harness", () => {
  beforeEach(() => {
    Object.defineProperty(Platform, "OS", { configurable: true, value: "android" });
    jest.spyOn(console, "warn").mockImplementation(() => undefined);
  });

  afterEach(cleanup);

  afterEach(() => {
    if (platformOS) Object.defineProperty(Platform, "OS", platformOS);
    jest.restoreAllMocks();
  });

  it("stays inert when disabled or outside Android", async () => {
    const initialize = jest.spyOn(P2PService.prototype, "initialize");

    await render(withP2PProvider(false));
    expect(initialize).not.toHaveBeenCalled();

    Object.defineProperty(Platform, "OS", { configurable: true, value: "ios" });
    await render(withP2PProvider());
    expect(initialize).not.toHaveBeenCalled();
  });

  it("initializes, pings, and shuts down", async () => {
    const initialize = jest.spyOn(P2PService.prototype, "initialize").mockResolvedValue(createHandshake());
    const ping = jest.spyOn(P2PService.prototype, "ping").mockResolvedValue("pong");
    const shutdown = jest.spyOn(P2PService.prototype, "shutdown").mockResolvedValue(undefined);
    await act(async () => {
      await render(withP2PProvider());
    });

    await waitFor(() => {
      expect(initialize).toHaveBeenCalledTimes(1);
      expect(ping).toHaveBeenCalledTimes(1);
      expect(shutdown).toHaveBeenCalledTimes(1);
    });
  });

  it("stops startup after unmount without pinging", async () => {
    const pendingHandshake = createPendingHandshake();
    jest.spyOn(P2PService.prototype, "initialize").mockReturnValue(pendingHandshake.promise);
    const ping = jest.spyOn(P2PService.prototype, "ping").mockResolvedValue("pong");
    const shutdown = jest.spyOn(P2PService.prototype, "shutdown").mockResolvedValue(undefined);

    const view = await render(withP2PProvider());
    await waitFor(() => expect(P2PService.prototype.initialize).toHaveBeenCalledTimes(1));
    await act(async () => view.unmount());
    pendingHandshake.complete(createHandshake());
    await act(async () => Promise.resolve());

    expect(ping).not.toHaveBeenCalled();
    expect(shutdown).toHaveBeenCalledTimes(1);
  });

  it("reports startup failures without exposing runtime details", async () => {
    const error = jest.spyOn(console, "error").mockImplementation(() => undefined);
    jest
      .spyOn(P2PService.prototype, "initialize")
      .mockRejectedValue(new Error("remote secret-token-must-not-leak"));
    const shutdown = jest.spyOn(P2PService.prototype, "shutdown").mockResolvedValue(undefined);

    await render(withP2PProvider());

    await waitFor(() => {
      expect(error).toHaveBeenCalledWith("[P2P development harness] INITIALIZE_FAILED");
      expect(shutdown).toHaveBeenCalledTimes(1);
    });
    expect(error).not.toHaveBeenCalledWith(expect.stringContaining("secret-token-must-not-leak"));
  });

  it("reports shutdown failures without exposing runtime details", async () => {
    const error = jest.spyOn(console, "error").mockImplementation(() => undefined);
    jest.spyOn(P2PService.prototype, "initialize").mockResolvedValue(createHandshake());
    jest.spyOn(P2PService.prototype, "ping").mockResolvedValue("pong");
    jest
      .spyOn(P2PService.prototype, "shutdown")
      .mockRejectedValue(new Error("remote shutdown secret-token-must-not-leak"));

    await render(withP2PProvider());

    await waitFor(() => {
      expect(error).toHaveBeenCalledWith("[P2P development harness] SHUTDOWN_FAILED");
    });
    expect(error).not.toHaveBeenCalledWith(expect.stringContaining("secret-token-must-not-leak"));
  });

  it("reports cleanup failures without exposing runtime details", async () => {
    const pendingHandshake = createPendingHandshake();
    jest.spyOn(P2PService.prototype, "initialize").mockReturnValue(pendingHandshake.promise);
    jest
      .spyOn(P2PService.prototype, "shutdown")
      .mockRejectedValue(new Error("channel closed after late reply"));
    const error = jest.spyOn(console, "error").mockImplementation(() => undefined);

    const view = await render(withP2PProvider());
    await waitFor(() => expect(P2PService.prototype.initialize).toHaveBeenCalledTimes(1));
    await act(async () => view.unmount());
    pendingHandshake.complete(createHandshake());

    await waitFor(() => {
      expect(error).toHaveBeenCalledWith("[P2P development harness] SHUTDOWN_FAILED");
    });
    expect(error).not.toHaveBeenCalledWith(expect.stringContaining("channel closed"));
  });
});
