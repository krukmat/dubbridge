import Constants from "expo-constants";

const ALLOWED_ENVS = ["local", "staging", "production"] as const;

type DubbridgeEnv = (typeof ALLOWED_ENVS)[number];

type RuntimeConfig = {
  dubbridgeEnv: DubbridgeEnv;
  gatewayBaseUrl: string;
};

type RuntimeConfigResult =
  | {
      ok: true;
      value: RuntimeConfig;
    }
  | {
      ok: false;
      message: string;
    };

type ExpoExtra = {
  dubbridgeEnv?: string | null;
  gatewayBaseUrl?: string | null;
};

function getExpoExtra(): ExpoExtra {
  return (Constants.expoConfig?.extra ?? {}) as ExpoExtra;
}

function isDubbridgeEnv(value: string): value is DubbridgeEnv {
  return (ALLOWED_ENVS as readonly string[]).includes(value);
}

export function readRuntimeConfig(): RuntimeConfigResult {
  const extra = getExpoExtra();
  const dubbridgeEnv = extra.dubbridgeEnv?.trim();
  const gatewayBaseUrl = extra.gatewayBaseUrl?.trim();

  if (!dubbridgeEnv) {
    return {
      ok: false,
      message:
        "Missing DUBBRIDGE_ENV. Expected one of: local, staging, production.",
    };
  }

  if (!isDubbridgeEnv(dubbridgeEnv)) {
    return {
      ok: false,
      message: `Invalid DUBBRIDGE_ENV '${dubbridgeEnv}'. Expected one of: ${ALLOWED_ENVS.join(
        ", ",
      )}.`,
    };
  }

  if (!gatewayBaseUrl) {
    return {
      ok: false,
      message:
        "Missing gateway base URL. Set EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL or DUBBRIDGE_GATEWAY_URL.",
    };
  }

  return {
    ok: true,
    value: {
      dubbridgeEnv,
      gatewayBaseUrl,
    },
  };
}
