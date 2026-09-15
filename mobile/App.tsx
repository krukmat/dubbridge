import { StatusBar } from "expo-status-bar";
import Constants from "expo-constants";
import {
  SafeAreaProvider,
  initialWindowMetrics,
} from "react-native-safe-area-context";

import { RootNavigator } from "./src/navigation/RootNavigator";
import { P2PDevelopmentHarness } from "./src/p2p/development/P2PDevelopmentHarness";
import { P5DeviceCertificationHarness } from "./src/p2p/development/P5DeviceCertificationHarness";
import { P2PProvider } from "./src/p2p/P2PProvider";
import { AuthProvider } from "./src/auth/AuthProvider";

const p2pDevelopmentHarnessEnabled =
  Constants.expoConfig?.extra?.p2pDevelopmentHarness === true;
const p5DeviceCertificationHarnessEnabled =
  Constants.expoConfig?.extra?.p5DeviceCertificationHarness === true;

export default function App() {
  return (
    <SafeAreaProvider initialMetrics={initialWindowMetrics}>
      <StatusBar style="dark" />
      <AuthProvider>
        <P2PProvider>
          <P2PDevelopmentHarness enabled={p2pDevelopmentHarnessEnabled} />
          <P5DeviceCertificationHarness enabled={p5DeviceCertificationHarnessEnabled} />
          <RootNavigator />
        </P2PProvider>
      </AuthProvider>
    </SafeAreaProvider>
  );
}
