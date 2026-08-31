import { RuntimeProtocolError, type RuntimeProtocolErrorCode } from "./protocol";

export function rethrowAsProtocolError(
  error: unknown,
  code: RuntimeProtocolErrorCode,
  message: string,
): never {
  if (error instanceof RuntimeProtocolError) throw error;
  throw new RuntimeProtocolError(code, message);
}
