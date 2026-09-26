import { promises as fs } from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

/**
 * Atomically writes contents to targetPath.
 *
 * @param targetPath - The path to the target file.
 * @param contents - The data to write (Uint8Array or Buffer).
 * @returns A promise that resolves when the write is complete.
 */
export async function writeFileAtomic(targetPath: string, contents: Uint8Array): Promise<void> {
    const dir = path.dirname(targetPath);
    const randomSuffix = crypto.randomBytes(16).toString('hex');
    const tempPath = path.join(dir, `.tmp-${randomSuffix}`);

    let fd: fs.FileHandle | null = null;
    try {
        // Open the temp file
        fd = await fs.open(tempPath, 'w');

        // Write the contents
        await fd.writeFile(contents);

        // Fsync to ensure durability
        await fd.sync();

        // Close the file descriptor
        await fd.close();
        fd = null;

        // Atomically rename the temp file to the target path
        await fs.rename(tempPath, targetPath);
    } catch (error) {
        // If an error occurred, clean up the temp file
        if (fd !== null) {
            try {
                await fd.close();
            } catch {
                // Ignore close errors during cleanup
            }
        }

        try {
            await fs.unlink(tempPath);
        } catch {
            // Ignore unlink errors (e.g., if the file was already removed)
        }

        throw error;
    }
}
