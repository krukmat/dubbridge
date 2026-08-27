import { StatusBar } from "expo-status-bar";
import Constants from "expo-constants";
import {
  SafeAreaProvider,
  initialWindowMetrics,
} from "react-native-safe-area-context";

import { RootNavigator } from "./src/navigation/RootNavigator";
import { AndroidBareRuntimeProbe } from "./src/p2p/AndroidBareRuntimeProbe";
import { P2PProvider } from "./src/p2p/P2PProvider";
import { AuthProvider } from "./src/auth/AuthProvider";

const bareRuntimeProbeEnabled = Constants.expoConfig?.extra?.bareRuntimeProbe === true;

export default function App() {
  return (
    <SafeAreaProvider initialMetrics={initialWindowMetrics}>
      <StatusBar style="dark" />
      <AuthProvider>
        <P2PProvider>
          <AndroidBareRuntimeProbe enabled={bareRuntimeProbeEnabled} />
          <RootNavigator />
        </P2PProvider>
      </AuthProvider>
    </SafeAreaProvider>
  );
}
