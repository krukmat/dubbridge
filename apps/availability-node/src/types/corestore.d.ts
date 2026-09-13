// Minimal local type declarations for `corestore`, scoped to the surface
// this codebase actually calls. Upstream ships no .d.ts and no @types
// package exists; these are hand-written against the installed version
// (corestore@^7.12.5), not a full API surface.
declare module "corestore" {
  export default class Corestore {
    constructor(storage: string);
    ready(): Promise<void>;
    close(): Promise<void>;
    namespace(name: string): Corestore;
  }
}
