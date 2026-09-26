import { replicateOverSocket, type DuplexSocket, type ReplicableDrive } from "../../src/p2p/runtime/transient-replication";
import { attachByteCounter } from "../../src/p2p/runtime/byte-counter";

describe("transient-replication byte count", () => {
  it("HP-B2.a-0: attachByteCounter on a socket whose on('data', cb) invokes cb with chunks of length 10 and 5 reports getByteCount() === 15", () => {
    let dataListener: ((chunk: { length: number }) => void) | null = null;
    const socket: DuplexSocket = {
      pipe: jest.fn(),
      on: jest.fn((event: string, listener: (chunk: { length: number }) => void) => {
        if (event === "data") {
          dataListener = listener;
        }
      }),
    };

    const counter = attachByteCounter(socket);
    expect(counter.getByteCount()).toBe(0);

    dataListener!({ length: 10 });
    dataListener!({ length: 5 });

    expect(counter.getByteCount()).toBe(15);
  });

  it("EC-B2.a-0: attachByteCounter on a socket with no on method does not throw, and getByteCount() returns 0", () => {
    const socket: DuplexSocket = {
      pipe: jest.fn(),
    };

    expect(() => attachByteCounter(socket)).not.toThrow();
    const counter = attachByteCounter(socket);
    expect(counter.getByteCount()).toBe(0);
  });

  it("HP-B2.a-0 (integration): replicateOverSocket with a socket double that emits a 'data' event carrying a chunk of length 20 right after pipe is called, and a drive double whose replicate() returns { destroy: jest.fn() }, results in a returned object whose getByteCount() returns 20", () => {
    let dataListener: ((chunk: { length: number }) => void) | null = null;
    const socket: DuplexSocket = {
      pipe: jest.fn((destination: unknown) => {
        if (dataListener) {
          dataListener({ length: 20 });
        }
        return { pipe: jest.fn() };
      }),
      on: jest.fn((event: string, listener: (chunk: { length: number }) => void) => {
        if (event === "data") {
          dataListener = listener;
        }
      }),
    };

    const drive: ReplicableDrive = {
      replicate: jest.fn(() => ({ destroy: jest.fn() })),
    };

    const result = replicateOverSocket(drive, socket, true);
    expect(result.getByteCount()).toBe(20);
  });
});
