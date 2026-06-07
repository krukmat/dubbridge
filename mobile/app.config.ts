import type { ConfigContext, ExpoConfig } from "expo/config";

export default ({ config }: ConfigContext): ExpoConfig => ({
  ...config,
  name: "DubBridge",
  slug: "dubbridge-mobile",
  version: "1.0.0",
  orientation: "portrait",
  userInterfaceStyle: "light",
  scheme: "dubbridge",
  plugins: ["expo-status-bar"],
  extra: {
    dubbridgeEnv: process.env.DUBBRIDGE_ENV ?? null,
    gatewayBaseUrl:
      process.env.EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL ??
      process.env.DUBBRIDGE_GATEWAY_URL ??
      null,
  },
});
