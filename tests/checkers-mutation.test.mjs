import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, writeFile, rm, cp } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));

async function runCheck(scriptName, cwd) {
  try {
    const stdout = execFileSync("node", [path.join(cwd, "scripts", scriptName)], {
      cwd,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    return { ok: true, stdout };
  } catch (err) {
    return { ok: false, stdout: err.stdout, stderr: err.stderr, exitCode: err.status };
  }
}

async function withTempCopy(fn) {
  const tempDir = await mkdtemp(path.join(tmpdir(), "eury-test-"));
  try {
    await cp(root, tempDir, {
      recursive: true,
      filter: (src) => {
        const basename = path.basename(src);
        return basename !== "node_modules" && basename !== "target" && basename !== ".git";
      },
    });
    await fn(tempDir);
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
}

console.log("Running security and contract checker mutation tests...");

await withTempCopy(async (temp) => {
  // Test 1: Clean check must pass
  const cleanSecurity = await runCheck("check-security-contracts.mjs", temp);
  assert.equal(cleanSecurity.ok, true, "Clean workspace must pass check-security-contracts");

  const cleanDocs = await runCheck("check-docs.mjs", temp);
  assert.equal(cleanDocs.ok, true, "Clean workspace must pass check-docs");

  const cleanProduct = await runCheck("check-product-contracts.mjs", temp);
  assert.equal(cleanProduct.ok, true, "Clean workspace must pass check-product-contracts");

  // Test 2: Duplicate threat ID in threat model must fail
  const threatModelPath = path.join(temp, "docs/03-security/01-threat-model.md");
  const origThreatModel = await readFile(threatModelPath, "utf8");
  await writeFile(threatModelPath, origThreatModel.replace("| T-002 |", "| T-001 |"));
  const dupThreat = await runCheck("check-security-contracts.mjs", temp);
  assert.equal(dupThreat.ok, false, "Duplicate threat ID must fail");
  await writeFile(threatModelPath, origThreatModel);

  // Test 3: Unregistered control reference must fail
  await writeFile(threatModelPath, origThreatModel.replace("C-003, C-005", "C-999, C-005"));
  const badControl = await runCheck("check-security-contracts.mjs", temp);
  assert.equal(badControl.ok, false, "Unknown control ID must fail");
  await writeFile(threatModelPath, origThreatModel);

  // Test 4: Manifest total count mismatch must fail
  const manifestPath = path.join(temp, "tests/fixtures/security/manifest.json");
  const origManifest = await readFile(manifestPath, "utf8");
  await writeFile(manifestPath, origManifest.replace('"totalCases": 56', '"totalCases": 999'));
  const badManifest = await runCheck("check-security-contracts.mjs", temp);
  assert.equal(badManifest.ok, false, "Manifest count mismatch must fail");
  await writeFile(manifestPath, origManifest);

  // Test 5: Unknown asset in test corpus must fail
  const corpusPath = path.join(temp, "tests/fixtures/security/path-traversal-symlink.json");
  const origCorpus = await readFile(corpusPath, "utf8");
  await writeFile(corpusPath, origCorpus.replace('"A-001"', '"A-999"'));
  const badAsset = await runCheck("check-security-contracts.mjs", temp);
  assert.equal(badAsset.ok, false, "Unknown asset reference in corpus must fail");
  await writeFile(corpusPath, origCorpus);

  // Test 6: Duplicate test case ID across corpora must fail
  const corpus2Path = path.join(temp, "tests/fixtures/security/command-metacharacter-egress.json");
  const origCorpus2 = await readFile(corpus2Path, "utf8");
  await writeFile(corpus2Path, origCorpus2.replace('"TEST-211"', '"TEST-201"'));
  const dupCase = await runCheck("check-security-contracts.mjs", temp);
  assert.equal(dupCase.ok, false, "Duplicate case ID in corpora must fail");
  await writeFile(corpus2Path, origCorpus2);

  // Test 7: Missing required security file must fail
  const denyPath = path.join(temp, "deny.toml");
  await rm(denyPath);
  const missingFile = await runCheck("check-security-contracts.mjs", temp);
  assert.equal(missingFile.ok, false, "Missing deny.toml must fail");

  // Test 8: Roadmap phase drift must fail check-docs
  const phase0Path = path.join(temp, "docs/09-roadmap/phase-00.md");
  const origPhase0 = await readFile(phase0Path, "utf8");
  await writeFile(phase0Path, origPhase0 + "\n# Extra drifting line\n");
  const driftDocs = await runCheck("check-docs.mjs", temp);
  assert.equal(driftDocs.ok, false, "Roadmap drift must cause check-docs to fail");
  await writeFile(phase0Path, origPhase0);
});

console.log("All checker mutation tests passed successfully!");
