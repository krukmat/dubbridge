export function createExpoFileSystemMock() {
  const registry = new Map<string, any>();

  class Directory {
    name!: string;
    uri!: string;
    exists!: boolean;
    deleteFn!: () => void;
    infoFn!: () => { modificationTime?: number };
    listFn!: () => (Directory | File)[];

    constructor(...args: (string | Directory | File)[]) {
      const joined = args
        .map((arg) => (typeof arg === 'string' ? arg : arg.uri))
        .join('/');
      const key = `Directory:${joined}`;

      if (registry.has(key)) {
        return registry.get(key);
      }

      const last = args[args.length - 1];
      this.name = typeof last === 'string' ? last : last.uri;
      this.uri = `file://mock/${joined}`;
      this.exists = true;
      this.deleteFn = () => {};
      this.infoFn = () => ({ modificationTime: undefined });
      this.listFn = () => [];

      registry.set(key, this);
      return this;
    }

    delete(): void {
      this.deleteFn();
    }

    info(): { modificationTime?: number } {
      return this.infoFn();
    }

    list(): (Directory | File)[] {
      return this.listFn();
    }
  }

  class File {
    name!: string;
    uri!: string;
    exists!: boolean;

    constructor(...args: (string | Directory | File)[]) {
      const joined = args
        .map((arg) => (typeof arg === 'string' ? arg : arg.uri))
        .join('/');
      const key = `File:${joined}`;

      if (registry.has(key)) {
        return registry.get(key);
      }

      const last = args[args.length - 1];
      this.name = typeof last === 'string' ? last : last.uri;
      this.uri = `file://mock/${joined}`;
      this.exists = true;

      registry.set(key, this);
      return this;
    }
  }

  const Paths = {
    cache: 'file://cache',
  };

  function __reset(): void {
    registry.clear();
  }

  return { Directory, File, Paths, __reset };
}
