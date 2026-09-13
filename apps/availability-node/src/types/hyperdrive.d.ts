// Minimal local type declarations for `hyperdrive`, scoped to the surface
// this codebase actually calls. Upstream ships no .d.ts and no @types
// package exists; these are hand-written against the installed version
// (hyperdrive@^13.3.3), not a full API surface.
declare module "hyperdrive" {
  import type Corestore from "corestore";

  export default class Hyperdrive {
    constructor(store: Corestore);
    readonly key: Buffer;
    ready(): Promise<void>;
    close(): Promise<void>;
    put(path: string, content: Buffer | Uint8Array): Promise<void>;
    get(path: string): Promise<Buffer | null>;
  }
}
