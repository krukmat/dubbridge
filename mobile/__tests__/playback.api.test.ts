import type { GatewayClient } from "../src/api/client";
import { buildManifestUrl, issuePlaybackGrant } from "../src/api/playback";

describe("playback api", () => {
  let client: jest.Mocked<GatewayClient>;

  beforeEach(() => {
    client = {
      get: jest.fn(),
      post: jest.fn(),
      postMultipart: jest.fn(),
    };
  });

  it("HP-1: issues a playback grant against the asset playback-grants path", async () => {
    client.post.mockResolvedValueOnce({
      ok: true,
      value: {
        data: { grant_id: "grant-123" },
        sessionRotation: null,
      },
    });

    const result = await issuePlaybackGrant(
      client,
      "opaque-session-abc123",
      "asset-42",
    );

    expect(client.post).toHaveBeenCalledWith(
      "/api/assets/asset-42/playback-grants",
      "opaque-session-abc123",
      {},
    );
    expect(result).toEqual({
      ok: true,
      value: {
        data: { grantId: "grant-123" },
        sessionRotation: null,
      },
    });
  });

  it("HP-2: builds the fully-qualified manifest URL from known inputs", () => {
    expect(
      buildManifestUrl(
        "https://gateway.example.com/",
        "asset-42",
        "grant-123",
      ),
    ).toBe(
      "https://gateway.example.com/api/assets/asset-42/playback/grant-123/manifest",
    );
  });

  it("EC-1: preserves session_expired without swallowing it", async () => {
    client.post.mockResolvedValueOnce({
      ok: false,
      error: { kind: "session_expired" },
    });

    await expect(
      issuePlaybackGrant(client, "opaque-session-abc123", "asset-42"),
    ).resolves.toEqual({
      ok: false,
      error: { kind: "session_expired" },
    });
  });

  it("EC-2: preserves forbidden without remapping it", async () => {
    client.post.mockResolvedValueOnce({
      ok: false,
      error: { kind: "forbidden" },
    });

    await expect(
      issuePlaybackGrant(client, "opaque-session-abc123", "asset-42"),
    ).resolves.toEqual({
      ok: false,
      error: { kind: "forbidden" },
    });
  });

  it("EC-3: preserves network failures as typed GatewayResult errors", async () => {
    client.post.mockResolvedValueOnce({
      ok: false,
      error: { kind: "network", message: "timeout" },
    });

    await expect(
      issuePlaybackGrant(client, "opaque-session-abc123", "asset-42"),
    ).resolves.toEqual({
      ok: false,
      error: { kind: "network", message: "timeout" },
    });
  });
});
