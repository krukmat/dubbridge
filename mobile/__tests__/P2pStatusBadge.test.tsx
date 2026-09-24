import { render } from "@testing-library/react-native";

import {
  P2pStatusBadge,
  p2pOwnerStatusPresentation,
  p2pViewerStatusPresentation,
} from "../src/p2p/dashboard/P2pStatusBadge";

describe("P2pStatusBadge", () => {
  it.each([
    ["processing", "Processing", "info"],
    ["ready", "Ready", "success"],
    ["failed", "Failed", "danger"],
  ] as const)("certifies owner %s presentation", (state, label, tone) => {
    expect(p2pOwnerStatusPresentation(state)).toEqual({ label, tone });
    const view = render(
      <P2pStatusBadge surface="owner" state={state} testID="owner-status" />,
    );
    expect(view.getByTestId("owner-status")).toBeTruthy();
    expect(view.getByText(label)).toBeTruthy();
  });

  it.each([
    ["pending", "Pending", "info"],
    ["syncing", "Syncing", "info"],
    ["sync_error", "Sync error", "danger"],
    ["available", "Available", "success"],
    ["expired", "Expired", "warning"],
  ] as const)("certifies viewer %s presentation", (state, label, tone) => {
    expect(p2pViewerStatusPresentation(state)).toEqual({ label, tone });
    const view = render(
      <P2pStatusBadge surface="viewer" state={state} testID="viewer-status" />,
    );
    expect(view.getByTestId("viewer-status")).toBeTruthy();
    expect(view.getByText(label)).toBeTruthy();
  });
});
