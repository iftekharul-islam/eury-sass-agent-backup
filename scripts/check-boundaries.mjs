import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const cratesRoot = path.join(root, "crates");

async function rustFiles(directory) {
  const entries = await readdir(directory);
  const files = await Promise.all(
    entries.map(async (entry) => {
      const target = path.join(directory, entry);
      return (await stat(target)).isDirectory() ? rustFiles(target) : [target];
    }),
  );
  return files.flat().filter((file) => file.endsWith(".rs"));
}

// Strips `//` line comments and `/* */` block comments so matches inside
// prose (e.g. a doc comment discussing why `.unwrap()` is avoided) don't
// register as violations. Deliberately simple — doesn't account for `//`
// inside a string literal — which is an acceptable false-negative risk for
// a lint gate, not a parser.
function stripComments(source) {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .split("\n")
    .map((line) => line.replace(/\/\/.*$/, ""))
    .join("\n");
}

const failures = [];
for (const file of await rustFiles(cratesRoot)) {
  const raw = await readFile(file, "utf8");
  const contents = stripComments(raw);
  const relative = path.relative(root, file);
  const isSandbox = relative.startsWith("crates/agent-sandbox/");
  // agent-store manages its own SQLite database file (open, quarantine-on-
  // corruption, migration backups) and JSON export/import of that database's
  // own content — always a fixed app-data-dir or user-chosen backup path,
  // never a workspace- or agent-supplied path. PathGuard exists to contain
  // agent/tool access to workspace content, which does not apply here, so
  // this crate is exempt from the same boundary as agent-sandbox.
  const isStore = relative.startsWith("crates/agent-store/");
  // Integration tests own their fixtures: a temp directory a test creates and
  // deletes is this repo's own scratch space, not workspace content reached
  // through an agent or tool, which is what PathGuard contains. Reads of
  // workspace content inside a test still go through PathGuard; only fixture
  // setup/teardown is exempt, and `std::process::Command` stays forbidden.
  const isCrateTest = /^crates\/[^/]+\/tests\//.test(relative);

  const forbidden = isCrateTest
    ? /\bstd::process::Command\b/
    : /\bstd::fs\b|\bstd::process::Command\b/;

  if (!isSandbox && !isStore && forbidden.test(contents)) {
    failures.push(`${relative} bypasses the agent-sandbox boundary`);
  }
  if (/\.(unwrap|expect)\s*\(/.test(contents)) {
    failures.push(`${relative} uses unwrap/expect`);
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Workspace boundary checks passed");
