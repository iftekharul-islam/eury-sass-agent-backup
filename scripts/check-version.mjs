import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const readJson = async (path) =>
  JSON.parse(await readFile(new URL(path, root), "utf8"));

const [version, workspacePackage, desktopPackage, tauriConfig] = await Promise.all([
  readJson("version.json"),
  readJson("package.json"),
  readJson("apps/desktop/package.json"),
  readJson("apps/desktop/src-tauri/tauri.conf.json"),
]);

const expected = version.version;
const values = {
  "root package": workspacePackage.version,
  "desktop package": desktopPackage.version,
  "Tauri config": tauriConfig.version,
};

const mismatches = Object.entries(values).filter(([, value]) => value !== expected);

if (mismatches.length > 0) {
  console.error(`Version source is ${expected}. Mismatches:`);
  for (const [name, value] of mismatches) {
    console.error(`- ${name}: ${value}`);
  }
  process.exit(1);
}

console.log(`Version consistency passed: ${expected}`);
