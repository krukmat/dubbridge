import { Button } from "../components/Button";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { useAuth } from "../auth/AuthProvider";

export function LoginScreen() {
  const auth = useAuth();

  return (
    <Screen
      testID="login-screen"
      contentContainerStyle={{ justifyContent: "space-between" }}
    >
      <ScreenHeader
        title="DubBridge"
        copy="Sign in to access your media assets."
      />
      <Button
        label="Sign in"
        onPress={() => void auth.login()}
        fullWidth
      />
    </Screen>
  );
}
