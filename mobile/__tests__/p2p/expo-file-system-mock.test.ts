import { createExpoFileSystemMock } from '../../test-utils/expo-file-system-mock';

describe('createExpoFileSystemMock', () => {
  it('HP: constructing Directory twice with identical arguments returns the SAME object', () => {
    const { Directory, Paths } = createExpoFileSystemMock();
    const dir1 = new Directory(Paths.cache, 'dubbridge-p2p', 'proofs', 'abc12345');
    const dir2 = new Directory(Paths.cache, 'dubbridge-p2p', 'proofs', 'abc12345');
    expect(dir1).toBe(dir2);
    dir1.exists = false;
    expect(dir2.exists).toBe(false);
  });

  it('HP: createExpoFileSystemMock called twice returns two independent factories', () => {
    const mock1 = createExpoFileSystemMock();
    const mock2 = createExpoFileSystemMock();
    const dir1 = new mock1.Directory(mock1.Paths.cache, 'test');
    const dir2 = new mock2.Directory(mock2.Paths.cache, 'test');
    expect(dir1).not.toBe(dir2);
  });

  it('EC: after calling __reset(), constructing Directory with same arguments returns a NEW object', () => {
    const { Directory, Paths, __reset } = createExpoFileSystemMock();
    const dir1 = new Directory(Paths.cache, 'test');
    __reset();
    const dir2 = new Directory(Paths.cache, 'test');
    expect(dir1).not.toBe(dir2);
  });

  it('EC: File instances follow the same identity-sharing behavior as Directory', () => {
    const { File, Paths } = createExpoFileSystemMock();
    const file1 = new File(Paths.cache, 'test.txt');
    const file2 = new File(Paths.cache, 'test.txt');
    expect(file1).toBe(file2);
    file1.exists = false;
    expect(file2.exists).toBe(false);
  });

  it('EC: Directory and File constructed with the same argument sequence are NOT the same object', () => {
    const { Directory, File, Paths } = createExpoFileSystemMock();
    const dir = new Directory(Paths.cache, 'test');
    const file = new File(Paths.cache, 'test');
    expect(dir).not.toBe(file);
  });
});
