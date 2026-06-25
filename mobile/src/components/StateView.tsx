import { ActivityIndicator, StyleSheet, Text, View } from "react-native";

import { color, space, type } from "../theme";
import { Button } from "./Button";

export type StateViewKind = "loading" | "empty" | "error";
export type StateViewAppearance = "default" | "inverse";

export type StateViewProps = {
  kind: StateViewKind;
  title?: string;
  message?: string;
  /** Error-only: when provided, renders a retry button. */
  onRetry?: () => void;
  retryLabel?: string;
  testID?: string;
  appearance?: StateViewAppearance;
};

/**
 * Consistent loading / empty / error surface. Renders centered within its
 * parent — the parent container must grow (flexGrow:1) for centering to work
 * when nested inside a ScrollView contentContainer.
 */
export function StateView({
  kind,
  title,
  message,
  onRetry,
  retryLabel = "Retry",
  testID,
  appearance = "default",
}: StateViewProps) {
  const foreground =
    appearance === "inverse"
      ? { title: color.onPrimary, message: color.canvas }
      : { title: color.ink900, message: color.ink500 };

  return (
    <View style={styles.container} testID={testID}>
      {kind === "loading" ? (
        <ActivityIndicator size="small" color={color.primary} />
      ) : null}
      {title ? <Text style={[styles.title, { color: foreground.title }]}>{title}</Text> : null}
      {message ? <Text style={[styles.message, { color: foreground.message }]}>{message}</Text> : null}
      {kind === "error" && onRetry ? (
        <Button
          label={retryLabel}
          onPress={onRetry}
          variant="secondary"
          size="sm"
          testID={testID ? `${testID}-retry` : undefined}
        />
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
    gap: space.md,
    padding: space.xl,
  },
  title: { ...type.heading, textAlign: "center" },
  message: { ...type.body, textAlign: "center" },
});
