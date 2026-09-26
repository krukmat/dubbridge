import {
  createAndJoinSwarm,
  awaitFirstConnection,
} from "../../src/p2p/runtime/transient-replication-discovery";
import { RuntimeProtocolError } from "../../src/p2p/runtime/protocol";

describe("transient-replication-discovery", () => {
  describe("createAndJoinSwarm", () => {
    it("HP-B2.a-cov: seed role calls join with server: true, client: false", () => {
      const joinMock = jest.fn().mockReturnValue("discovery-handle");
      const onMock = jest.fn();
      class FakeHyperswarm {
        join = joinMock;
        on = onMock;
      }

      const topic = Buffer.from("topic");
      const result = createAndJoinSwarm(FakeHyperswarm as any, topic, "seed");

      expect(result.discovery).toBe("discovery-handle");
      expect(joinMock).toHaveBeenCalledWith(topic, { server: true, client: false });
    });

    it("HP-B2.a-cov: client role calls join with server: false, client: true", () => {
      const joinMock = jest.fn().mockReturnValue("discovery-handle");
      const onMock = jest.fn();
      class FakeHyperswarm {
        join = joinMock;
        on = onMock;
      }

      const topic = Buffer.from("topic");
      const result = createAndJoinSwarm(FakeHyperswarm as any, topic, "client");

      expect(result.discovery).toBe("discovery-handle");
      expect(joinMock).toHaveBeenCalledWith(topic, { server: false, client: true });
    });
  });

  describe("awaitFirstConnection", () => {
    it("HP-B2.a-cov: resolves when connection event fires", async () => {
      let storedListener: ((socket: unknown, peerInfo: unknown) => void) | null = null;
      const offMock = jest.fn();
      const swarm = {
        on: jest.fn((event: string, cb: (socket: unknown, peerInfo: unknown) => void) => {
          if (event === "connection") {
            storedListener = cb;
          }
        }),
        off: offMock,
      };

      const fakeSocket = { pipe: jest.fn() };
      const fakePeerInfo = { id: "peer-1" };

      const promise = awaitFirstConnection(swarm, 1000);
      storedListener!(fakeSocket, fakePeerInfo);

      const result = await promise;
      expect(result.socket).toBe(fakeSocket);
      expect(result.peerInfo).toBe(fakePeerInfo);
      expect(offMock).toHaveBeenCalledTimes(1);
      expect(offMock).toHaveBeenCalledWith("connection", storedListener);
    });

    it("EC-B2.a-cov: rejects with REPLICATION_CONNECT_FAILED on timeout", async () => {
      jest.useFakeTimers();
      let storedListener: ((socket: unknown, peerInfo: unknown) => void) | null = null;
      const offMock = jest.fn();
      const swarm = {
        on: jest.fn((event: string, cb: (socket: unknown, peerInfo: unknown) => void) => {
          if (event === "connection") {
            storedListener = cb;
          }
        }),
        off: offMock,
      };

      const promise = awaitFirstConnection(swarm, 50);
      const rejection = expect(promise).rejects.toThrow(RuntimeProtocolError);

      await jest.advanceTimersByTimeAsync(50);
      await rejection;
      expect(offMock).toHaveBeenCalledTimes(1);
      jest.useRealTimers();
    });

    it("EC-B2.a-cov: late connection after settle is ignored", async () => {
      jest.useFakeTimers();
      let storedListener: ((socket: unknown, peerInfo: unknown) => void) | null = null;
      const offMock = jest.fn();
      const swarm = {
        on: jest.fn((event: string, cb: (socket: unknown, peerInfo: unknown) => void) => {
          if (event === "connection") {
            storedListener = cb;
          }
        }),
        off: offMock,
      };

      const promise = awaitFirstConnection(swarm, 50);
      const rejection = expect(promise).rejects.toThrow(RuntimeProtocolError);

      await jest.advanceTimersByTimeAsync(50);
      await rejection;

      const fakeSocket = { pipe: jest.fn() };
      const fakePeerInfo = { id: "peer-1" };
      expect(() => storedListener!(fakeSocket, fakePeerInfo)).not.toThrow();
      jest.useRealTimers();
    });
  });
});
