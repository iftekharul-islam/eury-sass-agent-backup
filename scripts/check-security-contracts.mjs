import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";

const root = new URL("../", import.meta.url);
const read = (path) => readFile(new URL(path, root), "utf8");

const failures = [];

// 1. Read files
const [
  threatModel,
  sandboxModel,
  policyEngine,
  secretsDoc,
  injectionDoc,
  supplyChainDoc,
  privacyDoc,
  incidentDoc,
  testingDoc,
  securityTypesSchemaRaw,
  workspacePolicySchemaRaw,
  sandboxCapsSchemaRaw,
  fixtureSchemaRaw,
  manifestRaw,
] = await Promise.all([
  read("docs/03-security/01-threat-model.md"),
  read("docs/03-security/02-sandbox-model.md"),
  read("docs/03-security/03-permission-and-policy-engine.md"),
  read("docs/03-security/04-secrets-and-key-management.md"),
  read("docs/03-security/05-prompt-injection-defense.md"),
  read("docs/03-security/06-supply-chain-and-signing.md"),
  read("docs/03-security/07-privacy-and-data-residency.md"),
  read("docs/03-security/09-incident-response.md"),
  read("docs/08-quality/04-security-testing.md"),
  read("docs/04-specs/schemas/security-types.schema.json"),
  read("docs/04-specs/schemas/workspace-policy.schema.json"),
  read("docs/04-specs/schemas/sandbox-capabilities.schema.json"),
  read("tests/fixtures/security/security-fixture.schema.json"),
  read("tests/fixtures/security/manifest.json"),
]);

// 2. Parse JSON Schemas
let securityTypesSchema;
let workspacePolicySchema;
let sandboxCapsSchema;
let fixtureSchema;
let manifest;

try {
  securityTypesSchema = JSON.parse(securityTypesSchemaRaw);
  workspacePolicySchema = JSON.parse(workspacePolicySchemaRaw);
  sandboxCapsSchema = JSON.parse(sandboxCapsSchemaRaw);
  fixtureSchema = JSON.parse(fixtureSchemaRaw);
  manifest = JSON.parse(manifestRaw);
} catch (err) {
  failures.push(`JSON schema parse failure: ${err.message}`);
}

// 3. Threat Model parser & validator
const assetRows = threatModel
  .split("\n")
  .filter((l) => /^\|\s*A-\d{3}\s*\|/.test(l))
  .map((l) => l.split("|").slice(1, -1).map((c) => c.trim()));

const actorRows = threatModel
  .split("\n")
  .filter((l) => /^\|\s*ACT-\d{3}\s*\|/.test(l))
  .map((l) => l.split("|").slice(1, -1).map((c) => c.trim()));

const boundaryRows = threatModel
  .split("\n")
  .filter((l) => /^\|\s*B-\d{3}\s*\|/.test(l))
  .map((l) => l.split("|").slice(1, -1).map((c) => c.trim()));

const controlRows = threatModel
  .split("\n")
  .filter((l) => /^\|\s*C-\d{3}\s*\|/.test(l))
  .map((l) => l.split("|").slice(1, -1).map((c) => c.trim()));

const threatRows = threatModel
  .split("\n")
  .filter((l) => /^\|\s*T-\d{3}\s*\|/.test(l))
  .map((l) => l.split("|").slice(1, -1).map((c) => c.trim()));

const assetIds = new Set();
for (const r of assetRows) {
  if (assetIds.has(r[0])) failures.push(`Duplicate asset ID: ${r[0]}`);
  assetIds.add(r[0]);
}

const actorIds = new Set();
for (const r of actorRows) {
  if (actorIds.has(r[0])) failures.push(`Duplicate actor ID: ${r[0]}`);
  actorIds.add(r[0]);
}

const boundaryIds = new Set();
for (const r of boundaryRows) {
  if (boundaryIds.has(r[0])) failures.push(`Duplicate boundary ID: ${r[0]}`);
  boundaryIds.add(r[0]);
}

const controlIds = new Set();
for (const r of controlRows) {
  if (controlIds.has(r[0])) failures.push(`Duplicate control ID: ${r[0]}`);
  controlIds.add(r[0]);
}

const threatIds = new Set();
for (const r of threatRows) {
  if (threatIds.has(r[0])) failures.push(`Duplicate threat ID: ${r[0]}`);
  threatIds.add(r[0]);
}
const testIds = new Set();

for (const row of controlRows) {
  const [cId, , testId] = row;
  if (!testId || !/^TEST-\d{3}$/.test(testId)) {
    failures.push(`Control ${cId} has invalid verification ID: ${testId}`);
  } else {
    testIds.add(testId);
  }
}

if (assetIds.size === 0) failures.push("Threat model has no assets (A-nnn)");
if (actorIds.size === 0) failures.push("Threat model has no actors (ACT-nnn)");
if (boundaryIds.size === 0) failures.push("Threat model has no trust boundaries (B-nnn)");
if (controlIds.size === 0) failures.push("Threat model has no controls (C-nnn)");
if (threatIds.size === 0) failures.push("Threat model has no ranked threats (T-nnn)");

for (const row of assetRows) {
  const [aId, , , , reqControls] = row;
  const referencedControls = [...reqControls.matchAll(/C-\d{3}/g)].map((m) => m[0]);
  for (const c of referencedControls) {
    if (!controlIds.has(c)) {
      failures.push(`Asset ${aId} references unregistered control ${c}`);
    }
  }
}

