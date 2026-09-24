import type { P2pOwnerContentState } from "../../api/p2pDashboard";
import { Badge, type BadgeTone } from "../../components";
import {
  MY_CONTENT_STATE_LABELS,
  MY_CONTENT_STATE_TONES,
} from "./MyContentModel";
import {
  VIEWER_STATE_LABELS,
  VIEWER_STATE_TONES,
  type ViewerProductState,
} from "./InvitesModel";

type StatusPresentation = Readonly<{
  label: string;
  tone: BadgeTone;
}>;

export function p2pOwnerStatusPresentation(
  state: P2pOwnerContentState,
): StatusPresentation {
  return {
    label: MY_CONTENT_STATE_LABELS[state],
    tone: MY_CONTENT_STATE_TONES[state],
  };
}

export function p2pViewerStatusPresentation(
  state: ViewerProductState,
): StatusPresentation {
  return {
    label: VIEWER_STATE_LABELS[state],
    tone: VIEWER_STATE_TONES[state],
  };
}

type P2pStatusBadgeProps =
  | {
      surface: "owner";
      state: P2pOwnerContentState;
      testID?: string;
    }
  | {
      surface: "viewer";
      state: ViewerProductState;
      testID?: string;
    };

export function P2pStatusBadge(props: P2pStatusBadgeProps) {
  const presentation =
    props.surface === "owner"
      ? p2pOwnerStatusPresentation(props.state)
      : p2pViewerStatusPresentation(props.state);

  return (
    <Badge
      testID={props.testID}
      label={presentation.label}
      tone={presentation.tone}
    />
  );
}
