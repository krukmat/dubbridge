import type { GatewayClient } from "../../src/api/client";
import { listP2pInbox, listP2pOwnerContent } from "../../src/api/p2pDashboard";
import { P2PDashboardService } from "../../src/p2p/P2PDashboardService";

function clientWith(data: unknown): { client: GatewayClient; get: jest.Mock } {
  const get = jest.fn().mockResolvedValue({
    ok: true,
    value: { data, sessionRotation: null },
  });
  return {
    get,
    client: {
      get,
      post: jest.fn(),
      postMultipart: jest.fn(),
    },
  };
}

const descriptor = {
  descriptor_version: "p2p-ready-descriptor-v1",
  asset_id: "asset-1",
  publication_id: "pub-1",
  lineage_id: "lin-1",
  manifest_version: "p2p-manifest-v1",
  manifest_digest_sha256: "a".repeat(64),
  external_publication_id: "hyperdrive:1",
  ck_wrap_ref: "p2p-k1-wrap/pub-1/lin-1",
  kek_id: "kek-1",
  kek_version: 1,
  ready_at: "2026-09-17T10:00:00.000000Z",
};

describe("P2P dashboard API contracts", () => {
  it("maps authoritative owner content without using generic asset status", async () => {
    const { client, get } = clientWith([
      {
        asset_id: "asset-1",
        title: "Interview selects",
        publication_id: "pub-1",
        lineage_id: "lin-1",
        state: "ready",
        descriptor,
      },
    ]);

    const result = await listP2pOwnerContent(client, "token");

    expect(get).toHaveBeenCalledWith("/api/p2p/content", "token");
    expect(result).toEqual({
      ok: true,
      value: {
        sessionRotation: null,
        data: [
          expect.objectContaining({
            assetId: "asset-1",
            publicationId: "pub-1",
            lineageId: "lin-1",
            state: "ready",
            descriptor: expect.objectContaining({
              manifestDigestSha256: "a".repeat(64),
              externalPublicationId: "hyperdrive:1",
            }),
          }),
        ],
      },
    });
  });

  it("restores claimed viewer authorization while keeping inactive access fail-closed", async () => {
    const { client, get } = clientWith([
      {
        invitation: {
          id: "invite-1",
          asset_id: "asset-1",
          publication_id: "pub-1",
          lineage_id: "lin-1",
          owner_subject_id: "owner-1",
          status: "expired",
          expires_at_unix: 10,
          claimed_at_unix: 5,
        },
        authorization: {
          id: "authz-1",
          invitation_id: "invite-1",
          asset_id: "asset-1",
          publication_id: "pub-1",
          lineage_id: "lin-1",
          viewer_subject_id: "viewer-1",
          device_id: "device-1",
          expires_at_unix: 10,
        },
        authorization_active: false,
        descriptor: null,
      },
    ]);

    const result = await listP2pInbox(client, "token");

    expect(get).toHaveBeenCalledWith("/api/p2p/inbox", "token");
    expect(result).toEqual({
      ok: true,
      value: {
        sessionRotation: null,
        data: [
          {
            invitation: expect.objectContaining({ id: "invite-1", status: "expired" }),
            authorization: expect.objectContaining({ id: "authz-1", viewerSubjectId: "viewer-1" }),
            authorizationActive: false,
            descriptor: null,
          },
        ],
      },
    });
  });

  it("wires both read models through the product dashboard service", async () => {
    const get = jest
      .fn()
      .mockResolvedValueOnce({ ok: true, value: { data: [], sessionRotation: null } })
      .mockResolvedValueOnce({ ok: true, value: { data: [], sessionRotation: null } });
    const client: GatewayClient = {
      get,
      post: jest.fn(),
      postMultipart: jest.fn(),
    };
    const service = new P2PDashboardService(client);

    await service.listOwnerContent("owner-token");
    await service.listInbox("viewer-token");

    expect(get).toHaveBeenNthCalledWith(1, "/api/p2p/content", "owner-token");
    expect(get).toHaveBeenNthCalledWith(2, "/api/p2p/inbox", "viewer-token");
  });
});
