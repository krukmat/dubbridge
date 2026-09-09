import type { IncomingMessage, ServerResponse } from "node:http";
import type { Server as HttpsServer } from "node:https";
import { TLSSocket } from "node:tls";
import { TextDecoder } from "node:util";
import {
  PATH_PREFIX,
  METHOD_PUT,
  parsePublicationRequest,
  parsePublicationResponse,
  serializePublicationBody,
  createPublicationError,
  PublicationContractError,
  type ErrorCode,
  type PublicationRequest,
  type PublicationHttpResponse,
} from "./contract.js";
import {
  createClientFingerprintPolicy,
  createPrivateMtlsServer,
  type PrivateMtlsCredentials,
} from "./mtls.js";

export const MAX_REQUEST_BODY_BYTES = 64 * 1024;

export type PublicationExecutor = (
  request: PublicationRequest
) => PublicationHttpResponse | Promise<PublicationHttpResponse>;

export const unavailablePublicationExecutor: PublicationExecutor = () =>
  createPublicationError("publication_unavailable");

export async function handlePublicationIngress(
  request: IncomingMessage,
  response: ServerResponse,
  publisher: PublicationExecutor = unavailablePublicationExecutor
): Promise<void> {
  if (request.method !== METHOD_PUT) {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  let url: URL;
  try {
    url = new URL(request.url ?? "", "http://localhost");
  } catch {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  if (url.search !== "") {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  const path = url.pathname;
  if (!path.startsWith(PATH_PREFIX)) {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  const publicationId = path.slice(PATH_PREFIX.length);
  if (publicationId === "" || publicationId.includes("/")) {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  const contentType = request.headers["content-type"];
  if (typeof contentType !== "string") {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  const ctParts = contentType.split(";").map(s => s.trim());
  if (ctParts[0]!.toLowerCase() !== "application/json") {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  if (ctParts.length > 1) {
    if (ctParts.length !== 2) {
      return sendPublicationResponse(response, createPublicationError("invalid_contract"));
    }
    const charsetPart = ctParts[1]!.toLowerCase();
    if (charsetPart !== "charset=utf-8") {
      return sendPublicationResponse(response, createPublicationError("invalid_contract"));
    }
  }

  const contentLengthHeader = request.headers["content-length"];
  let expectedLength: number | null = null;

  if (contentLengthHeader !== undefined) {
    if (typeof contentLengthHeader !== "string") {
      return sendPublicationResponse(response, createPublicationError("invalid_contract"));
    }
    if (!/^\d+$/.test(contentLengthHeader)) {
      return sendPublicationResponse(response, createPublicationError("invalid_contract"));
    }
    const len = Number(contentLengthHeader);
    if (len > MAX_REQUEST_BODY_BYTES) {
      return sendPublicationResponse(response, createPublicationError("invalid_contract"));
    }
    expectedLength = len;
  }

  const chunks: Buffer[] = [];
  let totalBytes = 0;

  try {
    for await (const chunk of request) {
      if (!Buffer.isBuffer(chunk) && !(chunk instanceof Uint8Array)) {
        return sendPublicationResponse(response, createPublicationError("invalid_contract"));
      }
      const buf = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
      totalBytes += buf.length;
      if (totalBytes > MAX_REQUEST_BODY_BYTES) {
        return sendPublicationResponse(response, createPublicationError("invalid_contract"));
      }
      chunks.push(buf);
    }
  } catch {
    return sendPublicationResponse(response, createPublicationError("publication_unavailable"));
  }

  if (expectedLength !== null && totalBytes !== expectedLength) {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  if (totalBytes === 0) {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  const bodyBuffer = Buffer.concat(chunks);
  const decoder = new TextDecoder("utf-8", { fatal: true });
  let bodyString: string;

  try {
    bodyString = decoder.decode(bodyBuffer);
  } catch {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  if (bodyString.trim() === "") {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  let parsedBody: unknown;
  try {
    parsedBody = JSON.parse(bodyString);
  } catch {
    return sendPublicationResponse(response, createPublicationError("invalid_contract"));
  }

  let validatedRequest: PublicationRequest;
  try {
    validatedRequest = parsePublicationRequest(parsedBody, publicationId);
  } catch (e) {
    if (e instanceof PublicationContractError) {
      if (e.code === "package_invalid") {
        return sendPublicationResponse(response, createPublicationError("package_invalid"));
      }
      return sendPublicationResponse(response, createPublicationError("invalid_contract"));
    }
    return sendPublicationResponse(response, createPublicationError("publication_unavailable"));
  }

  let result: PublicationHttpResponse;
  try {
    result = await publisher(validatedRequest);
  } catch (e) {
    if (e instanceof PublicationContractError) {
      sendPublicationResponse(response, createPublicationError(e.code));
    } else {
      sendPublicationResponse(response, createPublicationError("publication_unavailable"));
    }
    return;
  }

  const wireBody = "evidence" in result ? result.evidence : result.body;

  try {
    const validatedResponse = parsePublicationResponse(result.status, wireBody, validatedRequest);
    sendPublicationResponse(response, validatedResponse);
  } catch (e) {
    sendPublicationResponse(response, createPublicationError("publication_unavailable"));
  }
}

export function createPublicationHandler(
  publisher: PublicationExecutor = unavailablePublicationExecutor
): (req: IncomingMessage, res: ServerResponse) => void {
  return (req: IncomingMessage, res: ServerResponse) => {
    handlePublicationIngress(req, res, publisher).catch(() => {
      if (!res.writableEnded) {
        if (!res.headersSent) {
          sendPublicationResponse(res, createPublicationError("publication_unavailable"));
        } else {
          res.end();
        }
      }
    });
  };
}

export function createPrivatePublicationServer(
  credentials: PrivateMtlsCredentials,
  allowedClientFingerprints: readonly string[],
  publisher: PublicationExecutor = unavailablePublicationExecutor
): HttpsServer {
  const identityAllowed = createClientFingerprintPolicy(allowedClientFingerprints);
  const publicationHandler = createPublicationHandler(publisher);

  return createPrivateMtlsServer(credentials, (request, response) => {
    const socket = request.socket;
    const fingerprint =
      socket instanceof TLSSocket && socket.authorized
        ? socket.getPeerCertificate().fingerprint256
        : undefined;

    if (!identityAllowed(fingerprint)) {
      sendPublicationResponse(
        response,
        createPublicationError("service_identity_rejected")
      );
      return;
    }

    publicationHandler(request, response);
  });
}

function sendPublicationResponse(response: ServerResponse, pubResponse: PublicationHttpResponse): void {
  if (response.headersSent) {
    return;
  }

  const jsonBody = serializePublicationBody(pubResponse);
  response.statusCode = pubResponse.status;
  response.setHeader("Content-Type", "application/json; charset=utf-8");
  response.setHeader("Connection", "close");
  response.setHeader("Content-Length", Buffer.byteLength(jsonBody));
  response.end(jsonBody);
}
