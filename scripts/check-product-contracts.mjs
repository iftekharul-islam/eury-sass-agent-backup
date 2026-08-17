import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const read = (path) => readFile(new URL(path, root), "utf8");

const [catalog, modes, runtime, pricing, legacy, glossary, tools, policy, rbac, cloudApi] = await Promise.all([
  read("docs/01-product/02-feature-catalog.md"),
  read("docs/01-product/03-modes-and-workflows.md"),
  read("docs/04-specs/01-agent-runtime-spec.md"),
  read("docs/01-product/04-pricing-and-packaging.md"),
  read("docs/01-product/06-legacy-feature-inventory.md"),
  read("docs/00-overview/02-glossary.md"),
  read("docs/04-specs/02-tool-catalog-spec.md"),
  read("docs/06-enterprise/03-workspace-policies.md"),
  read("docs/06-enterprise/02-rbac-and-org-model.md"),
  read("docs/04-specs/06-cloud-api-contract.md"),
]);

const failures = [];
const featureRows = catalog
  .split("\n")
  .filter((line) => /^\|\s*F-\d{3}\s*\|/.test(line))
  .map((line) => line.split("|").slice(1, -1).map((cell) => cell.trim()));

const ids = new Set();
const allowedPriorities = new Set(["P0", "P1", "P2", "P3"]);
const allowedSizes = new Set(["XS", "S", "M", "L", "XL"]);
const allowedPersonas = new Set(["`P-SOLO`", "`P-LEAD`", "`P-SEC`", "`P-PLAT`"]);
const allowedLifecycle = new Set(["draft", "approved", "implemented", "verified", "deprecated"]);

for (const row of featureRows) {
  if (row.length !== 8) {
    failures.push(`Feature row has ${row.length} columns instead of 8: ${row.join(" | ")}`);
    continue;
  }
  const [id, , priority, size, phase, persona, entitlement, lifecycle] = row;
  if (ids.has(id)) failures.push(`Duplicate feature ID: ${id}`);
  ids.add(id);
  if (!allowedPriorities.has(priority)) failures.push(`${id} has invalid priority ${priority}`);
  if (!allowedSizes.has(size)) failures.push(`${id} has invalid size ${size}`);
  if (!allowedPersonas.has(persona)) failures.push(`${id} has invalid persona ${persona}`);
  if (!allowedLifecycle.has(lifecycle)) failures.push(`${id} has invalid lifecycle ${lifecycle}`);

  const phaseNumbers = [...phase.matchAll(/\d+/g)].map((match) => Number(match[0]));
  if (phaseNumbers.length === 0 || phaseNumbers.some((value) => value < 0 || value > 29)) {
    failures.push(`${id} has invalid phase ${phase}`);
  } else {
    const first = phaseNumbers[0];
    const last = phaseNumbers.at(-1);
    for (let value = first; value <= last; value += 1) {
      const phaseDoc = await read(`docs/09-roadmap/phase-${String(value).padStart(2, "0")}.md`);
      if (!phaseDoc.includes(`\`${id}\``)) {
        failures.push(`${id} is missing from phase-${String(value).padStart(2, "0")} traceability`);
      }
    }
  }

  const entitlementKey = entitlement.match(/^`([^`]+)`$/)?.[1];
  if (!entitlementKey) {
    failures.push(`${id} has malformed entitlement ${entitlement}`);
  } else if (!pricing.includes(`\`${entitlementKey}\``)) {
    failures.push(`${id} references unknown entitlement ${entitlementKey}`);
  }
}

if (featureRows.length === 0) failures.push("Feature catalog has no F-nnn rows");

const canonicalModes = ["chat", "ask", "plan", "agent", "build"];
for (const mode of canonicalModes) {
  if (!modes.includes(`| \`${mode}\` |`)) failures.push(`Mode spec is missing ${mode}`);
  if (!runtime.includes(`| \`${mode}\` |`)) failures.push(`Runtime spec is missing ${mode}`);
  if (!glossary.includes(mode[0].toUpperCase() + mode.slice(1))) failures.push(`Glossary is missing ${mode}`);
}

if (/\|\s*`review`\s*\|/i.test(modes) || /\|\s*`review`\s*\|/i.test(runtime)) {
  failures.push("Review is represented as a mode instead of a workflow");
}
const generateImageRow = tools.split("\n").find((line) => /^\|\s*`generate_image`\s*\|/.test(line));
if (generateImageRow?.includes("chat")) {
  failures.push("generate_image is incorrectly available in Chat mode");
}
if (/^\|\s*`(?:todo_write|plan_write)`\s*\|/m.test(tools)) {
  failures.push("Application-owned task/plan persistence is registered as a model tool");
}

const legacyRows = legacy
  .split("\n")
  .filter((line) => /^\|\s*L-\d{3}\s*\|/.test(line))
  .map((line) => line.split("|").slice(1, -1).map((cell) => cell.trim()));
const legacyIds = new Set();
const allowedDisposition = new Set(["preserve", "improve", "replace", "drop"]);

for (const row of legacyRows) {
  const [id, , disposition, target, phase, evidence, rationale] = row;
  if (legacyIds.has(id)) failures.push(`Duplicate legacy feature ID: ${id}`);
  legacyIds.add(id);
  if (!allowedDisposition.has(disposition)) failures.push(`${id} has invalid disposition ${disposition}`);
  if (!target || !phase || !evidence || !rationale) failures.push(`${id} has incomplete classification`);
}

if (legacyRows.length === 0) failures.push("Legacy inventory has no L-nnn rows");
if (!legacy.includes("Coverage: 100%")) failures.push("Legacy inventory does not declare 100% source-area coverage");

for (const plan of ["Free", "Starter", "Pro", "Business", "Enterprise"]) {
  if (!pricing.includes(`**${plan}**`)) failures.push(`Pricing is missing ${plan}`);
  if (!rbac.includes(`| ${plan} |`)) failures.push(`RBAC seat matrix is missing ${plan}`);
}
if (!pricing.includes("dailyManagedRuns") || !cloudApi.includes("dailyManagedRuns")) {
  failures.push("Canonical dailyManagedRuns quota is missing");
}
if (rbac.includes("72 h") || rbac.includes("72-hour")) failures.push("RBAC offline grace drifted from 24 hours");
if (/\bcostUsd\b|\bbudgetUsd\b/.test(cloudApi)) failures.push("Cloud API exposes floating-point USD fields");
if (!policy.includes('defaultDecision: Partial<Record<ToolClass, "allow" | "needsApproval" | "deny">>')) {
  failures.push("Workspace policy is missing canonical defaultDecision");
}
if (policy.includes("maxCostPerRunUsd?:") || policy.includes("maxCostPerDayUsd?:")) {
  failures.push("Workspace policy exposes floating-point cost caps");
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(
  `Product contracts passed: ${featureRows.length} features, ${canonicalModes.length} modes, ${legacyRows.length} legacy classifications`,
);
