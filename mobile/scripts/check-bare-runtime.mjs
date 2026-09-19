import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const mobileRoot = path.resolve(__dirname, "..");
const bareKitRoot = path.join(
  mobileRoot,
  "node_modules",
  "react-native-bare-kit",
);

const packageJsonPath = path.join(bareKitRoot, "package.json");
const arm64BinaryPath = path.join(
  bareKitRoot,
  "android",
  "libs",
  "bare-kit",
  "jni",
  "arm64-v8a",
  "libbare-kit.so",
);

function tuple(version) {
  const match = /^(\d+)\.(\d+)\.(\d+)/.exec(version);
  if (!match) throw new Error(`invalid semantic version: ${version}`);
  return match.slice(1).map(Number);
}

function atLeast(actual, minimum) {
  const left = tuple(actual);
  const right = tuple(minimum);
  for (let index = 0; index < 3; index += 1) {
    if (left[index] > right[index]) return true;
    if (left[index] < right[index]) return false;
  }
  return true;
}

if (!fs.existsSync(packageJsonPath)) {
  throw new Error(
    "react-native-bare-kit is not installed; run npm ci before this check",
  );
}

const bareKitPackage = JSON.parse(
  fs.readFileSync(packageJsonPath, "utf8"),
);

if (!atLeast(bareKitPackage.version, "0.15.5")) {
  throw new Error(
    `react-native-bare-kit ${bareKitPackage.version} is too old; ` +
      "P5 requires >= 0.15.5",
  );
}

if (!fs.existsSync(arm64BinaryPath)) {
  throw new Error(
    "react-native-bare-kit arm64 native runtime is missing from the package",
  );
}

const binaryText = fs.readFileSync(arm64BinaryPath).toString("latin1");
const embedded = /builtin:bare-module@(\d+\.\d+\.\d+)/.exec(binaryText);

if (!embedded) {
  throw new Error(
    "could not determine embedded bare-module version from libbare-kit.so",
  );
}

if (!atLeast(embedded[1], "6.4.0")) {
  throw new Error(
    `embedded bare-module ${embedded[1]} is vulnerable to the known ` +
      "bundle evaluation-order crash; require >= 6.4.0",
  );
}

console.log(
  `Bare runtime OK: react-native-bare-kit@${bareKitPackage.version}, ` +
    `embedded bare-module@${embedded[1]}`,
);
