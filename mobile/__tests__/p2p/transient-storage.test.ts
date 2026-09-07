import type { createExpoFileSystemMock } from '../../test-utils/expo-file-system-mock';

jest.mock('expo-file-system', () => {
  const { createExpoFileSystemMock: mockFactory } = require(
    '../../test-utils/expo-file-system-mock',
  );
  return mockFactory();
});

import {
  TransientStorageError,
  isWithinProofRoot,
  deleteProofRunDirectory,
  listAbandonedProofRuns,
} from '../../src/p2p/proof/transient-storage';

const mockFs: ReturnType<typeof createExpoFileSystemMock> = jest.requireMock('expo-file-system');

const PROOFS_DIR = ['dubbridge-p2p', 'proofs'] as const;

function proofRunDir(runId: string) {
  return new mockFs.Directory(mockFs.Paths.cache, ...PROOFS_DIR, runId);
}

function proofsParentDir() {
  return new mockFs.Directory(mockFs.Paths.cache, ...PROOFS_DIR);
}

describe('transient-storage', () => {
  beforeEach(() => {
    mockFs.__reset();
  });

  test('HP: delete on an existing dir removes it and resolves', async () => {
    const runId = 'abc12345';
    const dir = proofRunDir(runId);
    dir.exists = true;
    dir.deleteFn = () => {
      dir.exists = false;
    };

    await expect(deleteProofRunDirectory(runId)).resolves.not.toThrow();
    expect(dir.exists).toBe(false);
  });

  test('HP: listAbandonedProofRuns returns only stale run-id dirs', async () => {
    const parentDir = proofsParentDir();
    parentDir.exists = true;

    const oldRunId = 'oldrun123';
    const freshRunId = 'freshrun123';
    const oldDir = proofRunDir(oldRunId);
    const freshDir = proofRunDir(freshRunId);
    const nonRunIdDir = proofRunDir('invalid');
    const fileEntry = new mockFs.File(mockFs.Paths.cache, ...PROOFS_DIR, 'file.txt');

    oldDir.infoFn = () => ({ modificationTime: 500 });
    freshDir.infoFn = () => ({ modificationTime: 5000 });
    nonRunIdDir.infoFn = () => ({ modificationTime: 500 });

    parentDir.listFn = () => [oldDir, freshDir, nonRunIdDir, fileEntry];

    const result = await listAbandonedProofRuns(3000, () => 4000);

    expect(result).toContain(oldRunId);
    expect(result).not.toContain(freshRunId);
    expect(result).not.toContain('invalid');
    expect(result).not.toContain('file.txt');
    expect(result.length).toBe(1);
  });

  test('EC: delete with an invalid runId throws PATH_INVALID', async () => {
    await expect(deleteProofRunDirectory('invalid')).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory('invalid')).rejects.toMatchObject({
      code: 'TRANSIENT_STORAGE_PATH_INVALID',
    });
  });

  test('EC: delete on a dir whose .exists is false throws NOT_FOUND', async () => {
    const runId = 'abc12345';
    const dir = proofRunDir(runId);
    dir.exists = false;

    await expect(deleteProofRunDirectory(runId)).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory(runId)).rejects.toMatchObject({
      code: 'TRANSIENT_STORAGE_NOT_FOUND',
    });
  });

  test('EC: if the mocked .delete() throws, delete throws DELETE_FAILED', async () => {
    const runId = 'abc12345';
    const dir = proofRunDir(runId);
    dir.exists = true;
    dir.deleteFn = () => {
      throw new Error('Delete failed');
    };

    await expect(deleteProofRunDirectory(runId)).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory(runId)).rejects.toMatchObject({
      code: 'TRANSIENT_STORAGE_DELETE_FAILED',
    });
  });

  test('EC: if .delete() succeeds but .exists stays true, delete throws VERIFY_FAILED', async () => {
    const runId = 'abc12345';
    const dir = proofRunDir(runId);
    dir.exists = true;
    dir.deleteFn = () => {
      // Simulate delete not actually removing the directory
      dir.exists = true;
    };

    await expect(deleteProofRunDirectory(runId)).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory(runId)).rejects.toMatchObject({
      code: 'TRANSIENT_STORAGE_VERIFY_FAILED',
    });
  });

  test('EC: isWithinProofRoot rejects traversal, other runs, and paths outside proofs', () => {
    const runId = 'abc12345';
    const proofRoot = proofRunDir(runId).uri;
    const otherRunRoot = proofRunDir('different').uri;

    expect(isWithinProofRoot(proofRoot, runId)).toBe(true);
    expect(isWithinProofRoot(proofRoot + '/nested/file.txt', runId)).toBe(true);
    expect(isWithinProofRoot(proofRoot + '/../other', runId)).toBe(false);
    expect(isWithinProofRoot(otherRunRoot, runId)).toBe(false);
    expect(isWithinProofRoot(mockFs.Paths.cache + '/other/path', runId)).toBe(false);
  });

  test("EC: listAbandonedProofRuns returns [] without listing when parent doesn't exist", async () => {
    const parentDir = proofsParentDir();
    parentDir.exists = false;
    parentDir.listFn = () => {
      throw new Error('list() should not be called');
    };

    const result = await listAbandonedProofRuns(3000, () => 4000);

    expect(result).toEqual([]);
  });
});
