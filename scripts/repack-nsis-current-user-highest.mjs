import { copyFile, readFile, rm, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(await readFile(path.join(root, "package.json"), "utf8"));
const nsisDir = path.join(root, "src-tauri", "target", "release", "nsis", "x64");
const installerScript = path.join(nsisDir, "installer.nsi");
const nsisOutput = path.join(nsisDir, "nsis-output.exe");
const setupOutput = path.join(
  root,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "nsis",
  `AlertTimer_${packageJson.version}_x64-setup.exe`,
);

const localAppData = process.env.LOCALAPPDATA;
if (!localAppData) {
  throw new Error("LOCALAPPDATA is not set; cannot locate Tauri NSIS tools.");
}

const makensisCandidates = [
  path.join(localAppData, "tauri", "NSIS", "makensis.exe"),
  path.join(localAppData, "tauri", "NSIS", "Bin", "makensis.exe"),
];
const makensis = makensisCandidates.find((candidate) => existsSync(candidate));
if (!makensis) {
  throw new Error(`Could not find makensis.exe. Checked: ${makensisCandidates.join(", ")}`);
}

const original = await readFile(installerScript, "utf8");
if (!original.includes('!define INSTALLMODE "currentUser"')) {
  throw new Error("Expected currentUser NSIS install mode before repacking.");
}
if (!original.includes("RequestExecutionLevel user")) {
  throw new Error("Expected a user-level NSIS installer before repacking.");
}

const patched = original.replace("RequestExecutionLevel user", "RequestExecutionLevel highest");
await writeFile(installerScript, patched);
await rm(nsisOutput, { force: true });

const result = spawnSync(makensis, [installerScript], {
  cwd: nsisDir,
  encoding: "utf8",
  stdio: "pipe",
});

if (result.status !== 0) {
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  throw new Error(`makensis failed with exit code ${result.status}`);
}

if (!existsSync(nsisOutput)) {
  throw new Error(`Expected NSIS output was not created: ${nsisOutput}`);
}

await copyFile(nsisOutput, setupOutput);
console.log(`Repacked current-user NSIS installer with highest privilege: ${setupOutput}`);
