import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

import ts from "typescript";

const mobileRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const runtimeRoot = path.join(mobileRoot, "src", "p2p", "runtime");
const outputPath = path.join(runtimeRoot, "worklet.bundle.js");
const sourcePaths = [
  path.join(runtimeRoot, "protocol.ts"),
  path.join(runtimeRoot, "transient-drive.ts"),
  path.join(runtimeRoot, "worklet.ts"),
];
const isCheck = process.argv.slice(2).includes("--check");

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function transpile(source, filename) {
  const result = ts.transpileModule(source, {
    compilerOptions: {
      esModuleInterop: true,
      module: ts.ModuleKind.CommonJS,
      removeComments: true,
      target: ts.ScriptTarget.ES2020,
    },
    fileName: filename,
    reportDiagnostics: true,
  });
  const errors = result.diagnostics?.filter((diagnostic) => diagnostic.category === ts.DiagnosticCategory.Error) ?? [];
  if (errors.length > 0) {
    throw new Error(
      ts.formatDiagnosticsWithColorAndContext(errors, {
        getCanonicalFileName: (name) => name,
        getCurrentDirectory: () => mobileRoot,
        getNewLine: () => "\n",
      }),
    );
  }
  return result.outputText;
}

function run(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: mobileRoot, stdio: "inherit" });
    child.once("error", reject);
    child.once("exit", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${command} exited with code ${String(code)}`));
    });
  });
}

async function buildBundle() {
  const temporaryRoot = await mkdtemp(path.join(mobileRoot, ".bare-pack-"));
  const temporaryRuntime = path.join(temporaryRoot, "src", "p2p", "runtime");
  const temporaryOutput = path.join(temporaryRoot, "worklet.bundle.js");

  try {
    await mkdir(temporaryRuntime, { recursive: true });
    for (const sourcePath of sourcePaths) {
      const source = await readFile(sourcePath, "utf8");
      const target = path.join(temporaryRuntime, `${path.basename(sourcePath, ".ts")}.js`);
      await writeFile(target, transpile(source, sourcePath), "utf8");
    }

    await run(process.execPath, [
      path.join(mobileRoot, "node_modules", "bare-pack", "bin.js"),
      "--base",
      temporaryRoot,
      "--host",
      "android-arm64",
      "--host",
      "android-x64",
      "--linked",
      "--out",
      temporaryOutput,
      path.join(temporaryRuntime, "worklet.js"),
    ]);

    return await readFile(temporaryOutput, "utf8");
  } finally {
    await rm(temporaryRoot, { force: true, recursive: true });
  }
}

const generated = await buildBundle();
const digest = sha256(generated);

if (isCheck) {
  const committed = await readFile(outputPath, "utf8").catch(() => null);
  if (committed !== generated) {
    throw new Error("Bare worklet bundle drift detected. Run npm run build:bare-worklet.");
  }
  console.log(`Bare worklet bundle is current: sha256=${digest}`);
} else {
  await writeFile(outputPath, generated, "utf8");
  console.log(`Bare worklet bundle written: sha256=${digest}`);
}
