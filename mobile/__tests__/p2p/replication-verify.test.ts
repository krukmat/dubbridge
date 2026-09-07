import { verifyReplicatedFile } from "../../src/p2p/runtime/replication-verify";
import { RuntimeProtocolError } from "../../src/p2p/runtime/protocol";

describe("verifyReplicatedFile", () => {
  it("HP-B2.a-ii-b: a byte-perfect replicated fixture verifies as a match", async () => {
    const content = new Uint8Array([1, 2, 3, 4]);
    const drive = {
      get: jest.fn().mockResolvedValue(content),
    };
    
    // Mock createHash to return a digest that matches the content
    const createHash = jest.fn().mockReturnValue({
      update: jest.fn().mockReturnThis(),
      digest: jest.fn().mockReturnValue("expected_digest"),
    });
    
    const result = await verifyReplicatedFile(drive, createHash, "path/to/file", "expected_digest");
    expect(result).toEqual({ matched: true });
  });

  it("EC-B2.a-ii-b (mismatch): a corrupted/incomplete replica verifies as a typed mismatch", async () => {
    const content = new Uint8Array([1, 2, 3, 4]);
    const drive = {
      get: jest.fn().mockResolvedValue(content),
    };
    
    // Mock createHash to return a digest that does NOT match the content
    const createHash = jest.fn().mockReturnValue({
      update: jest.fn().mockReturnThis(),
      digest: jest.fn().mockReturnValue("wrong_digest"),
    });
    
    const result = await verifyReplicatedFile(drive, createHash, "path/to/file", "expected_digest");
    expect(result).toEqual({ matched: false });
  });

  it("EC-B2.a-ii-b (read failure): a read failure propagates as a typed IO error", async () => {
    const drive = {
      get: jest.fn().mockRejectedValue(new RuntimeProtocolError("TRANSIENT_DRIVE_READ_FAILED", "Read failed")),
    };
    
    const createHash = jest.fn();
    
    await expect(verifyReplicatedFile(drive, createHash, "path/to/file", "expected_digest")).rejects.toThrow(RuntimeProtocolError);
    await expect(verifyReplicatedFile(drive, createHash, "path/to/file", "expected_digest")).rejects.toMatchObject({ code: "TRANSIENT_DRIVE_READ_FAILED" });
  });
});