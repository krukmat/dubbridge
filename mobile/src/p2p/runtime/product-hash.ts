import { RuntimeProtocolError } from "./protocol";

interface ProductHash {
  update(data: Uint8Array): ProductHash;
  digest(encoding: "hex"): string;
}

type ProductHashFactory = (algorithm: "sha256") => ProductHash;

function loadHashFactory(): ProductHashFactory {
  try {
    const crypto = require("bare-crypto") as { createHash?: ProductHashFactory };
    if (typeof crypto.createHash !== "function") throw new Error("missing createHash");
    return crypto.createHash;
  } catch {
    throw new RuntimeProtocolError("PRODUCT_HASH_FAILED", "Product ciphertext could not be hashed");
  }
}

/** SHA-256 stays inside the Bare runtime; only ciphertext bytes cross this RPC. */
export function hashProductBytes(bytes: Uint8Array): string {
  try {
    return loadHashFactory()("sha256").update(bytes).digest("hex");
  } catch (error) {
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("PRODUCT_HASH_FAILED", "Product ciphertext could not be hashed");
  }
}
