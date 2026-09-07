import { readTransientDriveFile } from "../../src/p2p/runtime/transient-drive";
import { RuntimeProtocolError } from "../../src/p2p/runtime/protocol";

describe("readTransientDriveFile", () => {
  it("HP-B2.a-ii-a: returns file bytes for an existing path", async () => {
    const bytes = new Uint8Array([1, 2, 3]);
    const get = jest.fn().mockResolvedValue(bytes);
    const drive = { get };

    const result = await readTransientDriveFile(drive, "/some/path");

    expect(result).toEqual(bytes);
    expect(get).toHaveBeenCalledWith("/some/path");
  });

  it("EC-B2.a-ii-a: throws RuntimeProtocolError for a missing path", async () => {
    const get = jest.fn().mockResolvedValue(null);
    const drive = { get };

    await expect(readTransientDriveFile(drive, "/missing")).rejects.toThrow(
      RuntimeProtocolError,
    );
    await expect(readTransientDriveFile(drive, "/missing")).rejects.toMatchObject({
      code: "TRANSIENT_DRIVE_READ_FAILED",
    });
  });

  it("EC-B2.a-ii-a: throws RuntimeProtocolError when the drive read fails", async () => {
    const get = jest.fn().mockRejectedValue(new Error("boom"));
    const drive = { get };

    await expect(readTransientDriveFile(drive, "/some/path")).rejects.toThrow(
      RuntimeProtocolError,
    );
    await expect(readTransientDriveFile(drive, "/some/path")).rejects.toMatchObject({
      code: "TRANSIENT_DRIVE_READ_FAILED",
    });
  });
});