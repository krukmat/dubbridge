import * as fs from "node:fs";
import * as path from "node:path";

export class ContainmentError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ContainmentError";
  }
}

export function verifyContainedRealpath(root: string, relative: string): string {
  // 1. Canonicalize the root
  let canonicalRoot: string;
  try {
    canonicalRoot = fs.realpathSync(root);
  } catch {
    throw new ContainmentError("Failed to canonicalize root");
  }

  // 2. Join root and relative
  const candidate = path.join(root, relative);

  // 3. Split relative on '/' and filter out empty segments
  const components = relative.split("/").filter((s) => s !== "");

  // 4. Walk components, rejecting any intermediate or final symlink
  let current = canonicalRoot;

  for (const comp of components) {
    current = path.join(current, comp);

    try {
      const meta = fs.lstatSync(current);
      if (meta.isSymbolicLink()) {
        throw new ContainmentError("Symlink escape detected");
      }
    } catch (err) {
      if (err instanceof ContainmentError) {
        throw err;
      }
      const nodeErr = err as NodeJS.ErrnoException;
      if (nodeErr.code === "ENOENT") {
        // Component genuinely does not exist yet — fine for a to-be-created target file.
        continue;
      }
      throw new ContainmentError("Failed to lstat component");
    }
  }

  // 5. Canonicalize the deepest existing ancestor of the final candidate path
  // and confirm it is still inside the canonicalized root.
  let ancestor = path.dirname(current);

  while (true) {
    try {
      fs.lstatSync(ancestor);
      break;
    } catch (err) {
      const nodeErr = err as NodeJS.ErrnoException;
      if (nodeErr.code === "ENOENT") {
        const parent = path.dirname(ancestor);
        if (parent === ancestor) {
          // Reached root of filesystem without finding an existing ancestor
          throw new ContainmentError("No existing ancestor found");
        }
        ancestor = parent;
      } else {
        throw new ContainmentError("Failed to lstat ancestor");
      }
    }
  }

  let canonicalAncestor: string;
  try {
    canonicalAncestor = fs.realpathSync(ancestor);
  } catch {
    throw new ContainmentError("Failed to canonicalize ancestor");
  }

  if (!canonicalAncestor.startsWith(canonicalRoot)) {
    throw new ContainmentError("Path escapes root");
  }

  // 6. Return the non-canonicalized joined path
  return candidate;
}
