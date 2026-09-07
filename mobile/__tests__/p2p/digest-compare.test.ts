import { compareDigest, DigestHash } from "../../src/p2p/runtime/digest-compare";
import { RuntimeProtocolError } from "../../src/p2p/runtime/protocol";

describe("compareDigest", () => {
  it("HP-B2.a-i: returns matched true for matching digest", () => {
    const content = new Uint8Array([1, 2, 3]);
    const expectedHexDigest = "abc123";

    const mockHash: DigestHash = {
      update: jest.fn().mockReturnThis(),
      digest: jest.fn().mockReturnValue(expectedHexDigest),
    };

    const createHash = jest.fn().mockReturnValue(mockHash);

    const result = compareDigest(createHash, content, expectedHexDigest);

    expect(result).toEqual({ matched: true });
    expect(createHash).toHaveBeenCalledWith("sha256");
    expect(mockHash.update).toHaveBeenCalledWith(content);
    expect(mockHash.digest).toHaveBeenCalledWith("hex");
  });

  it("EC-B2.a-i: returns matched false for mismatched digest", () => {
    const content = new Uint8Array([1, 2, 3]);
    const expectedHexDigest = "different";

    const mockHash: DigestHash = {
      update: jest.fn().mockReturnThis(),
      digest: jest.fn().mockReturnValue("abc123"),
    };

    const createHash = jest.fn().mockReturnValue(mockHash);

    const result = compareDigest(createHash, content, expectedHexDigest);

    expect(result).toEqual({ matched: false });
  });

  it("EC-B2.a-i: throws RuntimeProtocolError when hash operation fails", () => {
    const content = new Uint8Array([1, 2, 3]);
    const expectedHexDigest = "abc123";

    const mockHash: DigestHash = {
      update: jest.fn().mockImplementation(() => {
        throw new Error("Hash update failed");
      }),
      digest: jest.fn(),
    };

    const createHash = jest.fn().mockReturnValue(mockHash);

    expect(() => compareDigest(createHash, content, expectedHexDigest)).toThrow(RuntimeProtocolError);
    expect(() => compareDigest(createHash, content, expectedHexDigest)).toThrowError(
      expect.objectContaining({
        code: "DIGEST_COMPARE_FAILED",
      })
    );
  });
});
