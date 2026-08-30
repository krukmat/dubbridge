import type { createExpoFileSystemMock } from '../../test-utils/expo-file-system-mock';

jest.mock('expo-file-system', () => {
  const { createExpoFileSystemMock: mockFactory } = require('../../test-utils/expo-file-system-mock');
  return mockFactory();
});

import { TransientStorageError, isWithinProofRoot, deleteProofRunDirectory, listAbandonedProofRuns } from '../../src/p2p/proof/transient-storage';

const mockFs: ReturnType<typeof createExpoFileSystemMock> = jest.requireMock('expo-file-system');

describe('transient-storage', () => {
  beforeEach(() => {
    mockFs.__reset();
  });

  test('HP: deleteProofRunDirectory on a directory that exists deletes it and resolves without throwing', async () => {
    const runId = 'abc12345';
    const dir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', runId);
    dir.exists = true;
    dir.deleteFn = () => {
      dir.exists = false;
    };

    await expect(deleteProofRunDirectory(runId)).resolves.not.toThrow();
    expect(dir.exists).toBe(false);
  });

  test('HP: listAbandonedProofRuns returns only run-id-shaped directory names older than the age bound', async () => {
    const parentDir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs');
    parentDir.exists = true;

    const oldRunId = 'oldrun123';
    const freshRunId = 'freshrun123';
    const oldDir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', oldRunId);
    const freshDir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', freshRunId);
    const nonRunIdDir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', 'invalid');
    const fileEntry = new mockFs.File(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', 'file.txt');

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

  test('EC: deleteProofRunDirectory with a runId that fails the regex throws TransientStorageError with code "TRANSIENT_STORAGE_PATH_INVALID"', async () => {
    await expect(deleteProofRunDirectory('invalid')).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory('invalid')).rejects.toMatchObject({ code: 'TRANSIENT_STORAGE_PATH_INVALID' });
  });

  test('EC: deleteProofRunDirectory on a directory whose .exists is false throws TransientStorageError with code "TRANSIENT_STORAGE_NOT_FOUND"', async () => {
    const runId = 'abc12345';
    const dir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', runId);
    dir.exists = false;

    await expect(deleteProofRunDirectory(runId)).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory(runId)).rejects.toMatchObject({ code: 'TRANSIENT_STORAGE_NOT_FOUND' });
  });

  test('EC: if the mocked .delete() throws, deleteProofRunDirectory throws TransientStorageError with code "TRANSIENT_STORAGE_DELETE_FAILED"', async () => {
    const runId = 'abc12345';
    const dir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', runId);
    dir.exists = true;
    dir.deleteFn = () => {
      throw new Error('Delete failed');
    };

    await expect(deleteProofRunDirectory(runId)).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory(runId)).rejects.toMatchObject({ code: 'TRANSIENT_STORAGE_DELETE_FAILED' });
  });

  test('EC: if .delete() succeeds but .exists is still true afterward, deleteProofRunDirectory throws TransientStorageError with code "TRANSIENT_STORAGE_VERIFY_FAILED"', async () => {
    const runId = 'abc12345';
    const dir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', runId);
    dir.exists = true;
    dir.deleteFn = () => {
      // Simulate delete not actually removing the directory
      dir.exists = true;
    };

    await expect(deleteProofRunDirectory(runId)).rejects.toThrow(TransientStorageError);
    await expect(deleteProofRunDirectory(runId)).rejects.toMatchObject({ code: 'TRANSIENT_STORAGE_VERIFY_FAILED' });
  });

  test('EC: isWithinProofRoot rejects a URI containing .., a URI for a different runId, and a URI entirely outside the proofs directory; accepts the exact proof-root URI and a nested path under it', () => {
    const runId = 'abc12345';
    const proofRoot = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', runId).uri;

    expect(isWithinProofRoot(proofRoot, runId)).toBe(true);
    expect(isWithinProofRoot(proofRoot + '/nested/file.txt', runId)).toBe(true);
    expect(isWithinProofRoot(proofRoot + '/../other', runId)).toBe(false);
    expect(isWithinProofRoot(new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs', 'different').uri, runId)).toBe(false);
    expect(isWithinProofRoot(mockFs.Paths.cache + '/other/path', runId)).toBe(false);
  });

  test('EC: listAbandonedProofRuns returns [] without calling .list() when the parent directory\'s .exists is false', async () => {
    const parentDir = new mockFs.Directory(mockFs.Paths.cache, 'dubbridge-p2p', 'proofs');
    parentDir.exists = false;
    parentDir.listFn = () => {
      throw new Error('list() should not be called');
    };

    const result = await listAbandonedProofRuns(3000, () => 4000);

    expect(result).toEqual([]);
  });
});
