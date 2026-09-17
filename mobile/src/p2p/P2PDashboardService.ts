import type { GatewayClient } from "../api/client";
import { listP2pInbox, listP2pOwnerContent } from "../api/p2pDashboard";

/**
 * Product-facing P6 read façade. It exposes only backend-authoritative owner
 * publication state and viewer inbox recovery facts; local sync state remains
 * owned by P2PSyncController.
 */
export class P2PDashboardService {
  constructor(private readonly client: GatewayClient) {}

  listOwnerContent(accessToken: string) {
    return listP2pOwnerContent(this.client, accessToken);
  }

  listInbox(accessToken: string) {
    return listP2pInbox(this.client, accessToken);
  }
}
