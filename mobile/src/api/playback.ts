import type { GatewayClient, GatewayResult } from "./client";

type IssuePlaybackGrantApiResponse = {
  grant_id: string;
};

export type IssuePlaybackGrantResponse = {
  grantId: string;
};

function trimTrailingSlash(value: string): string {
  return value.endsWith("/") ? value.slice(0, -1) : value;
}

export async function issuePlaybackGrant(
  client: GatewayClient,
  sessionRef: string | null,
  assetId: string,
): Promise<GatewayResult<IssuePlaybackGrantResponse>> {
  const result = await client.post<IssuePlaybackGrantApiResponse>(
    `/api/assets/${encodeURIComponent(assetId)}/playback-grants`,
    sessionRef,
    {},
  );

  if (!result.ok) {
    return result;
  }

  return {
    ok: true,
    value: {
      data: { grantId: result.value.data.grant_id },
      sessionRotation: result.value.sessionRotation,
    },
  };
}

export function buildManifestUrl(
  gatewayBaseUrl: string,
  assetId: string,
  grantId: string,
): string {
  const baseUrl = trimTrailingSlash(gatewayBaseUrl);
  const encodedAssetId = encodeURIComponent(assetId);
  const encodedGrantId = encodeURIComponent(grantId);
  return `${baseUrl}/api/assets/${encodedAssetId}/playback/${encodedGrantId}/manifest`;
}
