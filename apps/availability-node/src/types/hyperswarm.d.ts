// Minimal local type declarations for `hyperswarm`, scoped to the surface
// this codebase actually calls. Upstream ships no .d.ts and no @types
// package exists; these are hand-written against the installed version
// (hyperswarm@^4.17.1), not a full API surface.
declare module "hyperswarm" {
  import type { Duplex } from "node:stream";

  export interface HyperswarmJoinOptions {
    server?: boolean;
    client?: boolean;
  }

  export interface HyperswarmDiscovery {
    flushed(): Promise<boolean>;
    destroy(): Promise<void>;
  }

  export default class Hyperswarm {
    join(topic: Buffer, opts?: HyperswarmJoinOptions): HyperswarmDiscovery;
    status(topic: Buffer): HyperswarmDiscovery | null;
    destroy(): Promise<void>;
    on(event: "connection", listener: (connection: Duplex) => void): this;
  }
}
