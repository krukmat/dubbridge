import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { ComplianceScreen } from "../src/screens/ComplianceScreen";
import { ConsentScreen } from "../src/screens/ConsentScreen";

(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

jest.mock("../src/auth/AuthProvider", () => ({ useAuth: () => mockAuth }));
jest.mock("../src/api/client", () => ({ createGatewayClient: jest.fn() }));

const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;
let mockAuth: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };

const ok = <T,>(data: T) => ({ ok: true as const, value: { data, sessionRotation: null } });

describe("compliance screens", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockAuth = {
      sessionRef: "opaque-session",
      status: "authed",
      loginError: null,
      login: jest.fn().mockResolvedValue(undefined),
      logout: jest.fn().mockResolvedValue(undefined),
      onSessionRotation: jest.fn().mockResolvedValue(undefined),
    };
    mockClient = { get: jest.fn(), post: jest.fn(), postMultipart: jest.fn() };
    mockCreateGatewayClient.mockReturnValue(mockClient as unknown as ReturnType<typeof createGatewayClient>);
  });

  afterEach(async () => {
    await cleanup();
  });

  it("HP-1: renders audit events chronologically and rights entries", async () => {
    mockClient.get
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", events: [
        { id: "event-late", event_kind: "consent_granted", detail: null, happened_at: "2026-06-13T12:00:00Z" },
        { id: "event-early", event_kind: "ingest_finalized", detail: null, happened_at: "2026-06-13T10:00:00Z" },
      ] }))
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", entries: [
        { id: "rights-1", owner: "Acme", license_type: "owned", source_type: "direct_upload", proof_reference: "proof-1", created_at: "2026-06-13T09:00:00Z" },
      ] }))
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", current_status: "grant", rows: [] }));

    const view = await render(<ComplianceScreen assetId="asset-1" gatewayBaseUrl="http://gateway" onManageConsent={jest.fn()} />);
    await waitFor(() => expect(view.getByTestId("audit-event-event-early")).toBeTruthy());

    expect(view.getByTestId("audit-timeline")).toBeTruthy();
    expect(view.getByTestId("rights-entry-rights-1")).toBeTruthy();
    expect(view.getByText("Active")).toBeTruthy();
    const rendered = view.toJSON();
    expect(JSON.stringify(rendered).indexOf("ingest finalized")).toBeLessThan(JSON.stringify(rendered).indexOf("consent granted"));
  });

  it("EC-1: renders explicit empty audit and rights states", async () => {
    mockClient.get
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", events: [] }))
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", entries: [] }))
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", current_status: null, rows: [] }));
    const view = await render(<ComplianceScreen assetId="asset-1" gatewayBaseUrl="http://gateway" onManageConsent={jest.fn()} />);
    await waitFor(() => expect(view.getByTestId("audit-empty")).toBeTruthy());
    expect(view.getByTestId("rights-empty")).toBeTruthy();
    expect(view.getByText("Inactive")).toBeTruthy();
  });

  it("EC-2: forbidden compliance request shows no governance data", async () => {
    mockClient.get
      .mockResolvedValueOnce({ ok: false, error: { kind: "forbidden" } })
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", entries: [] }))
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", current_status: null, rows: [] }));
    const view = await render(<ComplianceScreen assetId="asset-1" gatewayBaseUrl="http://gateway" onManageConsent={jest.fn()} />);
    await waitFor(() => expect(view.getByText("Could not load compliance data")).toBeTruthy());
    expect(view.queryByTestId("audit-timeline")).toBeNull();
  });

  it("EC-3: session expiry logs out", async () => {
    mockClient.get
      .mockResolvedValueOnce({ ok: false, error: { kind: "session_expired" } })
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", entries: [] }))
      .mockResolvedValueOnce(ok({ asset_id: "asset-1", current_status: null, rows: [] }));
    await render(<ComplianceScreen assetId="asset-1" gatewayBaseUrl="http://gateway" onManageConsent={jest.fn()} />);
    await waitFor(() => expect(mockAuth.logout).toHaveBeenCalledTimes(1));
  });

  it("HP-2: grant consent posts evidence then reloads active status", async () => {
    mockClient.get
      .mockResolvedValueOnce(ok({ current_status: null, rows: [] }))
      .mockResolvedValueOnce(ok({ current_status: "grant", rows: [
        { id: "consent-1", scope: "voice_clone", status: "grant", evidence_ref: "proof://voice", happened_at: "2026-06-13T12:00:00Z" },
      ] }));
    mockClient.post.mockResolvedValueOnce(ok({ current_status: "grant" }));
    const view = await render(<ConsentScreen assetId="asset-1" gatewayBaseUrl="http://gateway" />);
    await waitFor(() => expect(view.getByTestId("consent-history-empty")).toBeTruthy());
    await act(async () => {
      fireEvent.changeText(view.getByTestId("consent-evidence-input"), "proof://voice");
    });
    await act(async () => {
      fireEvent.press(view.getByTestId("consent-grant"));
    });

    await waitFor(() => expect(view.getByTestId("consent-row-consent-1")).toBeTruthy());
    expect(mockClient.post).toHaveBeenCalledWith("/api/consents", "opaque-session", {
      asset_id: "asset-1",
      scope: "voice_clone",
      status: "grant",
      evidence_ref: "proof://voice",
    });
    expect(view.getByText("Active")).toBeTruthy();
  });

  it("HP-3: revoke consent posts without evidence and reloads inactive status", async () => {
    mockClient.get
      .mockResolvedValueOnce(ok({ current_status: "grant", rows: [] }))
      .mockResolvedValueOnce(ok({ current_status: "revoke", rows: [
        { id: "consent-2", scope: "voice_clone", status: "revoke", evidence_ref: null, happened_at: "2026-06-13T13:00:00Z" },
      ] }));
    mockClient.post.mockResolvedValueOnce(ok({ current_status: "revoke" }));
    const view = await render(<ConsentScreen assetId="asset-1" gatewayBaseUrl="http://gateway" />);
    await waitFor(() => expect(view.getByText("Active")).toBeTruthy());
    await act(async () => {
      fireEvent.press(view.getByTestId("consent-revoke"));
    });

    await waitFor(() => expect(view.getByTestId("consent-row-consent-2")).toBeTruthy());
    expect(mockClient.post).toHaveBeenCalledWith("/api/consents", "opaque-session", {
      asset_id: "asset-1",
      scope: "voice_clone",
      status: "revoke",
      evidence_ref: null,
    });
    expect(view.getByText("Inactive")).toBeTruthy();
  });

  it("EC-4: grant without evidence is rejected before POST", async () => {
    mockClient.get.mockResolvedValueOnce(ok({ current_status: null, rows: [] }));
    const view = await render(<ConsentScreen assetId="asset-1" gatewayBaseUrl="http://gateway" />);
    await waitFor(() => expect(view.getByTestId("consent-history-empty")).toBeTruthy());
    await act(async () => {
      fireEvent.press(view.getByTestId("consent-grant"));
    });
    expect(view.getByText("Evidence reference is required to grant consent.")).toBeTruthy();
    expect(mockClient.post).not.toHaveBeenCalled();
  });
});
