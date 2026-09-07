import { watchForDisconnect } from "../../src/p2p/runtime/transient-replication-discovery";
import { createReconnectBudget } from "../../src/p2p/runtime/reconnect-budget";

describe("transient-replication-discovery", () => {
  it("HP-B2.c-1: a disconnect within budget yields a retry decision", () => {
    const budget = createReconnectBudget(1);
    const listeners: Record<string, (err?: unknown) => void> = {};
    const stubSocket = {
      on: jest.fn(
        (event: "close" | "error", listener: (err?: unknown) => void) => {
          listeners[event] = listener;
        },
      ),
    };

    const onDisconnect = jest.fn();
    watchForDisconnect(stubSocket, budget, onDisconnect);

    expect(stubSocket.on).toHaveBeenCalledWith("close", expect.any(Function));
    expect(listeners["close"]).toBeDefined();

    listeners["close"]();

    expect(onDisconnect).toHaveBeenCalledWith(
      "retry",
      expect.objectContaining({ usedRetries: 1 }),
    );
  });

  it("EC-B2.c-1: a disconnect with exhausted budget yields a fail decision", () => {
    const budget = createReconnectBudget(0);
    const listeners: Record<string, (err?: unknown) => void> = {};
    const stubSocket = {
      on: jest.fn(
        (event: "close" | "error", listener: (err?: unknown) => void) => {
          listeners[event] = listener;
        },
      ),
    };

    const onDisconnect = jest.fn();
    watchForDisconnect(stubSocket, budget, onDisconnect);

    expect(stubSocket.on).toHaveBeenCalledWith("close", expect.any(Function));
    expect(listeners["close"]).toBeDefined();

    listeners["close"]();

    expect(onDisconnect).toHaveBeenCalledWith(
      "fail",
      expect.objectContaining({ usedRetries: 1 }),
    );
  });
});