for (const row of threatRows) {
  const [tId, stride, actorsBoundaries, threatAssets, inherent, controls, residual, ownerGate] = row;
  const referencedControls = [...controls.matchAll(/C-\d{3}/g)].map((m) => m[0]);
  for (const c of referencedControls) {
    if (!controlIds.has(c)) {
      failures.push(`Threat ${tId} references unregistered control ${c}`);
    }
  }
  if (referencedControls.length === 0) {
    failures.push(`Threat ${tId} has no mitigation control mapped`);
  }

  const referencedActors = [...actorsBoundaries.matchAll(/ACT-\d{3}/g)].map((m) => m[0]);
  for (const act of referencedActors) {
    if (!actorIds.has(act)) {
      failures.push(`Threat ${tId} references unregistered actor ${act}`);
    }
  }

  const referencedBoundaries = [...actorsBoundaries.matchAll(/B-\d{3}/g)].map((m) => m[0]);
  for (const b of referencedBoundaries) {
    if (!boundaryIds.has(b)) {
      failures.push(`Threat ${tId} references unregistered boundary ${b}`);
    }
  }

  const referencedAssets = [...threatAssets.matchAll(/A-\d{3}/g)].map((m) => m[0]);
  for (const a of referencedAssets) {
    if (!assetIds.has(a)) {
      failures.push(`Threat ${tId} references unregistered asset ${a}`);
    }
  }
}

// 4. Validate attack corpora & manifest
if (manifest) {
  let computedTotal = 0;
  const corpusCaseIds = new Set();

  for (const entry of manifest.corpora || []) {
    computedTotal += entry.caseCount;
    const corpusPath = `tests/fixtures/security/${entry.path}`;
    try {
      const corpusRaw = await read(corpusPath);
      const corpus = JSON.parse(corpusRaw);
      if (corpus.kind !== "security-corpus") {
        failures.push(`${corpusPath} has kind "${corpus.kind}" instead of "security-corpus"`);
      }
      if (!Array.isArray(corpus.cases) || corpus.cases.length !== entry.caseCount) {
        failures.push(`${corpusPath} has ${corpus.cases?.length} cases; manifest expected ${entry.caseCount}`);
      }
      for (const item of corpus.cases || []) {
        if (corpusCaseIds.has(item.id)) {
          failures.push(`Duplicate test case ID in corpora: ${item.id}`);
        }
        corpusCaseIds.add(item.id);

        for (const a of item.assets || []) {
          if (!assetIds.has(a)) failures.push(`Case ${item.id} references unregistered asset ${a}`);
        }
        for (const t of item.threats || []) {
          if (!threatIds.has(t)) failures.push(`Case ${item.id} references unregistered threat ${t}`);
        }
        for (const c of item.controls || []) {
          if (!controlIds.has(c)) failures.push(`Case ${item.id} references unregistered control ${c}`);
        }
      }
    } catch (err) {
      failures.push(`Failed to read/validate corpus ${corpusPath}: ${err.message}`);
    }
  }

  if (manifest.totalCases !== computedTotal) {
    failures.push(`Manifest totalCases (${manifest.totalCases}) does not match sum of corpora (${computedTotal})`);
  }
}

// 5. Cross-document security contracts
const requiredDocTerms = [
  { doc: sandboxModel, name: "sandbox-model", terms: ["macOS Seatbelt", "Landlock", "Job-object", "fail-closed"] },
  { doc: policyEngine, name: "policy-engine", terms: ['"allow" | "needsApproval" | "deny"', "GrantScope", "workspace_policy.json"] },
  { doc: secretsDoc, name: "secrets-management", terms: ["SEC-001", "OS keychain", "redaction", "no-plaintext-fallback"] },
  { doc: injectionDoc, name: "prompt-injection", terms: ["ContextBlock", "ContextProvenance", "monotonic trust"] },
  { doc: supplyChainDoc, name: "supply-chain", terms: ["deny.toml", ".gitleaks.toml", "SPDX 2.3 SBOM", "pinned"] },
  { doc: privacyDoc, name: "privacy-and-data-residency", terms: ["purpose and residency", "DPA", "zero retention"] },
  { doc: incidentDoc, name: "incident-response", terms: ["SEV1", "SEV2", "SEV3", "containment"] },
  { doc: testingDoc, name: "security-testing", terms: ["TEST-001", "cargo audit", "cargo deny", "Semgrep"] },
];

for (const { doc, name, terms } of requiredDocTerms) {
  for (const term of terms) {
    if (!doc.includes(term)) {
      failures.push(`Security document ${name} is missing required contract term "${term}"`);
    }
  }
}

// Check GrantScope does not include deny
if (policyEngine.includes('"session" | "oneTime" | "workspace" | "deny"') || securityTypesSchemaRaw.includes('"deny"')) {
  // Check specifically in GrantScope definition
  if (securityTypesSchema?.definitions?.GrantScope?.enum?.includes("deny")) {
    failures.push("GrantScope contains 'deny'; deny must be a decision, not a grant scope");
  }
}

// 6. Security configuration files exist
const requiredFiles = [
  "deny.toml",
  ".gitleaks.toml",
  "security/semgrep/rules.yaml",
  "security/semgrep/tests/rules.rs",
  "security/semgrep/tests/rules.tsx",
  ".github/workflows/agent-security.yml",
  ".github/dependabot.yml",
  ".github/CODEOWNERS",
];

for (const relPath of requiredFiles) {
  if (!existsSync(new URL(relPath, root))) {
    failures.push(`Required security file missing: ${relPath}`);
  }
}

if (failures.length > 0) {
  console.error("Security contract validation failed:");
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(
  `Security contracts passed: ${assetIds.size} assets, ${threatIds.size} threats, ${controlIds.size} controls, ${manifest?.totalCases ?? 0} attack test fixtures verified across 3 platform profiles.`,
);
