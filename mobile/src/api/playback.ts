import type { GatewayClient, GatewayErrorKind, GatewayResult } from "./client";

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

export function resolvePlaybackErrorMessage(error: GatewayErrorKind): string {
  if (error.kind === "forbidden") {
    return "You do not have access to this playback stream.";
  }
  if (error.kind === "network") {
    return error.message;
  }
  return `Could not load playback (${(error as Extract<GatewayErrorKind, { kind: "http" }>).status}).`;
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
