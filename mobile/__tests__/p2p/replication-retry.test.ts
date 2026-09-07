import { retryConnectAndReplicate } from "../../src/p2p/runtime/replication-retry";
import { RuntimeProtocolError } from "../../src/p2p/runtime/protocol";

describe("retryConnectAndReplicate", () => {
  it("HP-B2.c-2: a retry successfully re-establishes replication using the existing function", async () => {
    const fakeSocket = { pipe: jest.fn().mockReturnValue({ pipe: jest.fn() }) };
    const fakePeerInfo = { id: "peer-1" };
    const destroyMock = jest.fn();
    const drive = {
      replicate: jest.fn().mockReturnValue({ destroy: destroyMock }),
    };

    const swarm = {
      on: jest.fn(
        (event: string, cb: (socket: unknown, peerInfo: unknown) => void) => {
          if (event === "connection") {
            cb(fakeSocket, fakePeerInfo);
          }
        },
      ),
      off: jest.fn(),
    };

    const result = await retryConnectAndReplicate(swarm, 1000, drive, true);

    expect(typeof result.destroy).toBe("function");
    expect(drive.replicate).toHaveBeenCalledWith(true);
  });

  it("EC-B2.c-2: a second failure during retry propagates the same typed error connectAndReplicate already defines, not a new error shape", async () => {
    jest.useFakeTimers();

    const destroyMock = jest.fn();
    const drive = {
      replicate: jest.fn().mockReturnValue({ destroy: destroyMock }),
    };

    const swarm = {
      on: jest.fn(),
      off: jest.fn(),
    };

    const promise = retryConnectAndReplicate(swarm, 50, drive, true);

    expect(promise).rejects.toThrow(RuntimeProtocolError);

    await jest.advanceTimersByTimeAsync(50);

    await expect(promise).rejects.toThrow(RuntimeProtocolError);
    await expect(promise).rejects.toMatchObject({
      code: "REPLICATION_CONNECT_FAILED",
    });

    jest.useRealTimers();
  });
});
