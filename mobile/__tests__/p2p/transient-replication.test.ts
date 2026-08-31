import {
  replicateOverSocket,
  cancelReplicationOnTimeout,
  connectAndReplicate,
  connectReplicateAndCancelOnTimeout,
} from "../../src/p2p/runtime/transient-replication";
import { RuntimeProtocolError } from "../../src/p2p/runtime/protocol";

describe("transient-replication", () => {
  describe("replicateOverSocket", () => {
    it("HP-B2.a-cov: returns object with destroy function and calls underlying destroy", () => {
      const destroyMock = jest.fn();
      const drive = {
        replicate: jest.fn().mockReturnValue({ destroy: destroyMock }),
      };
      const pipeMock = jest.fn().mockReturnValue({ pipe: jest.fn() });
      const socket = { pipe: pipeMock };

      const result = replicateOverSocket(drive, socket, true);

      expect(typeof result.destroy).toBe("function");
      expect(drive.replicate).toHaveBeenCalledWith(true);

      result.destroy();
      expect(destroyMock).toHaveBeenCalledTimes(1);
    });

    it("EC-B2.a-cov: throws RuntimeProtocolError when replicate throws", () => {
      const drive = {
        replicate: jest.fn().mockImplementation(() => {
          throw new Error("replicate failed");
        }),
      };
      const socket = { pipe: jest.fn() };

      expect(() => replicateOverSocket(drive, socket, false)).toThrow(RuntimeProtocolError);
      try {
        replicateOverSocket(drive, socket, false);
      } catch (e) {
        expect(e).toBeInstanceOf(RuntimeProtocolError);
        expect((e as RuntimeProtocolError).code).toBe("REPLICATION_TRANSFER_FAILED");
      }
    });
  });

  describe("cancelReplicationOnTimeout", () => {
    it("HP-B2.a-cov: cancel before timeout prevents destroy", () => {
      jest.useFakeTimers();
      const destroyMock = jest.fn();
      const replicationStream = { destroy: destroyMock, getByteCount: () => 0 };

      const { cancel } = cancelReplicationOnTimeout(replicationStream, 1000);
      cancel();

      jest.advanceTimersByTime(1000);
      expect(destroyMock).not.toHaveBeenCalled();
      jest.useRealTimers();
    });

    it("EC-B2.a-cov: timeout fires and rejects promise", async () => {
      jest.useFakeTimers();
      const destroyMock = jest.fn();
      const replicationStream = { destroy: destroyMock, getByteCount: () => 0 };

      const { promise } = cancelReplicationOnTimeout(replicationStream, 50);
      const rejection = expect(promise).rejects.toThrow(RuntimeProtocolError);

      await jest.advanceTimersByTimeAsync(50);
      await rejection;
      expect(destroyMock).toHaveBeenCalledTimes(1);
      jest.useRealTimers();
    });
  });

  describe("connectAndReplicate", () => {
    it("HP-B2.a-cov: resolves with destroy function when connection is immediate", async () => {
      const destroyMock = jest.fn();
      const drive = {
        replicate: jest.fn().mockReturnValue({ destroy: destroyMock }),
      };
      const fakeSocket = { pipe: jest.fn().mockReturnValue({ pipe: jest.fn() }) };
      const fakePeerInfo = { id: "peer-1" };

      const swarm = {
        on: jest.fn((event: string, cb: (socket: unknown, peerInfo: unknown) => void) => {
          if (event === "connection") {
            cb(fakeSocket, fakePeerInfo);
          }
        }),
        off: jest.fn(),
      };

      const result = await connectAndReplicate(swarm, 1000, drive, true);
      expect(typeof result.destroy).toBe("function");
      expect(drive.replicate).toHaveBeenCalledWith(true);
    });

    it("EC-B2.a-cov: rejects with REPLICATION_CONNECT_FAILED on timeout", async () => {
      jest.useFakeTimers();
      const drive = {
        replicate: jest.fn().mockReturnValue({ destroy: jest.fn() }),
      };
      const swarm = {
        on: jest.fn(),
        off: jest.fn(),
      };

      const promise = connectAndReplicate(swarm, 50, drive, true);
      const rejection = expect(promise).rejects.toThrow(RuntimeProtocolError);

      await jest.advanceTimersByTimeAsync(50);
      await rejection;
      jest.useRealTimers();
    });
  });

  describe("connectReplicateAndCancelOnTimeout", () => {
    it("HP-B2.a-cov: resolves when finishedSignal resolves", async () => {
      const destroyMock = jest.fn();
      const drive = {
        replicate: jest.fn().mockReturnValue({ destroy: destroyMock }),
      };
      const fakeSocket = { pipe: jest.fn().mockReturnValue({ pipe: jest.fn() }) };
      const fakePeerInfo = { id: "peer-1" };

      const swarm = {
        on: jest.fn((event: string, cb: (socket: unknown, peerInfo: unknown) => void) => {
          if (event === "connection") {
            cb(fakeSocket, fakePeerInfo);
          }
        }),
        off: jest.fn(),
      };

      const result = await connectReplicateAndCancelOnTimeout(
        swarm,
        1000,
        drive,
        true,
        1000,
        Promise.resolve(),
      );
      expect(typeof result.destroy).toBe("function");
    });

    it("EC-B2.a-cov: rejects with REPLICATION_TRANSFER_FAILED on transfer timeout", async () => {
      jest.useFakeTimers();
      const destroyMock = jest.fn();
      const drive = {
        replicate: jest.fn().mockReturnValue({ destroy: destroyMock }),
      };
      const fakeSocket = { pipe: jest.fn().mockReturnValue({ pipe: jest.fn() }) };
      const fakePeerInfo = { id: "peer-1" };

      const swarm = {
        on: jest.fn((event: string, cb: (socket: unknown, peerInfo: unknown) => void) => {
          if (event === "connection") {
            cb(fakeSocket, fakePeerInfo);
          }
        }),
        off: jest.fn(),
      };

      const finishedSignal = new Promise<void>(() => undefined);
      const promise = connectReplicateAndCancelOnTimeout(
        swarm,
        1000,
        drive,
        true,
        50,
        finishedSignal,
      );
      const rejection = expect(promise).rejects.toThrow(RuntimeProtocolError);

      await jest.advanceTimersByTimeAsync(50);
      await rejection;
      jest.useRealTimers();
    });
  });
});
