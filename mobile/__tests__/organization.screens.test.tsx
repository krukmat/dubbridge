import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react-native";

import { createGatewayClient } from "../src/api/client";
import type { AuthContextValue } from "../src/auth/AuthProvider";
import { OrganizationListScreen } from "../src/screens/OrganizationListScreen";
import { OrganizationMembersScreen } from "../src/screens/OrganizationMembersScreen";

(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

jest.mock("../src/auth/AuthProvider", () => ({ useAuth: () => mockAuth }));
jest.mock("../src/api/client", () => ({ createGatewayClient: jest.fn() }));

const mockCreateGatewayClient = createGatewayClient as jest.MockedFunction<typeof createGatewayClient>;
const ORGANIZATION = {
  id: "org-001",
  name: "Acme Studio",
  viewer_role: "owner" as const,
  created_at: "2026-06-13T00:00:00Z",
  updated_at: "2026-06-13T00:00:00Z",
};

let mockAuth: AuthContextValue;
let mockClient: { get: jest.Mock; post: jest.Mock; postMultipart: jest.Mock };

describe("organization screens", () => {
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

  it("HP-1: lists organizations and opens projects with the selected organization", async () => {
    mockClient.get.mockResolvedValueOnce({ ok: true, value: { data: [ORGANIZATION], sessionRotation: "rotated" } });
    const onOpenProjects = jest.fn();
    const view = await render(
      <OrganizationListScreen
        gatewayBaseUrl="http://gateway"
        onOpenProjects={onOpenProjects}
        onOpenMembers={jest.fn()}
      />,
    );

    await waitFor(() => expect(view.getByText("Acme Studio")).toBeTruthy());
    await act(async () => {
      fireEvent.press(view.getByTestId("organization-projects-org-001"));
    });

    expect(onOpenProjects).toHaveBeenCalledWith(ORGANIZATION);
    expect(mockAuth.onSessionRotation).toHaveBeenCalledWith("rotated");
  });

  it("HP-2: creates an organization and opens its projects", async () => {
    mockClient.get.mockResolvedValueOnce({ ok: true, value: { data: [], sessionRotation: null } });
    mockClient.post.mockResolvedValueOnce({ ok: true, value: { data: ORGANIZATION, sessionRotation: null } });
    const onOpenProjects = jest.fn();
    const view = await render(
      <OrganizationListScreen
        gatewayBaseUrl="http://gateway"
        onOpenProjects={onOpenProjects}
        onOpenMembers={jest.fn()}
      />,
    );

    await waitFor(() => expect(view.getByTestId("organization-list-empty")).toBeTruthy());
    await act(async () => {
      fireEvent.changeText(view.getByTestId("organization-name-input"), " Acme Studio ");
    });
    await act(async () => {
      fireEvent.press(view.getByTestId("organization-create"));
    });

    await waitFor(() => expect(onOpenProjects).toHaveBeenCalledWith(ORGANIZATION));
    expect(mockClient.post).toHaveBeenCalledWith("/api/orgs", "opaque-session", { name: "Acme Studio" });
  });

  it("EC-2: rejects a blank organization name without a request", async () => {
    mockClient.get.mockResolvedValueOnce({ ok: true, value: { data: [], sessionRotation: null } });
    const view = await render(
      <OrganizationListScreen gatewayBaseUrl="http://gateway" onOpenProjects={jest.fn()} onOpenMembers={jest.fn()} />,
    );
    await waitFor(() => expect(view.getByTestId("organization-list-empty")).toBeTruthy());

    await act(async () => {
      fireEvent.press(view.getByTestId("organization-create"));
    });

    expect(view.getByText("Organization name is required.")).toBeTruthy();
    expect(mockClient.post).not.toHaveBeenCalled();
  });

  it("EC-3: logs out when organization listing reports session expiry", async () => {
    mockClient.get.mockResolvedValueOnce({ ok: false, error: { kind: "session_expired" } });
    await render(<OrganizationListScreen gatewayBaseUrl="http://gateway" onOpenProjects={jest.fn()} onOpenMembers={jest.fn()} />);
    await waitFor(() => expect(mockAuth.logout).toHaveBeenCalledTimes(1));
  });

  it("HP-1 members: owner adds a member and the returned row appears", async () => {
    mockClient.get.mockResolvedValueOnce({ ok: true, value: { data: [], sessionRotation: null } });
    const member = { org_id: "org-001", subject_id: "11111111-1111-1111-1111-111111111111", role: "reviewer", joined_at: "2026-06-13T00:00:00Z" };
    mockClient.post.mockResolvedValueOnce({ ok: true, value: { data: member, sessionRotation: null } });
    const view = await render(<OrganizationMembersScreen gatewayBaseUrl="http://gateway" orgId="org-001" viewerRole="owner" />);

    await waitFor(() => expect(view.getByTestId("member-list-empty")).toBeTruthy());
    await act(async () => {
      fireEvent.changeText(view.getByTestId("member-subject-input"), member.subject_id);
      fireEvent.press(view.getByTestId("member-role-reviewer"));
    });
    await act(async () => {
      fireEvent.press(view.getByTestId("member-add"));
    });

    await waitFor(() => expect(view.getByTestId(`member-row-${member.subject_id}`)).toBeTruthy());
    expect(mockClient.post).toHaveBeenCalledWith(`/api/orgs/org-001/members`, "opaque-session", {
      subject_id: member.subject_id,
      role: "reviewer",
    });
  });

  it.each(["viewer", "reviewer", "editor"] as const)("EC-1: %s cannot see add-member controls", async (viewerRole) => {
    mockClient.get.mockResolvedValueOnce({ ok: true, value: { data: [], sessionRotation: null } });
    const view = await render(<OrganizationMembersScreen gatewayBaseUrl="http://gateway" orgId="org-001" viewerRole={viewerRole} />);
    await waitFor(() => expect(view.getByTestId("member-list-empty")).toBeTruthy());
    expect(view.queryByTestId("member-add-controls")).toBeNull();
  });

  it("HP-1 large-list: 100 organizations render with stable testIDs", async () => {
    const organizations = Array.from({ length: 100 }, (_, i) => ({
      id: `org-${String(i).padStart(3, "0")}`,
      name: `Studio ${i}`,
      viewer_role: "viewer" as const,
      created_at: "2026-06-01T00:00:00Z",
      updated_at: "2026-06-01T00:00:00Z",
    }));

    mockClient.get.mockResolvedValueOnce({
      ok: true,
      value: { data: organizations, sessionRotation: null },
    });

    const view = await render(
      <OrganizationListScreen
        gatewayBaseUrl="http://gateway"
        onOpenProjects={jest.fn()}
        onOpenMembers={jest.fn()}
      />,
    );

    await waitFor(() => expect(view.getByTestId("organization-card-org-005")).toBeTruthy());
    expect(view.getByTestId("organization-card-org-005")).toBeTruthy();
  });

  it("HP-2 large-list: 100 members render with stable testIDs and role copy preserved", async () => {
    const members = Array.from({ length: 100 }, (_, i) => ({
      org_id: "org-001",
      subject_id: `user-${String(i).padStart(3, "0")}`,
      role: (i % 2 === 0 ? "viewer" : "reviewer") as "viewer" | "reviewer",
      joined_at: "2026-06-01T00:00:00Z",
    }));

    mockClient.get.mockResolvedValueOnce({ ok: true, value: { data: members, sessionRotation: null } });

    const view = await render(
      <OrganizationMembersScreen gatewayBaseUrl="http://gateway" orgId="org-001" viewerRole="owner" />,
    );

    await waitFor(() => expect(view.getByTestId("member-row-user-005")).toBeTruthy());

    const row = view.getByTestId("member-row-user-005");
    expect(row).toBeTruthy();
    expect(view.getAllByText(members[5].role).length).toBeGreaterThan(0);
  });
});
