import { ActivityIndicator, StyleSheet, Text, View } from "react-native";

import { color, space, type } from "../theme";
import { Button } from "./Button";
import { Panel } from "./Panel";

export type StateViewKind = "loading" | "empty" | "error";

export type StateViewProps = {
  kind: StateViewKind;
  title?: string;
  message?: string;
  /** Error-only: when provided, renders a retry button. */
  onRetry?: () => void;
  retryLabel?: string;
  testID?: string;
};

/**
 * Consistent loading / empty / error surface. Replaces the per-screen ad-hoc
 * states (rich panels in some screens, bare `<Text>Loading...</Text>` in others).
 */
export function StateView({
  kind,
  title,
  message,
  onRetry,
  retryLabel = "Retry",
  testID,
}: StateViewProps) {
  return (
    <Panel testID={testID}>
      {kind === "loading" ? (
        <ActivityIndicator size="small" color={color.primary} />
      ) : null}
      {title ? <Text style={styles.title}>{title}</Text> : null}
      {message ? <Text style={styles.message}>{message}</Text> : null}
      {kind === "error" && onRetry ? (
        <Button
          label={retryLabel}
          onPress={onRetry}
          variant="secondary"
          size="sm"
          testID={testID ? `${testID}-retry` : undefined}
        />
      ) : null}
    </Panel>
  );
}

const styles = StyleSheet.create({
  title: { ...type.heading, color: color.ink900 },
  message: { ...type.body, color: color.ink500 },
});
