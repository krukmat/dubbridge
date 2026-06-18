import { useState } from "react";
import {
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";

import { useAuth } from "../auth/AuthProvider";
import { Button } from "../components/Button";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { color, fieldStyle, space, type } from "../theme";

function getLoginErrorCopy(error: string | null): string | null {
  switch (error) {
    case "missing_runtime_config":
      return "This app is missing its gateway configuration.";
    case "network_error":
      return "We could not reach DubBridge. Try again.";
    case "login_failed":
      return "Invalid email or password.";
    default:
      return null;
  }
}

export function LoginScreen() {
  const auth = useAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const canSubmit =
    email.trim().length > 0 &&
    password.length > 0 &&
    !isSubmitting;

  async function handleSubmit(): Promise<void> {
    if (!canSubmit) {
      return;
    }

    setIsSubmitting(true);

    try {
      await auth.login(email, password);
    } finally {
      setIsSubmitting(false);
    }
  }

  const errorCopy = getLoginErrorCopy(auth.loginError);

  return (
    <Screen
      testID="login-screen"
      contentContainerStyle={{ justifyContent: "space-between" }}
    >
      <View style={styles.content}>
        <ScreenHeader
          kicker="DubBridge"
          title="Sign in"
          copy="Use your DubBridge email and password to access your workspace."
        />

        <View style={styles.form}>
          <View style={styles.fieldGroup}>
            <Text style={styles.label}>Email</Text>
            <TextInput
              testID="login-email-input"
              style={fieldStyle}
              value={email}
              onChangeText={setEmail}
              autoCapitalize="none"
              autoCorrect={false}
              keyboardType="email-address"
              textContentType="emailAddress"
              placeholder="you@company.com"
              placeholderTextColor={color.ink400}
            />
          </View>

          <View style={styles.fieldGroup}>
            <Text style={styles.label}>Password</Text>
            <TextInput
              testID="login-password-input"
              style={fieldStyle}
              value={password}
              onChangeText={setPassword}
              autoCapitalize="none"
              autoCorrect={false}
              secureTextEntry
              textContentType="password"
              placeholder="Enter your password"
              placeholderTextColor={color.ink400}
            />
          </View>

          {errorCopy ? (
            <Text testID="login-error-text" style={styles.error}>
              {errorCopy}
            </Text>
          ) : null}
        </View>
      </View>

      <Button
        testID="login-submit-button"
        label="Sign in"
        onPress={() => void handleSubmit()}
        loading={isSubmitting}
        disabled={!canSubmit}
        fullWidth
      />
    </Screen>
  );
}

const styles = StyleSheet.create({
  content: {
    gap: space.xxl,
  },
  form: {
    gap: space.lg,
  },
  fieldGroup: {
    gap: space.sm,
  },
  label: {
    ...type.label,
    color: color.ink700,
  },
  error: {
    ...type.meta,
    color: color.danger,
  },
});
