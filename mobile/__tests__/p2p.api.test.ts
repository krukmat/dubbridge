import type { GatewayClient } from "../src/api/client";
import {
  claimP2pInvitation,
  createP2pInvitation,
  registerP2pDevice,
} from "../src/api/p2p";

function ok<T>(data: T) {
  return Promise.resolve({
    ok: true as const,
    value: { data, sessionRotation: null },
  });
}

function client(overrides: Partial<GatewayClient>): GatewayClient {
  return {
    get: jest.fn(),
    post: jest.fn(),
    postMultipart: jest.fn(),
    ...overrides,
  } as GatewayClient;
}

describe("P2P audience API", () => {
  it("registers the opaque device public identity without private key material", async () => {
    const post = jest.fn().mockImplementation(() =>
      ok({ id: "device-1", key_id: "android-key-1", created_at_unix: 42 }),
    );
    const gateway = client({ post });

    const result = await registerP2pDevice(gateway, "access", "android-key-1", "spki-base64");

    expect(post).toHaveBeenCalledWith("/api/p2p/devices", "access", {
      key_id: "android-key-1",
      public_key_spki_base64: "spki-base64",
    });
    expect(result).toEqual({
      ok: true,
      value: {
        data: { id: "device-1", keyId: "android-key-1", createdAtUnix: 42 },
        sessionRotation: null,
      },
    });
  });

  it("creates an invitation and returns the one-time raw token", async () => {
    const post = jest.fn().mockImplementation(() =>
      ok({
        invitation: {
          id: "invite-1",
          asset_id: "asset-1",
          publication_id: "pub-1",
          lineage_id: "lineage-1",
          owner_subject_id: "owner-1",
          status: "pending",
          expires_at_unix: 99,
          claimed_at_unix: null,
        },
        token: "raw-token",
      }),
    );
    const gateway = client({ post });

    const result = await createP2pInvitation(gateway, "access", "asset/1", 3600);

    expect(post).toHaveBeenCalledWith(
      "/api/assets/asset%2F1/p2p/invitations",
      "access",
      { ttl_seconds: 3600 },
    );
    expect(result.ok && result.value.data.token).toBe("raw-token");
  });

  it("maps a successful claim to the frozen ready descriptor", async () => {
    const post = jest.fn().mockImplementation(() =>
      ok({
        invitation: {
          id: "invite-1",
          asset_id: "asset-1",
          publication_id: "pub-1",
          lineage_id: "lineage-1",
          owner_subject_id: "owner-1",
          status: "claimed",
          expires_at_unix: 99,
          claimed_at_unix: 50,
        },
        authorization: {
          id: "auth-1",
          invitation_id: "invite-1",
          asset_id: "asset-1",
          publication_id: "pub-1",
          lineage_id: "lineage-1",
          viewer_subject_id: "viewer-1",
          device_id: "device-1",
          expires_at_unix: 99,
        },
        descriptor: {
          descriptor_version: "p2p-ready-descriptor-v1",
          asset_id: "asset-1",
          publication_id: "pub-1",
          lineage_id: "lineage-1",
          manifest_version: "p2p-manifest-v1",
          manifest_digest_sha256: "a".repeat(64),
          external_publication_id: "external-1",
          ck_wrap_ref: "p2p-k1-wrap/pub-1/lineage-1",
          kek_id: "kek-1",
          kek_version: 1,
          ready_at: "2026-09-15T00:00:00Z",
        },
      }),
    );
    const gateway = client({ post });

    const result = await claimP2pInvitation(gateway, "access", "raw-token", "device-1");

    expect(post).toHaveBeenCalledWith("/api/p2p/invitations/claim", "access", {
      token: "raw-token",
      device_id: "device-1",
    });
    expect(result.ok && result.value.data.descriptor).toMatchObject({
      descriptorVersion: "p2p-ready-descriptor-v1",
      manifestVersion: "p2p-manifest-v1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
      manifestDigestSha256: "a".repeat(64),
    });
  });
});
