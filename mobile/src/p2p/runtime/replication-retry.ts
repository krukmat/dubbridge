import { connectAndReplicate } from "./transient-replication";

export async function retryConnectAndReplicate(
  ...args: Parameters<typeof connectAndReplicate>
): ReturnType<typeof connectAndReplicate> {
  return connectAndReplicate(...args);
}
