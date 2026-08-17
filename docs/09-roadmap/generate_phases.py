#!/usr/bin/env python3
"""Generate agent/docs/09-roadmap/phase-NN.md from the PHASES data below.

Usage:  python3 agent/docs/09-roadmap/generate_phases.py

Edit PHASES here rather than editing phase files by hand, so the roadmap stays
internally consistent. One-off prose tweaks in a phase file are acceptable but
will be overwritten on the next run.
"""

import argparse
import re
from pathlib import Path

OUT_DIR = Path(__file__).parent

FEATURE_IDS_BY_PHASE = {
    3: ["F-001", "F-002", "F-003", "F-004", "F-005", "F-007", "F-008"],
    4: ["F-008", "F-021", "F-025"],
    5: ["F-008", "F-025", "F-067"],
    6: ["F-008", "F-025", "F-029", "F-040", "F-041", "F-042", "F-043"],
    7: ["F-008", "F-025", "F-030"],
    8: ["F-008", "F-020", "F-028", "F-034", "F-035"],
    9: ["F-008", "F-068", "F-069"],
    10: ["F-010", "F-011", "F-012", "F-013", "F-014", "F-072", "F-073"],
    11: ["F-022", "F-031", "F-032", "F-033", "F-045", "F-048", "F-049"],
    12: ["F-044", "F-053"],
    13: ["F-050", "F-051", "F-052", "F-056"],
    14: ["F-047", "F-055"],
    15: ["F-060", "F-061"],
    16: ["F-062", "F-063"],
    17: ["F-023", "F-024", "F-026", "F-064"],
    18: ["F-065"],
    19: ["F-046", "F-066"],
    20: ["F-027"],
    22: ["F-054"],
    23: ["F-006", "F-070", "F-071"],
    24: ["F-074", "F-080", "F-081", "F-086"],
    25: ["F-082", "F-083", "F-084", "F-085"],
    27: ["F-075"],
}

TEMPLATE = """# Phase {n:02d} — {title}

Spec-Version: {spec_version}

**Track:** {track} · **Estimated size:** {size} · **Milestone:** {milestone}

## Goal

{goal}

## Why this phase exists here

{why}

## In scope

{scope}

## Feature IDs

{feature_ids}

## Out of scope

{out}

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
{deliverables}

## Key decisions and design notes

{decisions}

## Contracts touched

{contracts}

## Dependencies

{deps}

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
{risks}

## Test plan

| Layer | Coverage |
|---|---|
{tests}

## Metrics and targets

| Metric | Target |
|---|---|
{metrics}

## Exit criteria

{exit}

## Deferred from this phase

{defer}

## Related documents

{related}
"""


def bullets(items):
    return "\n".join("- " + i for i in items) if items else "- None"


def checks(items):
    lines = []
    for i in items:
        if isinstance(i, (tuple, list)):
            done, text = i
            mark = "x" if done else " "
            lines.append(f"- [{mark}] {text}")
        elif i.startswith("[x] ") or i.startswith("[ ] "):
            lines.append(f"- {i}")
        else:
            lines.append(f"- [ ] {i}")
    return "\n".join(lines)


def rows3(items):
    return "\n".join("| {} | {} | {} |".format(*i) for i in items)


def rows2(items):
    return "\n".join("| {} | {} |".format(*i) for i in items)


def render(p):
    return TEMPLATE.format(
        n=p["n"],
        title=p["title"],
        spec_version=p.get("spec_version", "1.1.0"),
        track=p["track"],
        size=p["size"],
        milestone=p["milestone"],
        goal=p["goal"],
        why=p["why"],
        scope=bullets(p["scope"]),
        feature_ids=", ".join(f"`{feature_id}`" for feature_id in FEATURE_IDS_BY_PHASE.get(p["n"], []))
        or "None — this phase establishes prerequisites or governance contracts.",
        out=bullets(p["out"]),
        deliverables=rows3(p["deliverables"]),
        decisions=bullets(p["decisions"]),
        contracts=bullets(p["contracts"]),
        deps=bullets(p["deps"]),
        risks=rows3(p["risks"]),
        tests=rows2(p["tests"]),
        metrics=rows2(p["metrics"]),
        exit=checks(p["exit"]),
        defer=bullets(p["defer"]),
        related=bullets(p.get("related", [
            "[Roadmap overview](00-roadmap-overview.md)",
            "[Definition of done](../08-quality/05-definition-of-done.md)",
            "[Risk register](risk-register.md)",
        ])),
    )


PHASES = []

# ---------------------------------------------------------------- Track A

PHASES.append(dict(
    n=0, title="Governance and Repo Foundation",
    spec_version="1.2.0",
    track="A — Foundations", size="1 week", milestone="M0 Docs",
    goal="Establish the `agent/` workspace, toolchain, CI skeleton, and the "
         "documentation and decision process that every later phase depends on.",
    why="Every convention that is cheap now becomes expensive later: crate layout, "
        "naming, lint rules, and the isolation boundary against the legacy `code` stack. "
        "Nothing here is user-visible, and skipping it is how the previous app accumulated "
        "its worst problems.",
    scope=[
        "`agent/` monorepo workspace: `apps/desktop`, `crates/*`, `docs/`, `bench/`, `eval/`, `tests/`",
        "Rust workspace with pinned toolchain (`rust-toolchain.toml`) and shared lint config",
        "pnpm workspace, pinned Node version, TypeScript strict mode",
        "Formatting and lint gates: `cargo fmt`, `cargo clippy -D warnings`, Biome/ESLint, `tsc --noEmit`",
        "`agent-ci.yml` skeleton running lint and unit jobs on three OSes",
        "ADR process, doc conventions, `Spec-Version` headers, changelog format",
        "Naming and migration map as the enforced source of truth for identifiers",
        "CODEOWNERS with security-sensitive paths requiring two reviewers",
        "The design mockup for all ten screens, so UI decisions are settled before any component exists",
    ],
    out=[
        "Any product feature",
        "Cersei integration",
        "Backend changes",
    ],
    deliverables=[
        ("D0.1", "Cargo workspace with the eight crates stubbed and building", "[ADR-0007](../02-architecture/adr/0007-rust-workspace-crate-split.md)"),
        ("D0.2", "Tauri 2 + React 19 + Tailwind 4 app that opens an empty window", "[desktop runtime](../02-architecture/02-desktop-runtime.md)"),
        ("D0.3", "`agent-ci.yml` with lint, typecheck, unit, debug-build jobs", "[CI/CD](../07-ops/02-ci-cd-pipelines.md)"),
        ("D0.4", "Lint rule set including no-`unwrap` and no-`std::fs`-outside-sandbox stubs", "[security testing](../08-quality/04-security-testing.md)"),
        ("D0.5", "ADR template and the first ten ADRs committed", "[doc conventions](../00-overview/04-doc-conventions.md)"),
        ("D0.6", "Naming and migration map published and referenced from the README", "[naming map](../00-overview/05-naming-and-migration-map.md)"),
        ("D0.7", "CODEOWNERS, PR template with the feature-done checklist", "[definition of done](../08-quality/05-definition-of-done.md)"),
        ("D0.8", "`version.json` and the version-consistency CI check", "[packaging](../07-ops/03-packaging-signing-notarization.md)"),
        ("D0.9", "Clickable design mockup covering all ten screens, offline, no build step", "[mockups](../../mockups/README.md), [visual language](../05-ui/00-visual-language.md)"),
        ("D0.10", "Token-parity check: the mockup's `:root` block matches the design system table", "[design system](../05-ui/01-design-system.md)"),
    ],
    decisions=[
        "Crate split is fixed now because moving module boundaries later invalidates every import and review habit.",
        "Toolchain versions are pinned, never floating — a silent compiler upgrade is an unreviewed change.",
        "`clippy -D warnings` from commit one; retrofitting lint cleanliness never happens.",
        "The `agent/` tree is self-contained: no build step reaches into `backend/`, `frontend/`, `ide/`, or `code-old/`.",
        "The mockup ships in Phase 0, not alongside the UI phases. Settling layout, tokens, and approval treatment on day one is what stops Phase 5 from re-litigating them, and a single offline HTML file is the cheapest artifact that survives review.",
    ],
    contracts=[
        "Repository layout and crate names",
        "CI job names (referenced by branch protection)",
        "Doc conventions and `Spec-Version` semantics",
    ],
    deps=["None — this is the entry point"],
    risks=[
        ("Premature crate boundaries", "Churn in later phases", "Boundaries follow ADR-0007, which was derived from the tool/policy/store separation the design already requires"),
        ("CI cost on three OSes", "Slow PRs", "Path filters, caching, and full matrix only on `main` and release PRs"),
        ("Toolchain drift between contributors", "Works-on-my-machine bugs", "Pinned toolchain files plus a `--verify-install` style doctor script"),
    ],
    tests=[
        ("Build", "Workspace builds clean on macOS, Windows, Linux"),
        ("Lint", "Formatting, clippy, ESLint, and typecheck gates fail on a seeded violation"),
        ("Meta", "Version-consistency check fails on a deliberate mismatch"),
        ("Meta", "Token-parity check fails when a design-system token and the mockup disagree"),
    ],
    metrics=[
        ("Cold CI run", "< 12 min"),
        ("Cached CI run", "< 5 min"),
        ("Empty-app cold start", "< 1 s"),
    ],
    exit=[
        "Workspace builds and lints clean on all three platforms",
        "CI runs on PRs with required checks configured in branch protection",
        "Empty Tauri window launches on all three platforms",
        "ADR process documented and the initial ADRs merged",
        "Naming and migration map merged and linked from the README",
        "PR template enforces the feature-done checklist",
        "Mockup opens offline in a browser and covers all ten wireframes, in both themes and all five accents",
    ],
    defer=[
        "Release signing (Phase 27)",
        "E2E harness (Phase 28)",
    ],
    related=[
        "[Roadmap overview](00-roadmap-overview.md)",
        "[Definition of done](../08-quality/05-definition-of-done.md)",
        "[Risk register](risk-register.md)",
        "[Mockups](../../mockups/README.md)",
        "[Visual language](../05-ui/00-visual-language.md)",
    ],
))

PHASES.append(dict(
    n=1, title="Product Definition",
    spec_version="1.2.0",
    track="A — Foundations", size="1 week", milestone="M0 Docs",
    goal="Lock what we are building and for whom, with a feature catalog that is "
         "prioritized, sized, and traceable to a phase.",
    why="The deprecated app grew by accretion. Writing the catalog before the code means "
        "every later phase can point at a numbered feature, and scope arguments are settled "
        "once rather than per sprint.",
    scope=[
        "Personas, jobs to be done, and the deprecated app's feature inventory",
        "Feature catalog with priority (P0–P3), size, owning phase, persona, entitlement, and lifecycle",
        "Mode definitions: chat, agent, plan, ask, build",
        "Pricing and packaging tiers mapped to entitlements",
        "Explicit non-goals",
        "Competitive analysis with the specific behaviors we intend to beat",
    ],
    out=[
        "Marketing site copy",
        "Final visual design",
        "Any implementation",
    ],
    deliverables=[
        ("D1.1", "Personas and jobs document", "[personas](../01-product/01-personas-and-jobs.md)"),
        ("D1.2", "Feature catalog, every row mapped to a phase", "[feature catalog](../01-product/02-feature-catalog.md)"),
        ("D1.3", "Mode semantics: default permissions and allowed tools per mode", "[modes](../01-product/03-modes-and-workflows.md)"),
        ("D1.4", "Pricing tiers mapped to quotas, gateway access, and policy features", "[pricing](../01-product/04-pricing-and-packaging.md)"),
        ("D1.5", "Non-goals, signed off by product and engineering", "[non-goals](../01-product/05-non-goals.md)"),
        ("D1.6", "Preserve/avoid list distilled from `code-old`", "[competitive landscape](../00-overview/03-competitive-landscape.md)"),
        ("D1.7", "Authoritative `code-old` feature inventory with preserve/improve/replace/drop decisions", "[legacy inventory](../01-product/06-legacy-feature-inventory.md)"),
        ("D1.8", "Automated product-contract validation for features, modes, entitlements, and legacy coverage", "[CI/CD](../07-ops/02-ci-cd-pipelines.md)"),
    ],
    decisions=[
        "Modes are permission profiles, not prompt presets — this is what makes `ask` safe by construction.",
        "The catalog is the only place feature scope lives; phase files reference it rather than redefining it.",
        "Anything from the old app that is not in the catalog is intentionally dropped, and the reason is recorded.",
    ],
    contracts=[
        "Feature IDs (F-nnn) referenced by phase files and PRs",
        "Mode names used by the runtime, policy engine, and UI",
        "Stable entitlement and quota names used by the cloud gateway",
        "Legacy feature IDs (`L-nnn`) and disposition vocabulary",
    ],
    deps=["Phase 0 (doc conventions)"],
    risks=[
        ("Scope creep during implementation", "Slipped milestones", "Feature IDs plus a phase mapping; new features enter the catalog with a phase, or they wait"),
        ("Modes under-specified", "Inconsistent permissions", "Each mode ships with an explicit default tool set asserted in tests"),
        ("Pricing assumptions wrong", "Rework in Phase 25", "Entitlement names are decoupled from prices; tiers can be re-priced without code change"),
    ],
    tests=[
        ("Review", "Every catalog row has a priority, size, and phase"),
        ("Traceability", "Every mode named in the catalog exists in the mode spec"),
        ("Product contract", "No duplicate feature IDs, invalid phases, unknown entitlements, or incomplete legacy rows"),
    ],
    metrics=[
        ("Catalog coverage of `code-old` features", "100% classified as preserve / improve / replace / drop"),
        ("Features without an owning phase", "0"),
    ],
    exit=[
        "[x] Feature catalog complete and mapped to phases",
        "[x] Mode semantics defined with default permissions per mode",
        "[x] Pricing tiers mapped to entitlements",
        "[x] Non-goals signed off",
        "[x] Every `code-old` feature classified",
        "[x] Automated product-contract check passes",
    ],
    defer=[
        "Marketing positioning (Phase 29)",
    ],
))

PHASES.append(dict(
    n=2, title="Security Foundation",
    spec_version="1.2.0",
    track="A — Foundations", size="1–2 weeks", milestone="M0 Docs",
    goal="Produce the threat model, sandbox design, policy model, and secret handling "
         "rules that constrain every later phase — before any tool can touch a filesystem.",
    why="Retrofitting containment onto a working agent is how agents ship with escapes. "
        "The sandbox and policy design must exist before the tool layer, or the tool layer "
        "will be written against the wrong assumptions.",
    scope=[
        "Threat model: assets (A-001..A-009), actors (ACT-001..ACT-007), boundaries (B-001..B-006), controls (C-001..C-016), ranked threats (T-001..T-016)",
        "Layered sandbox design for macOS (Seatbelt), Linux (Landlock+seccomp), Windows (restricted token+Job object) with fail-closed degradation",
        "Canonical machine-readable schemas: `security-types.schema.json`, `workspace-policy.schema.json`, `sandbox-capabilities.schema.json`",
        "Permission model: tool classes, grant scopes, deny-by-default, fail-closed rules, and monotonic trust",
        "Secret handling: canonical inventory (SEC-001..SEC-010), OS keychain, no-plaintext-fallback, and redaction",
        "Prompt injection defense strategy: `ContextBlock`, `ContextProvenance`, and the untrusted-content model",
        "Supply chain requirements: pinned deps, SPDX 2.3 SBOM, signing, `deny.toml`, `.gitleaks.toml`, `agent-security.yml`, Dependabot, CODEOWNERS",
        "Executable attack corpora (56 cases) in `agent/tests/fixtures/security/` with schema and manifest",
        "Semgrep security rules and positive/negative test fixtures",
        "Security checklists in PR template, Definition of Done, and release management",
    ],
    out=[
        "Implementation of the sandbox (Phase 5)",
        "Policy engine code (Phase 7)",
        "Penetration testing (Phase 28)",
    ],
    deliverables=[
        ("D2.1", "Threat model with ranked risks and named mitigations", "[threat model](../03-security/01-threat-model.md)"),
        ("D2.2", "Sandbox design per platform, including documented degradation", "[sandbox model](../03-security/02-sandbox-model.md)"),
        ("D2.3", "Permission and policy engine design with merge semantics and JSON schemas", "[policy engine](../03-security/03-permission-and-policy-engine.md)"),
        ("D2.4", "Secrets and key management rules, inventory, no-plaintext-fallback, and redaction", "[secrets](../03-security/04-secrets-and-key-management.md)"),
        ("D2.5", "Prompt injection defense plan and the initial attack corpus", "[injection defense](../03-security/05-prompt-injection-defense.md)"),
        ("D2.6", "Supply chain controls wired into CI (`cargo audit`, `cargo deny`, `pnpm audit`, gitleaks, Semgrep)", "[supply chain](../03-security/06-supply-chain-and-signing.md)"),
        ("D2.7", "Custom Semgrep rules and positive/negative test fixtures", "[security testing](../08-quality/04-security-testing.md)"),
        ("D2.8", "Attack test corpora and manifest under `tests/fixtures/security/`", "[security testing](../08-quality/04-security-testing.md)"),
        ("D2.9", "Automated security-contract check script (`scripts/check-security-contracts.mjs`)", "[CI/CD](../07-ops/02-ci-cd-pipelines.md)"),
        ("D2.10", "Per-PR, definition-of-done, and release security checklists", "[definition of done](../08-quality/05-definition-of-done.md)"),
    ],
    decisions=[
        "Deny by default for write, execute, and network tools ([ADR-0006](../02-architecture/adr/0006-deny-by-default-permissions.md)); no dev-mode exception that could ship.",
        "Defense in depth: policy check, path/command guard, and OS sandbox are three independent layers, and no layer is trusted alone.",
        "Fail closed everywhere: missing policy, unreadable keychain, or unavailable sandbox disables privileged tools rather than proceeding.",
        "All model-adjacent content (file contents, tool output, web results) is untrusted data, never instructions.",
        "The legacy `CODE_API_TOKEN`-unset-means-open pattern is explicitly named as a prohibited anti-pattern.",
        "GrantScope contains only `session`, `oneTime`, and `workspace`; `deny` is strictly a policy decision.",
        "Child-process egress defaults off (`networkDuringExecute: false`).",
    ],
    contracts=[
        "Tool classes and grant scopes used by policy and UI",
        "Untrusted-content marking convention used by prompt assembly",
        "Security checklist items referenced by CI and reviews",
        "Security JSON schemas (`security-types`, `workspace-policy`, `sandbox-capabilities`)",
        "Attack fixture schema (`security-fixture.schema.json`) and manifest",
    ],
    deps=["Phase 0 (CI to host the security jobs)"],
    risks=[
        ("Platform sandbox gaps", "Weaker containment on some OS", "Documented degradation, path/command guard always active, capability reported in the UI"),
        ("Over-restrictive defaults", "Product feels unusable", "Grant scopes and mode profiles tuned in Phase 7 with real usage, never by weakening the default"),
        ("Injection defense treated as prompt engineering", "False confidence", "Defense is structural (privilege separation and approvals), with the prompt layer as a secondary measure"),
    ],
    tests=[
        ("Static", "Semgrep rules fire on seeded violations; `rules.rs` and `rules.tsx` test fixtures"),
        ("Supply chain", "CI fails on an introduced vulnerable dependency via `cargo audit` / `deny`"),
        ("Corpus", "56 traversal, command, injection, secrets, SSRF, MCP, and policy fixtures committed and verified"),
        ("Security contract", "`pnpm security:check` validates asset/threat/control IDs, JSON schemas, and doc consistency"),
    ],
    metrics=[
        ("Threat model coverage", "Every asset and Critical risk has at least one named mitigation"),
        ("Security CI jobs", "Running on every PR and weekly on main"),
        ("Attack fixture coverage", "56 machine-readable test cases across 7 categories"),
    ],
    exit=[
        "[x] Threat model reviewed and signed off with stable asset, threat, and control IDs",
        "[x] Sandbox design covers all three platforms with explicit fail-closed degradation",
        "[x] Permission model documented with canonical schemas and fail-closed semantics",
        "[x] Security CI workflows, `deny.toml`, `.gitleaks.toml`, Semgrep rules, and CODEOWNERS configured",
        "[x] 56 attack test fixtures committed under `tests/fixtures/security/`",
        "[x] Security checklists in PR template, definition of done, and release management",
        "[x] Automated security-contract check passes (`pnpm security:check`)",
    ],
    defer=[
        "Sandbox implementation (Phase 5)",
        "Policy engine implementation (Phase 7)",
        "External pentest (Phase 28)",
    ],
))

# ---------------------------------------------------------------- Track B

PHASES.append(dict(
    n=3, title="Desktop Shell",
    track="B — Core runtime", size="2 weeks", milestone="—",
    goal="A production-quality application shell: window management, navigation, theming, "
         "settings, and the IPC plumbing that every later feature rides on.",
    why="The shell defines the performance ceiling and the IPC discipline. Building it before "
        "the agent means streaming, approvals, and diffs land in a structure that already "
        "handles focus, persistence, and window lifecycle correctly.",
    scope=[
        "Tauri window configuration, custom titlebar, single-instance enforcement, multi-window",
        "Persistent Home/Code top-level switch with independent navigation and draft state",
        "Conversation sidebar, center pane, and collapsible live context panel",
        "Design tokens, light/dark themes, five accents, density modes",
        "Settings surface with persisted preferences (file-backed until Phase 9)",
        "Command palette with the command registry and `when`-clause evaluation",
        "IPC command and event scaffolding with typed contracts and golden fixtures",
        "Error boundary, toast system, empty/loading states, deep-link handler",
    ],
    out=[
        "Agent runs",
        "Editor and terminal",
        "SQLite persistence",
    ],
    deliverables=[
        ("D3.1", "Token system and theme switching with no re-render", "[design system](../05-ui/01-design-system.md)"),
        ("D3.1a", "Home area and persistent Home/Code switch with independent state restoration", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D3.2", "App shell: sidebar, center pane, collapsible context panel, status bar", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D3.3", "Window state persistence per display; single-instance focus behavior", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D3.4", "Command registry, palette with prefixes, rebinding UI", "[keyboard](../05-ui/08-keyboard-and-command-palette.md)"),
        ("D3.5", "Typed IPC command layer with validation and golden fixtures", "[IPC spec](../04-specs/04-ipc-command-spec.md)"),
        ("D3.6", "Event channel scaffolding with a frame-batched consumer hook", "[event protocol](../04-specs/03-event-protocol-spec.md)"),
        ("D3.7", "Component library: buttons, inputs, cards, dialogs, panes, toasts, skeletons", "[design system](../05-ui/01-design-system.md)"),
        ("D3.8", "`eury-agent://` deep-link handler that never auto-sends a prompt", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D3.9", "Strict CSP with no remote script, font, or frame origins", "[threat model](../03-security/01-threat-model.md)"),
    ],
    decisions=[
        "Theme and accent switch via CSS variables and data attributes, not React state, so switching costs no re-render.",
        "The UI never touches the filesystem or network directly; everything is an IPC command ([ADR-0008](../02-architecture/adr/0008-event-protocol-over-tauri-channels.md)).",
        "Every command lives in one registry with a `when` clause, so disabled states are explainable rather than missing.",
        "Fonts are bundled; CSP forbids remote origins from day one so no feature can quietly add one.",
    ],
    contracts=[
        "IPC command envelope and error shape",
        "Event channel envelope",
        "Command registry interface",
        "Design token names",
    ],
    deps=["Phase 0 (scaffold)", "Phase 1 (surfaces to build)", "Phase 2 (CSP and IPC discipline)"],
    risks=[
        ("WebView differences across platforms", "Visual and behavioral bugs", "Three-OS visual smoke in CI; avoid bleeding-edge CSS; feature-detect"),
        ("IPC contract churn", "Rework in later phases", "Golden fixtures make changes visible; envelope is versioned"),
        ("Shell performance regressions", "Misses the latency budget", "Cold-start and IPC benchmarks in CI from this phase forward"),
    ],
    tests=[
        ("Unit", "Component behavior, command registry `when` evaluation, keybinding conflicts"),
        ("Contract", "IPC and event golden fixtures with TypeScript type parity"),
        ("Integration", "Window lifecycle, single instance, deep link, state persistence"),
        ("A11y", "`jest-axe` clean; keyboard-only navigation of every surface"),
        ("Visual", "Theme × accent × density snapshots on three platforms"),
    ],
    metrics=[
        ("Cold start to interactive shell", "< 400 ms p95"),
        ("Cold start to fully ready", "< 2 s p95"),
        ("IPC round-trip", "< 1 ms p95"),
        ("Palette first results", "< 50 ms"),
    ],
    exit=[
        "Shell launches and navigates on macOS, Windows, and Linux",
        "Themes, accents, and density switch without flicker or re-render",
        "Command palette works with all prefixes and rebinding",
        "IPC and event golden fixtures committed and enforced in CI",
        "Window and settings state survive restart",
        "CSP verified: no remote script, font, or frame loads",
        "Cold-start and IPC benchmarks meet targets",
    ],
    defer=[
        "SQLite-backed settings (Phase 9)",
        "Editor and terminal panes (Phases 12–13)",
    ],
))

PHASES.append(dict(
    n=4, title="Agent Core",
    spec_version="1.2.0",
    track="B — Core runtime", size="2–3 weeks", milestone="—",
    goal="Run the agent loop behind the `AgentEngine` trait and stream a real model response "
         "end to end, from the Rust core to the UI, with a stub provider for determinism.",
    why="This is the phase that validates the central architectural bet ([ADR-0001](../02-architecture/adr/0001-embed-cersei-in-desktop.md)). "
        "If the abstraction or the streaming path is wrong, everything after it is built on sand.\n\n"
        "**Architecture as shipped: managed gateway, not embedded Cersei.** This supersedes the original "
        "\"Cersei embedded, no network hop\" framing. The engine wired into the desktop app is "
        "`AgentLoopEngine`, which reaches models through the managed gateway (`/agent/v1/chat/stream`) "
        "rather than running Cersei in-process; `CerseiAdapter` is retained as scaffolding and is **not** "
        "on the live path. The reason is enforcement: org control, cost caps, usage metering, and "
        "model-policy filtering (Phase 11) cannot be enforced from a client the user controls. "
        "The agent loop, tool dispatch, policy, sandboxing, and all filesystem access still run locally — "
        "the hop is on model inference only, and Phase 4's latency targets account for it. "
        "The `AgentEngine` trait boundary is what makes this a one-crate substitution, and it keeps an "
        "embedded/BYOK engine viable later. Tool calls come from the gateway's typed `tool_call` NDJSON "
        "events and are assembled by `ToolCallAccumulator` — never by scanning assistant text for "
        "` ```tool_call ` fences, which would execute prose that merely resembles a tool call.",
    scope=[
        "`AgentEngine` trait: run lifecycle, streaming, abort, hook points",
        "Gateway-backed engine implementing the trait, isolated in one crate",
        "Run manager: run ids, concurrency limits, cancellation, cleanup",
        "Event mapping from engine events to the wire protocol",
        "Structured `tool_call` event assembly (no parsing of assistant prose)",
        "Provider abstraction with a deterministic stub provider plus the managed gateway",
        "Prompt assembly v1: system prompt, mode framing, history, untrusted-content marking",
        "Token counting, context-window accounting, and a cost estimator",
        "Structured error mapping into the `EURY_*` taxonomy",
    ],
    out=[
        "Tools (Phase 6)",
        "Persistence (Phase 9)",
        "Retrieval (Phase 15)",
    ],
    deliverables=[
        ("D4.1", "`AgentEngine` trait with documented invariants", "[engine abstraction](../02-architecture/04-agent-engine-abstraction.md)"),
        ("D4.2", "Gateway-backed engine behind the trait, no provider types leaking outward", "[ADR-0003](../02-architecture/adr/0003-agent-engine-trait-boundary.md)"),
        ("D4.3", "Run manager with abort within 250 ms and guaranteed resource cleanup", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D4.4", "Event stream: `meta`, `delta`, `reasoning`, `usage`, `done`, `error`", "[event protocol](../04-specs/03-event-protocol-spec.md)"),
        ("D4.5", "Stub provider with scripted responses and tool calls", "[test strategy](../08-quality/01-test-strategy.md)"),
        ("D4.6", "BYOK provider client with keychain-sourced credentials", "[secrets](../03-security/04-secrets-and-key-management.md)"),
        ("D4.7", "Prompt assembly with explicit untrusted-content regions", "[injection defense](../03-security/05-prompt-injection-defense.md)"),
        ("D4.8", "Token and cost accounting with a versioned price table", "[latency budget](../02-architecture/05-latency-budget.md)"),
    ],
    decisions=[
        "The agent loop runs locally; model inference goes through the managed gateway. Tool execution, policy, and file access never leave the machine.",
        "The trait boundary exists so an engine or provider change is a one-crate change; product code never imports engine-specific types.",
        "Tool calls come from typed `tool_call` stream events, never from parsing assistant prose.",
        "The stub provider is a first-class deliverable — deterministic agent tests are impossible without it.",
        "Aborts are cooperative but bounded: the UI reflects cancellation within 250 ms even if unwinding takes longer.",
    ],
    contracts=[
        "`AgentEngine` trait",
        "Run event stream variants",
        "Provider interface",
        "Error taxonomy mapping for engine and provider failures",
    ],
    deps=["Phase 3 (shell, IPC, event plumbing)", "Phase 2 (untrusted-content rules)"],
    risks=[
        ("Cersei API gaps or instability", "Blocked or forked", "Trait boundary plus an internal fallback loop kept behind the same trait; vendor pinned by exact version"),
        ("Streaming backpressure", "UI stalls", "Frame-batched consumption with documented drop priorities; load test with a synthetic fast stream"),
        ("Prompt assembly sprawl", "Unpredictable behavior", "Assembly is a pure function over typed inputs with golden-output tests"),
        ("Token accounting drift", "Wrong cost and context decisions", "Per-provider tokenizer tests against recorded responses"),
    ],
    tests=[
        ("Unit", "Run lifecycle, abort, concurrency limits, event mapping"),
        ("Integration", "Full stream with the stub provider; recorded-cassette replay for real providers"),
        ("Contract", "Event golden fixtures for every variant"),
        ("Load", "Synthetic 200-events/second stream stays within the frame budget"),
        ("Negative", "Provider 429/500/timeout/malformed-chunk handling"),
    ],
    metrics=[
        ("Time to first token (managed gateway, incl. network hop)", "< 800 ms p95"),
        ("Engine overhead per turn (excl. model)", "< 20 ms p95"),
        ("Abort acknowledged in UI", "< 250 ms p95"),
        ("Frame rate while streaming", "≥ 55 fps"),
    ],
    exit=[
        "A prompt streams a response end to end with the stub and one real provider",
        "Abort cancels cleanly with no orphaned tasks or leaked handles",
        "No engine-specific type appears outside the adapter crate (enforced by lint)",
        "Event golden fixtures cover every variant",
        "Latency targets met on reference hardware",
        "Provider failures surface as taxonomy codes, never raw strings",
        "`PromptAssembler` (D4.7) is invoked by the live path — it is currently dead code",
    ],
    defer=[
        "Tool execution (Phase 6)",
        "Retrieval-based context (Phase 15)",
        "Sub-agents (Phase 20)",
    ],
))

PHASES.append(dict(
    n=5, title="Workspace and Sandbox",
    track="B — Core runtime", size="2–3 weeks", milestone="—",
    goal="Implement the containment layer: workspace roots, trust states, path guard, "
         "command guard, and OS-level sandboxing — before any tool exists to use it.",
    why="Tools must be born inside a sandbox. Writing the guard after the tools would mean "
        "auditing every call site instead of having one enforced chokepoint.",
    scope=[
        "Workspace model: root registration, trust states, git remote detection, availability",
        "Path guard: canonicalization, symlink resolution, root containment, deny globs",
        "Open-then-verify semantics to close TOCTOU windows",
        "Command guard: argv parsing, allow/deny pattern matching, shell-metacharacter handling",
        "OS sandbox: Seatbelt profile (macOS), Landlock ruleset (Linux), Job object + restricted token (Windows)",
        "Process supervision: timeouts, output caps, process-group kill, orphan reaping",
        "Sandbox capability reporting and documented degradation",
        "Fuzz targets for the path and command parsers",
    ],
    out=[
        "The tool catalog itself (Phase 6)",
        "Approval UI (Phase 7)",
    ],
    deliverables=[
        ("D5.1", "Workspace registry with trust states and the trust prompt flow", "[approval and trust UX](../05-ui/05-approval-and-trust-ux.md)"),
        ("D5.2", "Path guard as the only filesystem entry point in the codebase", "[sandbox model](../03-security/02-sandbox-model.md)"),
        ("D5.3", "Command guard with normalized-shape matching", "[sandbox model](../03-security/02-sandbox-model.md)"),
        ("D5.4", "Per-platform OS sandbox with capability probing at startup", "[sandbox model](../03-security/02-sandbox-model.md)"),
        ("D5.5", "Process supervisor: timeout, output ring buffer, group kill", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D5.6", "Traversal and command corpora wired into a blocking CI suite", "[security testing](../08-quality/04-security-testing.md)"),
        ("D5.7", "Fuzz targets for path normalization and command parsing", "[security testing](../08-quality/04-security-testing.md)"),
        ("D5.8", "Lint rules banning `std::fs` and `Command::new` outside this crate", "[security testing](../08-quality/04-security-testing.md)"),
    ],
    decisions=[
        "One chokepoint: all filesystem and process access goes through `agent-sandbox`, enforced by lint rather than convention.",
        "Open-then-verify, not verify-then-open, because a path can change between the two.",
        "Untrusted workspaces are read-only with no execute, no network, and no workspace config honored.",
        "The OS sandbox is an additional layer, never a replacement for the guards — Landlock may be unavailable, and the guards may not be.",
        "Command matching is on normalized argv shape, so a grant cannot be widened by string tricks.",
    ],
    contracts=[
        "Path guard API and its error codes",
        "Command guard API and match semantics",
        "Sandbox capability report shown in the UI",
    ],
    deps=["Phase 2 (design)", "Phase 3 (settings and UI surfaces)"],
    risks=[
        ("Windows containment weaker than Unix", "Uneven security posture", "Job object plus restricted token plus guards; capability honestly reported; documented in the threat model"),
        ("Landlock unavailable on older kernels", "Degraded Linux containment", "Detect and report; guards still enforce; policy may require the OS layer for regulated orgs"),
        ("Guard false positives", "Legitimate work blocked", "Corpus-driven tuning plus clear error messages naming the rule that fired"),
        ("Symlink and case-insensitivity edge cases", "Escape", "Dedicated corpus, fuzzing, and platform-specific tests"),
    ],
    tests=[
        ("Unit", "Canonicalization, glob matching, argv parsing"),
        ("Security", "Full traversal and command corpora on all three platforms"),
        ("Integration", "Real temp filesystems, real symlinks, real processes"),
        ("Fuzz", "Path and command parsers, persisted corpora, no crashes"),
        ("Platform", "OS sandbox verifiably blocks a probe tool's forbidden syscall"),
    ],
    metrics=[
        ("Path guard overhead", "< 1 ms per call"),
        ("Command guard overhead", "< 1 ms per call"),
        ("Escape suite", "0 escapes on all platforms"),
        ("Orphaned processes after abort", "0"),
    ],
    exit=[
        "Escape suite passes on macOS, Windows, and Linux with zero escapes",
        "Lint proves no filesystem or process access exists outside the sandbox crate",
        "OS sandbox verified active, with degradation reported when unavailable",
        "Untrusted workspace mode enforces read-only",
        "Fuzz targets run in CI with committed corpora",
        "Abort leaves no orphaned processes",
    ],
    defer=[
        "Per-tool policy decisions (Phase 7)",
        "Egress control during execute (Phase 7 policy, enforced here)",
    ],
))

PHASES.append(dict(
    n=6, title="Tool Layer v1",
    track="B — Core runtime", size="2–3 weeks", milestone="—",
    goal="Ship the core tool catalog on top of the sandbox: read, search, write, patch, "
         "and execute, with structured results and streaming progress.",
    why="Tools are what make the agent useful and what make it dangerous. They land after the "
        "sandbox and before the approval UI so that every tool is deniable from the moment it exists.",
    scope=[
        "Tool registry with JSON-schema declarations and strict argument validation",
        "Read tools: `read_file`, `list_dir`, `glob`, `grep`",
        "Write tools: `write_file`, `edit_file`, `apply_patch` with atomic writes",
        "Execute tool: `run_command` with streamed stdout/stderr and exit codes",
        "Diff computation in Rust with hunk-level output",
        "Encoding and line-ending detection and preservation",
        "Tool result envelope: truncation with byte counts, never silent loss",
        "Tool activity events: start, progress, end, with durations",
        "Cost/limit guard hook points for Phase 11",
    ],
    out=[
        "Approval prompts (Phase 7)",
        "Network tools (Phase 11)",
        "MCP tools (Phase 19)",
    ],
    deliverables=[
        ("D6.1", "Tool registry with schema validation and versioned tool ids", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D6.2", "Read/search tools with deterministic, bounded output", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D6.3", "Write tools with atomic replace and preserved encoding/EOL", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D6.4", "`run_command` with streaming, timeout, and output caps", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D6.5", "Rust-side diff engine emitting hunks", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D6.6", "Tool activity timeline in the UI with per-class expanded views", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D6.7", "Tool result truncation policy with full payload retained locally", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D6.8", "Structured tool errors mapped to the taxonomy", "[error taxonomy](../04-specs/15-error-taxonomy.md)"),
    ],
    decisions=[
        "Tool calls are structured events from the engine, never parsed out of markdown — the single biggest correctness fix over the deprecated app.",
        "Diffs are computed in Rust and streamed as hunks; the webview never diffs large files.",
        "Writes are atomic (temp file plus rename) so an interrupted write cannot corrupt a source file.",
        "Encoding and line endings are preserved; silent normalization is treated as data loss.",
        "Every tool declares its class, which is what policy and approvals key on.",
    ],
    contracts=[
        "Tool schemas and ids",
        "Tool result envelope",
        "`tool_start`/`tool_progress`/`tool_end` events",
        "Diff hunk format",
    ],
    deps=["Phase 4 (engine and tool-call events)", "Phase 5 (sandbox)"],
    risks=[
        ("Tool output floods context", "Cost and latency blowups", "Hard output caps, structured truncation, retrieval-based reading in Phase 15"),
        ("Patch application fragility", "Failed or wrong edits", "Fuzzed patch corpus; exact-match then fuzzy fallback with an explicit failure rather than a guess"),
        ("Encoding regressions", "Corrupted files", "Round-trip tests across UTF-8/16, BOM, CRLF, and mixed-EOL fixtures"),
        ("Long-running commands", "Hung runs", "Timeouts, progress events, promotion to a terminal tab in Phase 12"),
    ],
    tests=[
        ("Unit", "Each tool's argument validation and result shape"),
        ("Integration", "Real filesystem and real processes through the sandbox"),
        ("Property", "Patch apply/revert round trips"),
        ("Security", "Every tool re-run through the traversal corpus"),
        ("Performance", "`read_file`, `grep`, and diff benchmarks against targets"),
    ],
    metrics=[
        ("`read_file` 10 KB", "< 10 ms p95"),
        ("`write_file` 10 KB", "< 25 ms p95"),
        ("`grep` across 10k files", "< 300 ms p95"),
        ("Diff 500-line file", "< 20 ms p95"),
        ("Tool argument validation rejection rate on the fuzz corpus", "100%"),
    ],
    exit=[
        "All v1 tools work through the sandbox with no direct filesystem access",
        "Tool activity timeline shows structured status and durations",
        "Diffs render with hunk-level detail from Rust-computed patches",
        "Encoding and EOL round trips are lossless",
        "Traversal corpus passes for every tool",
        "Tool performance targets met",
    ],
    defer=[
        "Approval gating (Phase 7)",
        "Live write preview in the editor (Phase 13)",
        "Checkpoints (Phase 18)",
    ],
))

PHASES.append(dict(
    n=7, title="Policy and Approval",
    track="B — Core runtime", size="2 weeks", milestone="—",
    goal="Make deny-by-default real: a local policy engine, grant scopes, approval "
         "cards, critical-risk dialogs, and the local audit queue.",
    why="Phase 6 gave the agent power. This phase makes that power consented and recorded. "
        "It must land immediately after tools, before any wider testing gives users a habit "
        "of unrestricted runs.",
    scope=[
        "Policy document schema, defaults, and the four presets",
        "Merge engine with the never-widen property",
        "Grant store: once, run, session, always-per-workspace, with normalized-shape matching",
        "Risk classification rules driving badges and typed confirmation",
        "Inline approval cards, critical-risk dialog, queueing, batching, timeout auto-deny",
        "Approvals pane with decision history and grant revocation",
        "Local audit queue with hash chaining and redaction before write",
        "Policy-denied UX with structured denial back to the agent",
    ],
    out=[
        "Cloud policy distribution (Phase 24)",
        "Cloud audit ingest (Phase 25)",
    ],
    deliverables=[
        ("D7.1", "Policy schema, validator, and the Permissive/Standard/Strict/Regulated presets", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
        ("D7.2", "Merge engine with property tests proving it never widens", "[policy engine](../03-security/03-permission-and-policy-engine.md)"),
        ("D7.3", "Grant store with scope expiry and shape-based matching", "[policy engine](../03-security/03-permission-and-policy-engine.md)"),
        ("D7.4", "Approval cards with risk levels, real buttons, and safe default focus", "[approval UX](../05-ui/05-approval-and-trust-ux.md)"),
        ("D7.5", "Approval queue, batching of identical shapes, 10-minute auto-deny", "[approval UX](../05-ui/05-approval-and-trust-ux.md)"),
        ("D7.6", "Approvals pane: pending, history, standing grants, revoke", "[approval UX](../05-ui/05-approval-and-trust-ux.md)"),
        ("D7.7", "Local audit queue: hash chain, redaction, durable pre-result write", "[audit and retention](../06-enterprise/04-audit-and-retention.md)"),
        ("D7.8", "Mode-to-permission profiles enforced for chat/agent/plan/ask/build", "[modes](../01-product/03-modes-and-workflows.md)"),
    ],
    decisions=[
        "The safest button always has default focus, and Allow is enabled only after 400 ms — accidental approval is a real threat.",
        "`Esc` always denies. There is no auto-approve on a timer.",
        "Standing grants match a normalized shape, never a raw string, so `npm test` cannot silently authorize a command substitution.",
        "Audit events are written durably before the tool result returns to the model, so a crash cannot hide an action.",
        "Policy evaluation happens at tool-call boundaries only; policy never changes mid-tool.",
    ],
    contracts=[
        "Policy document schema (`schemaVersion: 1`)",
        "Grant record shape and scope semantics",
        "Approval request/response IPC commands",
        "Audit event envelope",
    ],
    deps=["Phase 5 (guards)", "Phase 6 (tools to gate)", "Phase 2 (model)"],
    risks=[
        ("Approval fatigue", "Users blanket-allow", "Shape batching, per-run scopes, and read tools not requiring approval; measured approvals-per-run as a product metric"),
        ("Merge bugs widening permissions", "Security hole", "Never-widen property test, fuzzed policy pairs, golden merges"),
        ("Audit queue growth", "Disk pressure", "Size and age caps with fail-closed behavior when upload is required"),
        ("Risk misclassification", "Dangerous op looks routine", "Corpus of dangerous commands asserted to classify as elevated or critical"),
    ],
    tests=[
        ("Unit", "Merge semantics, grant expiry, risk classification"),
        ("Property", "Merge never widens; grant matching is shape-exact"),
        ("Integration", "Deny-by-default across every tool and mode from a fresh profile"),
        ("UI", "Card button order, `Esc` denies, 400 ms enable delay, keyboard-only decisions"),
        ("Security", "Fail-closed on corrupt policy; redaction fuzz corpus"),
    ],
    metrics=[
        ("Policy evaluation per tool call", "< 0.5 ms p95"),
        ("Approval card paint", "< 100 ms from request"),
        ("Deny-by-default suite", "100% pass"),
        ("Approvals per typical agent run", "< 3 median (product target)"),
    ],
    exit=[
        "Fresh profile denies every write, execute, and network tool without a grant",
        "All four grant scopes behave correctly, including expiry and revocation",
        "Merge engine passes the never-widen property tests",
        "Approval UX is fully keyboard-operable with a safe default",
        "Audit events are chained, redacted, and durable",
        "Corrupt or missing policy fails closed",
    ],
    defer=[
        "Cloud policy distribution and signing (Phase 24)",
        "Cloud audit ingest and export (Phase 25)",
    ],
))

PHASES.append(dict(
    n=8, title="Chat Experience",
    track="B — Core runtime", size="2 weeks", milestone="—",
    goal="Turn the streaming pipeline into the product surface: composer, turn rendering, "
         "markdown safety, interruption, steering, and context management.",
    why="Chat is where users spend their time and where the deprecated app's worst behaviors "
        "lived (fence parsing, regex output surgery, per-token animation). Doing it properly "
        "requires tools and approvals to already exist so their surfaces render natively.",
    scope=[
        "Composer: auto-grow, attachments, `@` mentions, `/` commands, persisted drafts",
        "Turn rendering with interleaved text, reasoning, and tool cards in emission order",
        "Sanitized markdown: allowlist renderer, code and diff fences, math, sandboxed mermaid",
        "Autoscroll with sticky-bottom and non-fighting detach behavior",
        "Abort, steering mid-run, and queued follow-ups",
        "Turn actions: copy, retry, retry-with-model, edit-and-resend, delete",
        "Context meter with explicit compaction (`/compact`), never silent truncation",
        "Citations for files and web sources",
        "Virtualized message list for long conversations",
    ],
    out=[
        "Persistence of history (Phase 9)",
        "Retrieval-driven context (Phase 15)",
    ],
    deliverables=[
        ("D8.1", "Composer with attachments, mentions, slash commands, drafts", "[chat UX](../05-ui/03-chat-and-streaming-ux.md)"),
        ("D8.2", "Turn renderer with frame-batched streaming and open-block parsing", "[chat UX](../05-ui/03-chat-and-streaming-ux.md)"),
        ("D8.3", "Sanitized markdown pipeline with confirm-on-external-link", "[injection defense](../03-security/05-prompt-injection-defense.md)"),
        ("D8.4", "Autoscroll and jump-to-latest behavior", "[chat UX](../05-ui/03-chat-and-streaming-ux.md)"),
        ("D8.5", "Abort, steering, and queued follow-ups", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D8.6", "Turn actions including retry and conversation forking", "[chat UX](../05-ui/03-chat-and-streaming-ux.md)"),
        ("D8.7", "Context meter and explicit compaction with a visible summary", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D8.8", "Message list virtualization above 200 turns", "[chat UX](../05-ui/03-chat-and-streaming-ux.md)"),
    ],
    decisions=[
        "Never delete or rewrite model output for display; hide with explicit affordances instead.",
        "Only the open markdown block re-parses per delta; closed blocks are memoized.",
        "No per-token animation — a blinking cursor conveys liveness at a fraction of the cost.",
        "Compaction is user-visible and reversible in the sense that the original turns remain stored.",
        "Untrusted content (tool output, web results) renders in a marked region and can never produce active content.",
    ],
    contracts=[
        "Conversation and message in-memory shapes (persisted in Phase 9)",
        "Slash-command registry entries",
        "Attachment reference format"
    ],
    deps=["Phase 4 (streaming)", "Phase 6 (tool cards)", "Phase 7 (approval surfaces)"],
    risks=[
        ("Markdown XSS", "Compromise of the webview", "Allowlist renderer, no raw HTML, CSP, and an injection corpus in CI"),
        ("Streaming jank on long conversations", "Poor perceived quality", "Virtualization plus frame batching, measured in the streaming benchmark"),
        ("Context overflow surprises", "Failed runs, wasted cost", "Live meter, warnings at 80%, explicit compaction path"),
        ("Steering semantics confusing", "Users think a message was ignored", "Queued messages render as pending with a clear label"),
    ],
    tests=[
        ("Unit", "Renderer sanitization, autoscroll state machine, composer behaviors"),
        ("Integration", "Long streaming session with tool cards and approvals interleaved"),
        ("Security", "Injection corpus rendered without active content or link auto-navigation"),
        ("Performance", "500-turn conversation scroll and 200 events/second streaming"),
        ("A11y", "Live-region announcement throttling; keyboard-only turn actions"),
    ],
    metrics=[
        ("Frame rate while streaming", "≥ 55 fps"),
        ("Main-thread work per delta batch", "< 4 ms"),
        ("500-turn conversation scroll", "60 fps"),
        ("Time to first rendered token after `meta`", "< 50 ms"),
    ],
    exit=[
        "Full conversation flow works with tools and approvals interleaved",
        "Markdown sanitization passes the injection corpus",
        "Abort, steering, and queued follow-ups behave as specified",
        "Context meter and compaction work with no silent truncation",
        "Performance and accessibility targets met",
    ],
    defer=[
        "History across restarts (Phase 9)",
        "Retrieval-based context assembly (Phase 15)",
    ],
))

PHASES.append(dict(
    n=9, title="Local Persistence",
    track="B — Core runtime", size="2 weeks", milestone="M1 Alpha",
    goal="Persist everything locally in an encrypted SQLite database with migrations, "
         "and reach the internal alpha: chat, tools, approvals, and history that survive restart.",
    why="Persistence is what turns a demo into a tool. It is also the phase that replaces the "
        "deprecated app's whole-file JSON store, which was slow, corruptible, and unqueryable "
        "([ADR-0004](../02-architecture/adr/0004-sqlite-local-store-and-encryption.md)).",
    scope=[
        "SQLite schema: workspaces, conversations, messages, runs, tool_calls, grants, settings, audit_queue",
        "Migration framework with forward-only, transactional migrations and pre-migration snapshots",
        "Encryption at rest with the key held in the OS keychain",
        "WAL mode, multi-window concurrency, and integrity checks on open",
        "Corruption recovery: WAL recovery, quarantine, fresh start with import offer",
        "Data export and import (conversations, runs, plans, settings) as JSON",
        "Retention and pruning for large tool payloads",
        "Deterministic eval harness skeleton running against the stub provider",
        "Alpha packaging for internal dogfood (unsigned)",
    ],
    out=[
        "Cloud sync (Phase 23)",
        "Index storage (Phase 15)",
    ],
    deliverables=[
        ("D9.1", "Full local schema with indexes on every query path", "[local data model](../04-specs/05-local-data-model.md)"),
        ("D9.2", "Migration framework with snapshot-before-migrate", "[local data model](../04-specs/05-local-data-model.md)"),
        ("D9.3", "At-rest encryption keyed from the OS keychain", "[secrets](../03-security/04-secrets-and-key-management.md)"),
        ("D9.4", "Multi-window safe access with WAL and busy-timeout handling", "[desktop runtime](../02-architecture/02-desktop-runtime.md)"),
        ("D9.5", "Corruption detection, quarantine, and recovery flow", "[runbooks](../07-ops/06-runbooks.md)"),
        ("D9.6", "Export/import round trip covering all local entities", "[backup and DR](../07-ops/07-backup-and-dr.md)"),
        ("D9.7", "Settings migrated from file-backed storage to SQLite", "[local data model](../04-specs/05-local-data-model.md)"),
        ("D9.8", "Eval harness skeleton with deterministic tasks in CI", "[eval harness](../08-quality/02-agent-eval-harness.md)"),
        ("D9.9", "Internal alpha build and dogfood feedback loop", "[release management](../07-ops/08-release-management.md)"),
    ],
    decisions=[
        "SQLite with WAL, not JSON files: queryable, incremental, and crash-resistant.",
        "Encryption key lives in the keychain; the database file alone is useless if copied.",
        "Migrations are forward-only within a major version, and every one takes a snapshot first so a bad release can be reverted.",
        "The agent is not a backup system: user code safety comes from git, and this store holds conversation state.",
        "The eval skeleton lands here because the stub provider plus persistence is the minimum needed to score behavior.",
    ],
    contracts=[
        "Local schema and migration version table",
        "Export bundle format",
        "Settings key namespace (`ui.*`, `agent.*`, `policy.*`)",
    ],
    deps=["Phase 8 (data worth persisting)", "Phase 7 (grants and audit queue)"],
    risks=[
        ("Migration bugs losing data", "Unacceptable", "Snapshots, transactional migrations, round-trip tests, and a forced-restore test in CI"),
        ("Encryption breaking keychain portability", "Users locked out", "Documented recovery path; export before migration; clear error rather than silent reset"),
        ("Multi-window write contention", "Lock errors", "WAL, short transactions, busy timeout, and a concurrency stress test"),
        ("Large payload growth", "Disk bloat", "Payload caps with pruning policy and a visible storage breakdown in Settings"),
    ],
    tests=[
        ("Unit", "Each migration up-path; schema constraints"),
        ("Integration", "Restart persistence, multi-window concurrency, WAL recovery"),
        ("Property", "Export → wipe → import equality"),
        ("Fault injection", "Kill mid-write and mid-migration; database remains usable"),
        ("Performance", "500-message conversation load benchmark"),
    ],
    metrics=[
        ("Conversation load (500 messages)", "< 50 ms p95"),
        ("Message insert during streaming", "< 2 ms p95"),
        ("Migration time on a 1 GB database", "< 10 s"),
        ("Data loss in fault-injection suite", "0"),
    ],
    exit=[
        "All state survives restart, including drafts, pane history, terminal tabs, grants, and audit queue",
        "Migrations are transactional with a pre-migration snapshot",
        "Encryption verified: no plaintext content in the database file",
        "Corruption recovery works without losing unaffected data",
        "Export/import round trip is lossless",
        "Alpha build in internal dogfood with feedback captured",
        "M1 Alpha milestone declared",
    ],
    defer=[
        "Cloud sync (Phase 23)",
        "Full eval suite (Phase 28)",
    ],
))

# ---------------------------------------------------------------- Track C

PHASES.append(dict(
    n=10, title="Identity",
    track="C — Product surfaces", size="2 weeks", milestone="—",
    goal="Ship the Agent-owned authentication stack: a self-contained NestJS module at "
         "`/agent/v1/auth/*`, PKCE device flow, rotating refresh tokens, and keychain storage "
         "on the desktop.",
    why="Everything commercial and enterprise depends on identity, and it must be built as an "
        "isolated module from the start. Reusing the legacy `/auth/ide/*` flow would couple the "
        "new product to the deprecated one and inherit its open-endpoint bug.",
    scope=[
        "`AgentModule` created in the backend with its own controllers, guards, DTOs, and config",
        "PKCE S256 device flow: start, browser exchange, poll",
        "Rotating refresh tokens with chain revocation on reuse detection",
        "`AgentAuthGuard` with no fallback and boot-time config validation",
        "Prisma migration `agent_identity`: `AgentDevice`, `AgentAuthSession`, `AgentRefreshToken`",
        "Device enrollment with an Ed25519 signing key for audit batches",
        "Web authorize page at `/{locale}/agent/authorize`",
        "Desktop login UI, session state, token refresh, logout, device management",
        "Module isolation lint check in CI",
    ],
    out=[
        "SSO and SCIM (Phase 24)",
        "Managed gateway (Phase 11)",
    ],
    deliverables=[
        ("D10.1", "`AgentModule` skeleton with all sub-areas and zero feature-module imports", "[module structure](../04-specs/16-backend-module-structure.md)"),
        ("D10.2", "Device flow endpoints with rate limiting and slow-down handling", "[cloud API](../04-specs/06-cloud-api-contract.md)"),
        ("D10.3", "Refresh rotation with reuse detection and chain revocation", "[cloud architecture](../02-architecture/03-cloud-architecture.md)"),
        ("D10.4", "`agent_identity` Prisma migration, additive only", "[cloud data model](../04-specs/07-cloud-data-model.md)"),
        ("D10.5", "`GET /agent/v1/me` returning explicit permissions and limits", "[RBAC](../06-enterprise/02-rbac-and-org-model.md)"),
        ("D10.6", "Device enrollment and key registration", "[audit and retention](../06-enterprise/04-audit-and-retention.md)"),
        ("D10.7", "Desktop keychain storage for access and refresh tokens", "[secrets](../03-security/04-secrets-and-key-management.md)"),
        ("D10.8", "Web authorize page with clear device identification", "[identity](../06-enterprise/01-identity-sso-scim.md)"),
        ("D10.9", "CI isolation check proving no cross-module imports", "[CI/CD](../07-ops/02-ci-cd-pipelines.md)"),
    ],
    decisions=[
        "The Agent module owns its tokens, tables, and guards; it duplicates a little logic rather than coupling to the auth module.",
        "PKCE `plain` is rejected outright; only S256 is accepted.",
        "Refresh reuse is treated as compromise: the whole chain is revoked and an audit event is raised.",
        "Missing auth configuration fails app boot. There is no environment variable that can make an Agent route public.",
        "BYOK users can work without signing in at all, so login is never a hostage for basic functionality.",
    ],
    contracts=[
        "`/agent/v1/auth/*` request and response shapes",
        "`AgentPrincipal` shape",
        "`AgentDevice`, `AgentAuthSession`, `AgentRefreshToken` schemas",
    ],
    deps=["Phase 3 (settings UI)", "Phase 9 (local session state)", "Phase 2 (secret rules)"],
    risks=[
        ("Token theft from a compromised machine", "Account access", "Short access TTL, rotation, device binding, revocation, and strict-revocation mode for enterprises"),
        ("Device flow abuse", "Phishing a user into approving", "Device name and platform shown prominently; short TTL; single-use codes; rate limits"),
        ("Accidental coupling to legacy auth", "Isolation broken", "CI import check plus review ownership on the module path"),
        ("Keychain unavailable", "Cannot store tokens", "Explicit error with a documented remedy; never fall back to a file"),
    ],
    tests=[
        ("Unit", "PKCE verification, rotation, reuse detection, guard behavior"),
        ("Contract", "Every auth endpoint against golden fixtures"),
        ("Negative", "No token, expired, wrong audience, `alg:none`, revoked device, missing config"),
        ("Integration", "Full desktop login against a local backend, including logout and refresh"),
        ("Security", "Rate-limit enforcement; replayed device code and assertion rejection"),
    ],
    metrics=[
        ("Login flow completion (user-paced)", "< 30 s median"),
        ("`/agent/v1/auth/refresh` latency", "< 50 ms p95"),
        ("Refresh reuse detection", "100% in tests"),
        ("Cross-module imports in the Agent module", "0"),
    ],
    exit=[
        "Desktop login works end to end against a local backend",
        "Tokens are stored only in the OS keychain",
        "Refresh rotation and reuse detection verified",
        "Negative-auth suite green, including the missing-config boot failure",
        "Prisma migration is additive and reversible by table drop",
        "CI isolation check passes",
    ],
    defer=[
        "SSO, SCIM, and personal access tokens (Phase 24)",
    ],
))

PHASES.append(dict(
    n=11, title="Model Routing and Cost",
    spec_version="1.4.0",
    track="C — Product surfaces", size="2 weeks", milestone="—",
    goal="Support Eury-managed gateway inference with a policy-aware model catalog, network "
         "tools, and real cost accounting with caps. BYOK is deferred to a follow-up phase — "
         "all desktop traffic for this phase routes through the managed gateway.",
    why="Model choice, cost visibility, and the managed path are what make the product viable "
        "commercially and predictable for users. It lands after identity because the gateway "
        "requires authentication and entitlements.",
    scope=[
        "Model catalog served by the Agent module with policy-aware `allowed` flags",
        "Managed gateway `POST /agent/v1/chat/stream` with NDJSON passthrough and abort propagation",
        "Network tools: `web_search` and `web_fetch` through the Agent module proxy",
        "Vision attachments (base64 inline) and consent-gated image generation for capable models",
        "Cost estimation with a versioned price table; per-run and per-day caps, enforced server-side",
        "Model picker UI with context window, cost hint, and disabled reasons",
        "Usage surface in Settings backed by `GET /agent/v1/usage/current`",
    ],
    out=[
        "Quota enforcement at the org level (Phase 25)",
        "Budgets and admin dashboards (Phase 25)",
    ],
    deliverables=[
        ("D11.1", "Provider abstraction routing four catalog providers (OpenAI, Anthropic, Google, xAI) through the managed gateway", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D11.2", "`GET /agent/v1/models` with plan and policy filtering", "[cloud API](../04-specs/06-cloud-api-contract.md)"),
        ("D11.3", "Managed gateway with tee'd NDJSON, 300 s timeout, and abort within 250 ms", "[cloud architecture](../02-architecture/03-cloud-architecture.md)"),
        ("D11.4", "Server-side usage metering and cost-cap enforcement: `AgentUsageCounter` incremented per completed stream, cost/token caps on `AgentChatStreamDto`", "[usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md)"),
        ("D11.5", "Network tools with SSRF protections and untrusted-content marking", "[injection defense](../03-security/05-prompt-injection-defense.md)"),
        ("D11.6", "Client-side cost guard enforcing `maxCostPerRunUsdMicros` and `maxTokensPerRun`", "[usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md)"),
        ("D11.7", "Model picker and run cost display", "[chat UX](../05-ui/03-chat-and-streaming-ux.md)"),
        ("D11.8", "Versioned price table with integer micro-USD accounting, single-sourced between desktop and backend", "[usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md)"),
        ("D11.9", "Capability-gated image input and generation, encrypted attachment storage, and explicit save flow", "[multimodal spec](../04-specs/17-multimodal-and-attachment-spec.md)"),
    ],
    decisions=[
        "The managed gateway is the only inference path delivered in this phase; BYOK (ADR-0005) is deferred to a follow-up phase and re-evaluated once server-side usage metering is live.",
        "Money is integer micro-USD end to end; no floating-point currency anywhere.",
        "Model failover never silently changes the model — the user and the audit log always see which model ran.",
        "Web results are untrusted content and are marked as such before entering a prompt.",
        "Vision observations and generated images are untrusted content; generated images never write into a project until the user approves a normal file save.",
        "Provider/model rollout is governed by data classification, evaluation gates, staged rollout, and explicit fallback disclosure.",
        "The gateway is a thin proxy: it validates, meters, and forwards; it never runs the agent loop — so caps and usage must be enforced server-side, not only in the desktop client.",
    ],
    contracts=[
        "`/agent/v1/models` and `/agent/v1/chat/stream` shapes",
        "`/agent/v1/tools/web-search` shape",
        "Image provider capability, generation, and attachment-reference shapes",
        "Provider/model governance, routing, and rollout evidence",
        "Provider client interface",
        "Price table format and versioning",
    ],
    deps=["Phase 4 (engine and provider interface)", "Phase 10 (auth and entitlements)"],
    risks=[
        ("Provider API drift", "Broken streaming", "Recorded cassettes per provider in CI; adapter per provider; contract tests"),
        ("Gateway added latency", "Feels slower than a direct provider call", "Guard chain budget of 10 ms; streaming passthrough without buffering; measured in CI"),
        ("Cost estimates wrong", "User distrust or overspend", "Versioned price table, monthly reconciliation, estimates labeled as estimates"),
        ("SSRF via fetch tools", "Internal network exposure", "Address allowlisting, blocked link-local and metadata ranges, DNS rebinding protection"),
        ("Caps enforced only client-side", "Non-official clients bypass spend limits entirely", "Server-side `AgentUsageCounter` increment and cap check on every gateway stream, not just the desktop `CostGuard`"),
    ],
    tests=[
        ("Unit", "Provider adapters against recorded streams; price math"),
        ("Contract", "Gateway request/response and error codes"),
        ("Integration", "Gateway streaming path produces a spec-conformant event stream end to end"),
        ("Security", "SSRF corpus; gateway auth token never leaves the OS keychain; no token in logs"),
        ("Security", "Image metadata stripping; generated output cannot create a workspace file without a separate write approval"),
        ("Evaluation", "Provider capability fixtures, fallback disclosure, and model-change regression gates"),
        ("Performance", "Gateway added latency and guard chain benchmarks"),
    ],
    metrics=[
        ("Gateway added latency", "< 400 ms p95"),
        ("Guard chain (auth + quota + policy)", "< 10 ms p95"),
        ("Time to first token via gateway", "< 1.2 s p95"),
        ("Cost estimate error vs. provider invoice", "< 2%"),
    ],
    exit=[
        "Managed gateway streams inference for catalog models via `EuryGatewayProvider`",
        "Model catalog reflects plan and policy restrictions with visible reasons, enforced server-side (not a hardcoded allow)",
        "Abort propagates from desktop through the gateway to the provider",
        "Cost and usage caps are enforced server-side (`AgentUsageCounter` incremented, caps checked before forwarding), not only by the desktop `CostGuard`",
        "Network tools pass the SSRF corpus",
        "Vision attachments and image generation honor model capability, policy, cost consent, and explicit-save rules",
        "Latency targets met",
    ],
    defer=[
        "Org quotas, budgets, and dashboards (Phase 25)",
        "BYOK key management, per-provider validation, and BYOK/gateway fallback routing (follow-up phase; ADR-0005 remains the target end state)",
    ],
))

PHASES.append(dict(
    n=12, title="Terminal",
    track="C — Product surfaces", size="1–2 weeks", milestone="—",
    goal="A real PTY-backed terminal pane with high-throughput rendering, plus promotion of "
         "long-running tool commands into that pane.",
    why="Developers need to see and drive the shell themselves, and agent-run commands need "
        "somewhere to live when they outgrow a collapsed tool card.",
    scope=[
        "PTY management in Rust with `portable-pty`; ConPTY on Windows",
        "xterm.js integration with a ring buffer and frame-coalesced writes",
        "Up to four sessions per workspace as tabs, with resize and signal forwarding",
        "Sanitized environment; no secrets injected",
        "Promotion of a `run_command` tool execution into a terminal view",
        "Explicit `share terminal output` action to expose text to the agent",
        "Selection to composer; clear, kill, and scrollback controls",
    ],
    out=[
        "Agent typing into a user terminal (explicitly never)",
        "Remote or SSH terminals",
    ],
    deliverables=[
        ("D12.1", "PTY lifecycle with resize, signals, and reliable kill", "[editor/terminal/preview](../05-ui/06-editor-terminal-preview.md)"),
        ("D12.2", "xterm.js renderer sustaining 10 MB/s without UI stall", "[editor/terminal/preview](../05-ui/06-editor-terminal-preview.md)"),
        ("D12.3", "Terminal pane with per-session tabs and scrollback", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D12.4", "Tool-to-terminal promotion preserving history", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D12.5", "Explicit output-sharing action with an untrusted-content marker", "[injection defense](../03-security/05-prompt-injection-defense.md)"),
        ("D12.6", "Windows ConPTY support with a degradation warning when unavailable", "[desktop runtime](../02-architecture/02-desktop-runtime.md)"),
    ],
    decisions=[
        "The agent never types into a user terminal; agent commands always go through `run_command` with policy and sandbox enforcement.",
        "Terminal output only becomes model context when the user explicitly shares it, and then it is marked untrusted.",
        "Promotion is a view change, not a permission change.",
        "Environment is sanitized: keychain secrets are never injected into a shell the model can influence.",
    ],
    contracts=[
        "Terminal IPC commands (create, write, resize, kill)",
        "Terminal output event framing",
    ],
    deps=["Phase 5 (process supervision)", "Phase 6 (`run_command`)", "Phase 3 (pane shell)"],
    risks=[
        ("Windows PTY differences", "Broken terminal on Windows", "ConPTY-first implementation, platform test matrix, honest degradation"),
        ("Output flooding the UI", "Freeze", "Ring buffer, frame coalescing, and dropping output frames rather than UI frames"),
        ("Terminal used as an injection channel", "Agent misdirection", "Sharing is explicit and marked untrusted"),
        ("Orphaned shells", "Resource leak", "Process-group kill on tab close and on app quit; leak test"),
    ],
    tests=[
        ("Unit", "PTY lifecycle, resize, signal handling"),
        ("Integration", "Interactive programs (`vim`, `top`), ANSI rendering, large output"),
        ("Performance", "10 MB/s throughput with frame-rate assertion"),
        ("Platform", "All three OSes including ConPTY"),
        ("Leak", "No orphaned processes after close or quit"),
    ],
    metrics=[
        ("First prompt after tab open", "< 300 ms"),
        ("Throughput", "≥ 10 MB/s with UI ≥ 55 fps"),
        ("Keystroke echo latency", "< 16 ms p95"),
        ("Orphaned shells", "0"),
    ],
    exit=[
        "Terminal works on all three platforms with interactive programs",
        "Throughput and latency targets met",
        "Tool commands can be promoted to a terminal without losing history",
        "Output sharing is explicit and marked untrusted",
        "No orphaned processes in the leak suite",
    ],
    defer=[
        "Remote and container shells (post-GA)",
    ],
))

PHASES.append(dict(
    n=13, title="Editor and Explorer",
    track="C — Product surfaces", size="2 weeks", milestone="—",
    goal="A CodeMirror-based editor with live write preview, plus a file explorer and search, "
         "so users can review and steer the agent's edits in place.",
    why="Live write preview was the deprecated app's best idea. Reproducing it properly requires "
        "the diff engine from Phase 6 and the approval flow from Phase 7 to already exist.",
    scope=[
        "CodeMirror 6 with 20 bundled languages and plain fallback",
        "Read-only mode for untrusted workspaces; edit and save through the sandbox",
        "Live write preview decorations for pending agent writes, with hunk navigation",
        "Hunk-level apply and skip, reporting skipped hunks back to the agent",
        "External and agent change detection with safe reload behavior",
        "File explorer with git status decorations (statuses land fully in Phase 14)",
        "Workspace search and replace with preview",
        "Encoding and EOL display and preservation; large and binary file handling",
    ],
    out=[
        "LSP features (deferred; see open questions)",
        "Debugging",
    ],
    deliverables=[
        ("D13.1", "Editor with save through the sandbox path guard", "[editor/terminal/preview](../05-ui/06-editor-terminal-preview.md)"),
        ("D13.2", "Live write preview overlay with pending-hunk gutter markers", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D13.3", "Hunk apply/skip with structured feedback to the agent", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D13.4", "Changes pane with multi-file diff review for a whole run", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D13.5", "File explorer with lazy loading for large trees", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D13.6", "Workspace search and replace with per-match preview", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D13.7", "Encoding/EOL preservation with visible indicators", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D13.8", "Cross-surface actions: ask about selection, open at line, insert at cursor", "[keyboard](../05-ui/08-keyboard-and-command-palette.md)"),
    ],
    decisions=[
        "CodeMirror over Monaco: roughly a seventh of the bundle size, faster mount, and a decoration API that suits diff overlays.",
        "Editing a file with a pending preview dismisses and invalidates that write rather than merging blindly.",
        "Partial hunk application is allowed and is reported to the agent, so it can react instead of assuming success.",
        "The editor is deliberately not an IDE; language intelligence stays out of scope for v1.",
    ],
    contracts=[
        "Pending-write preview IPC commands and events",
        "Hunk apply/skip result reported into the tool result",
    ],
    deps=["Phase 6 (diffs and write tools)", "Phase 7 (approvals)", "Phase 3 (pane shell)"],
    risks=[
        ("Preview desynchronizing from the file", "Wrong edits applied", "Content hash checked at apply time; mismatch invalidates the pending write"),
        ("Large file performance", "Freeze", "Size thresholds, virtualization, highlight disabled beyond limits"),
        ("Encoding mishandling", "Corrupted files", "Round-trip tests across encodings and EOL styles"),
        ("Explorer on huge repos", "Slow UI", "Lazy loading, virtualized tree, throttled watchers"),
    ],
    tests=[
        ("Unit", "Decoration mapping, hunk selection state, dirty-buffer logic"),
        ("Integration", "Agent write → preview → partial apply → save → verify on disk"),
        ("Property", "Apply/skip combinations produce the expected file content"),
        ("Performance", "1k, 100k-line files; 50k-file explorer tree"),
        ("A11y", "Keyboard hunk navigation and apply"),
    ],
    metrics=[
        ("Open 1000-line file", "< 120 ms p95"),
        ("Open 100k-line file", "< 600 ms p95"),
        ("Preview decoration render", "< 50 ms"),
        ("Keystroke latency while streaming", "< 16 ms p95"),
    ],
    exit=[
        "Live write preview works with hunk-level apply and skip",
        "Skipped hunks are reported to the agent as not applied",
        "Encoding and EOL round trips are lossless",
        "Explorer and search perform acceptably on a 50k-file repo",
        "Untrusted workspaces are read-only in the editor",
    ],
    defer=[
        "LSP integration (open question Q07)",
        "Git status detail (Phase 14)",
    ],
))

PHASES.append(dict(
    n=14, title="Git",
    track="C — Product surfaces", size="1–2 weeks", milestone="—",
    goal="First-class git awareness: status, diff, staging, commit, branch context, and "
         "agent-authored commits with human-reviewable messages.",
    why="Git is the user's real safety net and the agent's most useful context source. It also "
        "makes checkpoints in Phase 18 cheaper by giving them a baseline to reason about.",
    scope=[
        "`git2`-based status, branch, remote, and diff reading",
        "Staging, unstaging, and committing from the UI",
        "Agent-facing git tools: status, diff, log, blame, branch — read-only by default",
        "Commit message generation with a mandatory human review step",
        "Push and pull as elevated, approval-gated operations",
        "Branch and worktree awareness for background runs in Phase 21",
        "Conflict detection with a clear read-only presentation",
        "`.gitignore` awareness in indexing and explorer",
    ],
    out=[
        "Interactive rebase and merge resolution UI",
        "GitHub/GitLab API integration (post-GA)",
    ],
    deliverables=[
        ("D14.1", "Git status and diff reading with efficient refresh on file change", "[feature catalog](../01-product/02-feature-catalog.md)"),
        ("D14.2", "Stage, unstage, and commit UI with diff review", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D14.3", "Read-only git tools for the agent", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D14.4", "Commit message drafting with required human confirmation", "[approval UX](../05-ui/05-approval-and-trust-ux.md)"),
        ("D14.5", "Push/pull as elevated operations requiring explicit approval", "[policy engine](../03-security/03-permission-and-policy-engine.md)"),
        ("D14.6", "Conflict state detection and presentation", "[feature catalog](../01-product/02-feature-catalog.md)"),
        ("D14.7", "`.gitignore` respected by index, explorer, and search", "[indexing spec](../04-specs/09-indexing-and-retrieval-spec.md)"),
    ],
    decisions=[
        "Git write operations that leave the machine (push) are always elevated risk and never covered by a broad standing grant.",
        "The agent may draft a commit message but never commits without human confirmation in v1.",
        "History rewriting operations are not exposed to the agent at all.",
        "Git is read via `git2` rather than shelling out, so status refresh does not depend on command approval.",
    ],
    contracts=[
        "Git tool schemas and result shapes",
        "Git status event for UI decorations",
    ],
    deps=["Phase 13 (editor and explorer decorations)", "Phase 7 (approval classes)"],
    risks=[
        ("Destructive git operations", "Lost work", "Reset, force push, and history rewrite are not agent-accessible; push is approval-gated"),
        ("Status refresh cost on large repos", "UI lag", "Debounced, incremental refresh; watcher-driven rather than polling"),
        ("Submodules and worktrees", "Incorrect status", "Explicitly detected and reported; unsupported cases surfaced rather than guessed"),
        ("Credential handling", "Leaked credentials", "Delegate to the system credential helper; never store git credentials ourselves"),
    ],
    tests=[
        ("Unit", "Status parsing, diff generation, ignore handling"),
        ("Integration", "Fixture repos: clean, dirty, staged, conflicted, detached HEAD, submodule"),
        ("Security", "Push requires approval; no history-rewriting tool is registered"),
        ("Performance", "Status refresh on a 50k-file repo"),
    ],
    metrics=[
        ("Status refresh (50k files)", "< 500 ms p95"),
        ("Diff for a staged change", "< 100 ms p95"),
        ("Destructive git operations reachable by the agent", "0"),
    ],
    exit=[
        "Status, diff, stage, and commit work from the UI",
        "Agent git tools are read-only and pass the tool security suite",
        "Commit messages require human confirmation",
        "Push and pull are approval-gated as elevated operations",
        "Conflicts are detected and clearly presented",
    ],
    defer=[
        "Merge conflict resolution UI (post-GA)",
        "Forge API integration (post-GA)",
    ],
))

# ---------------------------------------------------------------- Track D

PHASES.append(dict(
    n=15, title="Code Intelligence",
    track="D — Intelligence", size="3 weeks", milestone="—",
    goal="Index the workspace and retrieve only what matters, so context assembly is fast, "
         "cheap, and accurate on large repositories.",
    why="Without retrieval, every run either wastes tokens or misses the relevant file. This is "
        "the phase that determines whether the agent works on a real monorepo "
        "([ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md)).",
    scope=[
        "Incremental indexer: file walk, ignore rules, content hashing, change detection",
        "Lexical index (trigram/inverted) for fast literal and regex search",
        "Symbol extraction via tree-sitter for the top languages",
        "Optional local embeddings with a bundled model; hybrid ranking",
        "Retrieval pipeline: query expansion, hybrid scoring, dedup, budget-aware packing",
        "Context assembly with explicit provenance shown in the UI",
        "Index lifecycle: background build, throttling, pause on battery, staleness handling",
        "Graceful degradation on very large repos and on unsupported languages",
    ],
    out=[
        "Cloud-hosted embeddings (not planned; local only)",
        "Cross-repository search",
    ],
    deliverables=[
        ("D15.1", "Incremental indexer with watcher-driven updates", "[indexing spec](../04-specs/09-indexing-and-retrieval-spec.md)"),
        ("D15.2", "Lexical index powering `grep`/`glob` and palette search", "[indexing spec](../04-specs/09-indexing-and-retrieval-spec.md)"),
        ("D15.3", "Tree-sitter symbol extraction for the top 10 languages", "[indexing spec](../04-specs/09-indexing-and-retrieval-spec.md)"),
        ("D15.4", "Local embedding option with a hybrid ranker", "[ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md)"),
        ("D15.5", "Retrieval pipeline with a token budget and dedup", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D15.6", "Context panel showing exactly what was sent, with scores", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D15.7", "Index state UI: queued, scanning, embedding, ready, failed with reasons", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D15.8", "Degradation policy for 200k+ file repos", "[offline and degraded modes](../02-architecture/06-offline-and-degraded-modes.md)"),
    ],
    decisions=[
        "Indexing is local only. Sending a repository to a hosted embedding service is incompatible with our privacy posture.",
        "Lexical search is the reliable floor; embeddings are an enhancement, and the product must be good without them.",
        "Retrieved context is always shown to the user with provenance — an agent that cites invisible context is unauditable.",
        "The index never gates chat; retrieval quality degrades until the index is ready.",
        "Indexing respects `.gitignore` plus a secret-file deny list, so `.env` files are not indexed at all.",
    ],
    contracts=[
        "Index storage layout and version",
        "Retrieval result shape with scores and provenance",
        "Context assembly inputs and outputs",
    ],
    deps=["Phase 9 (storage)", "Phase 6 (file access through the sandbox)"],
    risks=[
        ("Index build cost on large repos", "Battery and CPU complaints", "Throttling, battery awareness, incremental updates, configurable exclusions"),
        ("Retrieval misses the right file", "Poor answers", "Eval categories specifically for retrieval; hybrid ranking; symbol-aware boosts"),
        ("Embedding model size", "Installer bloat", "Optional download rather than bundled by default; lexical-only remains fully supported"),
        ("Secrets indexed", "Leak into prompts", "Deny list plus secret-shape detection with redaction before assembly"),
    ],
    tests=[
        ("Unit", "Ignore rules, hashing, chunking, ranking math"),
        ("Integration", "1k/10k/50k/200k-file fixtures; incremental update correctness"),
        ("Quality", "Retrieval precision/recall on a labeled query set"),
        ("Security", "Secret files never appear in the index or in assembled context"),
        ("Performance", "Build, incremental update, and assembly benchmarks"),
    ],
    metrics=[
        ("Full index (10k files)", "< 60 s background"),
        ("Incremental update (1 file)", "< 200 ms"),
        ("Context assembly (10k files)", "< 30 ms p95"),
        ("Context assembly (50k files)", "< 80 ms p95"),
        ("Retrieval precision@5 on the labeled set", "≥ 0.8"),
    ],
    exit=[
        "Index builds incrementally and survives restart",
        "Retrieval meets the precision target on the labeled query set",
        "Context panel shows every retrieved item with provenance",
        "Secret files are provably excluded",
        "Assembly latency targets met on 10k and 50k-file repos",
        "200k-file repo degrades gracefully rather than failing",
    ],
    defer=[
        "LSP-quality cross-references (open question)",
        "Cross-repo retrieval (post-GA)",
    ],
))

PHASES.append(dict(
    n=16, title="Memory",
    track="D — Intelligence", size="2 weeks", milestone="—",
    goal="Give the agent durable, inspectable, user-controlled memory: the `EURY.md` hierarchy, "
         "a local memory graph, and explicit pinning.",
    why="Repeating project conventions every session is the most common source of user frustration. "
        "Memory must be visible and editable, or it becomes an unpredictable hidden prompt.",
    scope=[
        "`EURY.md` hierarchy: global, workspace, subdirectory, plus `.eury.local.md` overrides",
        "`CLAUDE.md` compatibility import as a one-time, explicit action",
        "Memory graph with entities, relations, and provenance, stored locally",
        "Automatic memory extraction proposals that require user confirmation",
        "Explicit pin and forget actions; memory inspector UI with search",
        "Recall integrated into context assembly under a token budget",
        "Memory scoping rules: never leak one workspace's memory into another",
        "Export and import of memory",
    ],
    out=[
        "Cloud memory sync (Phase 23)",
        "Org-shared memory (post-GA)",
    ],
    deliverables=[
        ("D16.1", "`EURY.md` hierarchy loader with precedence and size caps", "[memory spec](../04-specs/08-memory-spec.md)"),
        ("D16.2", "Memory graph store with provenance per entry", "[memory spec](../04-specs/08-memory-spec.md)"),
        ("D16.3", "Extraction proposals shown for confirmation, never auto-written", "[memory spec](../04-specs/08-memory-spec.md)"),
        ("D16.4", "Memory inspector: browse, search, edit, pin, forget", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D16.5", "Recall in context assembly with a bounded token budget", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D16.6", "Per-workspace scoping with an explicit global tier", "[memory spec](../04-specs/08-memory-spec.md)"),
        ("D16.7", "Memory export/import and the `CLAUDE.md` import path", "[memory spec](../04-specs/08-memory-spec.md)"),
    ],
    decisions=[
        "Memory is never written silently. Every automatic extraction is a proposal the user accepts or rejects.",
        "`EURY.md` is plain markdown in the repository, so it is reviewable and versioned by git like any other convention document.",
        "Memory is scoped per workspace by default; cross-workspace leakage is a bug, not a feature.",
        "Recall latency is measured independently rather than trusted from vendor claims ([benchmarks](../08-quality/03-performance-benchmarks.md)).",
        "Untrusted repository content cannot write memory — a hostile repo must not be able to plant instructions.",
    ],
    contracts=[
        "`EURY.md` precedence rules",
        "Memory entry schema with provenance",
        "Recall query and result shapes",
    ],
    deps=["Phase 15 (assembly pipeline)", "Phase 9 (storage)"],
    risks=[
        ("Memory poisoning from repo content", "Persistent misdirection", "Untrusted content cannot write memory; proposals require confirmation and show their source"),
        ("Stale memory", "Wrong behavior over time", "Provenance and timestamps shown; easy forget; conflicting entries surfaced"),
        ("Memory bloat", "Token waste", "Size caps, relevance-ranked recall, and a visible budget"),
        ("Vendor recall claims unverified", "Wrong performance assumptions", "Independent benchmark before relying on graph recall in the hot path"),
    ],
    tests=[
        ("Unit", "Hierarchy precedence, extraction proposal generation, scoping"),
        ("Integration", "Recall affects behavior in a scripted multi-session scenario"),
        ("Security", "Hostile repo cannot write memory or influence global tier"),
        ("Performance", "Recall latency benchmark against the < 1 ms target"),
    ],
    metrics=[
        ("Memory recall", "< 1 ms p95"),
        ("Memory token budget per run", "< 2000 tokens default"),
        ("Silent memory writes", "0"),
    ],
    exit=[
        "`EURY.md` hierarchy loads with correct precedence",
        "Memory graph recall meets the latency target on our own benchmark",
        "No memory is written without user confirmation",
        "Memory inspector supports search, edit, pin, and forget",
        "Hostile-repo memory poisoning test passes",
    ],
    defer=[
        "Cloud memory sync (Phase 23)",
        "Team-shared memory (post-GA)",
    ],
))

PHASES.append(dict(
    n=17, title="Modes and Plan Execution",
    track="D — Intelligence", size="2 weeks", milestone="M2 Beta",
    goal="Ship the five modes as real permission profiles and plan mode as a durable, "
         "reviewable, step-by-step execution model. Reach beta.",
    why="Plan mode is the answer to 'the agent did too much at once'. It converts a risky "
        "long run into a reviewable sequence, and it needs tools, policy, memory, and the "
        "editor already in place.",
    scope=[
        "Mode profiles enforcing distinct tool sets and approval requirements",
        "Plan format as markdown with structured frontmatter, stored in `<workspace>/.eury/plans/`",
        "Plan generation, editing, and re-planning after a failed step",
        "Step-by-step execution with per-step approval, skip, and retry",
        "Plan progress in the context panel with per-step tool attribution",
        "`requirePlanBeforeWrite` policy support for plan-gated organizations",
        "Plan resume across restarts",
        "Beta packaging and the opt-in beta channel",
    ],
    out=[
        "Multi-agent execution of plan steps (Phase 20)",
        "Scheduled plan runs (Phase 21)",
    ],
    deliverables=[
        ("D17.1", "Mode profiles with per-mode default tools and approval rules", "[modes](../01-product/03-modes-and-workflows.md)"),
        ("D17.2", "Plan file format with a validating parser and writer", "[plan format](../04-specs/11-plan-format-spec.md)"),
        ("D17.3", "Plan generation and human editing before execution", "[plan format](../04-specs/11-plan-format-spec.md)"),
        ("D17.4", "Step executor with approval, skip, retry, and re-plan", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D17.5", "Plan progress UI with step-to-tool attribution", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D17.6", "Plan-gated writes enforced from policy", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
        ("D17.7", "Plan resume after restart or abort", "[checkpoint spec](../04-specs/12-checkpoint-and-rollback-spec.md)"),
        ("D17.8", "Beta build on the beta channel with dogfood telemetry", "[release management](../07-ops/08-release-management.md)"),
    ],
    decisions=[
        "Plans are markdown files in the repository: diffable, reviewable, and shareable without our app.",
        "Modes are permission profiles first and prompt framing second, which is what makes `ask` mode genuinely safe.",
        "A failed step stops execution and offers re-planning rather than improvising past the failure.",
        "Plan state is persisted after every step so a crash resumes rather than restarts.",
    ],
    contracts=[
        "Plan file schema and frontmatter",
        "Plan step execution events",
        "Mode profile definitions",
    ],
    deps=["Phase 7 (approvals)", "Phase 13 (review surfaces)", "Phase 16 (memory in planning)"],
    risks=[
        ("Plans too coarse or too granular", "Poor usability", "Eval tasks scoring plan quality; step-count guidance in the prompt; user editing always available"),
        ("Plan drift from reality", "Steps no longer make sense", "Re-plan on failure; plan references files by path with content hashes where relevant"),
        ("Mode confusion", "Unexpected permissions", "Mode badge always visible; approval cards state the mode; mode-permission matrix tested"),
        ("Plan files cluttering repos", "User annoyance", "Single directory, gitignore guidance, easy cleanup"),
    ],
    tests=[
        ("Unit", "Plan parse/serialize round trip; mode profile resolution"),
        ("Integration", "Generate, edit, execute, fail a step, re-plan, resume after restart"),
        ("Eval", "Plan-mode task category with validity and execution assertions"),
        ("Security", "Each mode's tool set enforced; `ask` mode cannot write"),
    ],
    metrics=[
        ("Plan generation latency", "< 15 s p95 (model-bound)"),
        ("Plan parse", "< 5 ms"),
        ("Plan-mode eval pass rate", "≥ 85%"),
        ("Plan resume success after forced kill", "100%"),
    ],
    exit=[
        "All five modes enforce distinct, tested permission profiles",
        "Plans generate, validate, edit, execute, and resume correctly",
        "Failed steps offer re-planning instead of silent continuation",
        "Plan-gated write policy works",
        "Beta channel build shipped to dogfood users",
        "M2 Beta milestone declared",
    ],
    defer=[
        "Parallel step execution via sub-agents (Phase 20)",
    ],
))

PHASES.append(dict(
    n=18, title="Checkpoints and Rollback",
    track="D — Intelligence", size="1–2 weeks", milestone="—",
    goal="Make every agent write reversible: automatic checkpoints, per-turn restore, and a "
         "clear preview of what a revert will do.",
    why="The single most valuable safety feature after approvals. Users tolerate an agent that "
        "makes mistakes if undo is instant and trustworthy.",
    scope=[
        "Automatic checkpoint before the first write of each turn",
        "Content-addressed snapshot storage with deduplication",
        "Restore per turn, per run, and per file, with a preview diff before applying",
        "Handling of renames, deletes, new files, binary files, and unusual encodings",
        "Interaction with git: checkpoints are independent of, and complementary to, commits",
        "Retention: age and size caps with visible storage usage and pruning",
        "Checkpoint audit events for create and restore",
        "Crash recovery offering resume or revert of an interrupted run",
    ],
    out=[
        "Cloud-backed checkpoints",
        "Cross-machine restore",
    ],
    deliverables=[
        ("D18.1", "Checkpoint creation hooked into the write path", "[checkpoint spec](../04-specs/12-checkpoint-and-rollback-spec.md)"),
        ("D18.2", "Content-addressed store with dedup and compression", "[checkpoint spec](../04-specs/12-checkpoint-and-rollback-spec.md)"),
        ("D18.3", "Restore with a preview diff and explicit file list", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D18.4", "`CheckpointBadge` per turn and the Changes panel revert actions", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D18.5", "Retention policy with storage visibility in Settings", "[local data model](../04-specs/05-local-data-model.md)"),
        ("D18.6", "Crash recovery flow offering resume or revert", "[runbooks](../07-ops/06-runbooks.md)"),
        ("D18.7", "Audit events for checkpoint create and restore", "[audit and retention](../06-enterprise/04-audit-and-retention.md)"),
    ],
    decisions=[
        "Checkpoints are snapshots of files the agent touched, not whole-repo copies — bounded cost, bounded restore.",
        "Restore always previews first. A revert that surprises the user is as bad as the original bad edit.",
        "Checkpoints do not create git commits; users who want commits get them explicitly in Phase 14.",
        "Checkpoint storage is capped and prunable, and its size is always visible.",
    ],
    contracts=[
        "Checkpoint record schema",
        "Restore request/response IPC commands",
        "Checkpoint audit events",
    ],
    deps=["Phase 6 (write path)", "Phase 9 (storage)", "Phase 13 (diff preview)"],
    risks=[
        ("Restore losing user edits made after the agent's write", "Data loss", "Preview shows conflicts; concurrent user edits block a silent overwrite and require explicit choice"),
        ("Storage growth", "Disk pressure", "Dedup, compression, age and size caps, visible usage"),
        ("Binary and large files", "Slow or huge snapshots", "Size thresholds with an explicit skip and a clear warning that those files are not revertible"),
        ("Partial restore leaving an inconsistent tree", "Broken build", "Restores are atomic per operation with a rollback on failure"),
    ],
    tests=[
        ("Unit", "Snapshot creation, dedup, retention pruning"),
        ("Property", "Write → checkpoint → restore returns byte-identical content across encodings"),
        ("Integration", "Renames, deletes, new files, binaries, and user-edited-after-write cases"),
        ("Fault injection", "Kill during restore; tree ends consistent"),
        ("Performance", "Checkpoint overhead per write; restore of a 100-file change"),
    ],
    metrics=[
        ("Checkpoint overhead per write", "< 15 ms p95"),
        ("Restore of a 100-file change", "< 2 s p95"),
        ("Storage per typical run", "< 5 MB"),
        ("Restore fidelity in the property suite", "100%"),
    ],
    exit=[
        "Every agent write is covered by a checkpoint",
        "Restore works per turn, per run, and per file with a preview",
        "Round-trip fidelity holds for binaries, renames, and unusual encodings",
        "Retention caps enforced with visible storage usage",
        "Crash recovery offers resume or revert",
    ],
    defer=[
        "Cloud or cross-machine checkpoints (post-GA)",
    ],
))

PHASES.append(dict(
    n=19, title="MCP",
    track="D — Intelligence", size="2 weeks", milestone="—",
    goal="Support Model Context Protocol servers as first-class, policy-governed tool sources "
         "with a trust model that matches their risk.",
    why="MCP is how the product extends without us writing every integration. It is also the "
        "largest new attack surface in the product, so it lands after policy, approvals, and "
        "sandboxing are mature.",
    scope=[
        "MCP client supporting stdio and HTTP/SSE transports",
        "Server lifecycle: spawn, health, restart with backoff, shutdown",
        "Tool discovery mapped into the tool registry with namespaced ids",
        "Manifest hashing, optional signature verification, and an approval registry",
        "Per-server permission scoping and org allowlists",
        "Resource and prompt support where the server provides it",
        "Sandboxing of local server processes",
        "Server management UI with logs, tool listing, and enable/disable",
    ],
    out=[
        "A public MCP marketplace (post-GA)",
        "Authoring MCP servers ourselves beyond examples",
    ],
    deliverables=[
        ("D19.1", "MCP client with both transports and robust framing", "[MCP integration](../04-specs/10-mcp-integration-spec.md)"),
        ("D19.2", "Server supervisor with restart backoff and health reporting", "[MCP integration](../04-specs/10-mcp-integration-spec.md)"),
        ("D19.3", "Namespaced tool registration (`mcp__<server>__<tool>`)", "[tool catalog](../04-specs/02-tool-catalog-spec.md)"),
        ("D19.4", "Manifest hash pinning and re-approval on change", "[admin console](../06-enterprise/06-admin-console-spec.md)"),
        ("D19.5", "Per-server policy scoping and org allowlist enforcement", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
        ("D19.6", "Local server process sandboxing", "[sandbox model](../03-security/02-sandbox-model.md)"),
        ("D19.7", "Server management UI with per-server logs", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D19.8", "MCP message parser fuzz target", "[security testing](../08-quality/04-security-testing.md)"),
    ],
    decisions=[
        "MCP tools are untrusted by default: their results are untrusted content, and their calls require the same approval discipline as any other tool class.",
        "Manifests are pinned by hash. A server that changes its tool surface requires re-approval rather than silently gaining capability.",
        "Local MCP servers are disabled by default for Enterprise policy presets.",
        "A misbehaving server is isolated and disabled, never allowed to hang a run.",
    ],
    contracts=[
        "MCP server configuration schema",
        "Namespaced tool id format",
        "MCP audit events",
    ],
    deps=["Phase 6 (tool registry)", "Phase 7 (policy and approvals)", "Phase 5 (sandbox)"],
    risks=[
        ("Malicious MCP server", "Data exfiltration or code execution", "Approval registry, hash pinning, sandboxing, network policy, untrusted results, and audit"),
        ("Prompt injection via MCP results", "Agent misdirection", "Results marked untrusted; privileged actions still require approval"),
        ("Server instability", "Hung or flaky runs", "Timeouts, health checks, backoff, and automatic disable after repeated failure"),
        ("Tool name collisions", "Wrong tool invoked", "Mandatory namespacing; collision detection at registration"),
    ],
    tests=[
        ("Unit", "Protocol framing, capability negotiation, error mapping"),
        ("Integration", "Reference servers over both transports; restart and failure paths"),
        ("Security", "Hostile server suite: oversized responses, injection payloads, unauthorized tool claims"),
        ("Fuzz", "MCP message parser"),
        ("Policy", "Org allowlist and local-server denial enforced"),
    ],
    metrics=[
        ("Server startup", "< 2 s p95"),
        ("Tool call overhead vs. native", "< 20 ms added p95"),
        ("Hostile server suite", "100% contained"),
        ("Unapproved server invocations", "0"),
    ],
    exit=[
        "MCP servers connect over stdio and HTTP/SSE and register namespaced tools",
        "Manifest hash pinning forces re-approval on change",
        "Local servers are sandboxed and policy-gated",
        "Hostile server suite fully contained",
        "Server management UI shows health, tools, and logs",
    ],
    defer=[
        "MCP marketplace and discovery (post-GA)",
    ],
))

PHASES.append(dict(
    n=20, title="Multi-Agent Orchestration",
    track="D — Intelligence", size="2–3 weeks", milestone="—",
    goal="Use sub-agents for genuinely parallel or specialized work — planner, implementer, "
         "tester, reviewer — with strict budgets, isolation, and a single accountable parent run.",
    why="Some tasks are naturally parallel (test three hypotheses, review a large change). "
        "Sub-agents make those faster, but they multiply cost and risk, so they land last in "
        "the intelligence track with everything else already governed "
        "([ADR-0010](../02-architecture/adr/0010-multi-agent-orchestration-model.md)).",
    scope=[
        "Sub-agent spawning through Cersei with role-specific prompts and tool subsets",
        "Parent/child run hierarchy with nested timelines in the UI",
        "Per-sub-agent budgets: turns, tokens, cost, wall clock, and tool classes",
        "Result aggregation and conflict handling when children touch the same files",
        "Write serialization: only one child may hold a write lease on a path",
        "Cancellation propagation from parent to all children",
        "Role presets: planner, implementer, tester, reviewer",
        "Feature flag and kill switch for the whole subsystem",
    ],
    out=[
        "Distributed or remote sub-agents",
        "User-authored custom roles (post-GA)",
    ],
    deliverables=[
        ("D20.1", "Sub-agent lifecycle with inherited-but-narrowable permissions", "[multi-agent](../04-specs/13-multi-agent-spec.md)"),
        ("D20.2", "Nested run timeline UI", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D20.3", "Budget enforcement per child and in aggregate", "[usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md)"),
        ("D20.4", "Write lease manager preventing concurrent writes to one path", "[multi-agent](../04-specs/13-multi-agent-spec.md)"),
        ("D20.5", "Result aggregation with explicit conflict reporting", "[multi-agent](../04-specs/13-multi-agent-spec.md)"),
        ("D20.6", "Role presets with distinct tool subsets", "[modes](../01-product/03-modes-and-workflows.md)"),
        ("D20.7", "`allowSubAgents` policy flag and server-side kill switch", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
    ],
    decisions=[
        "A child agent can never hold more permission than its parent, and may hold less.",
        "Approvals always surface to the human on the parent run; a child cannot approve its own request.",
        "Writes are serialized by path lease, because concurrent edits to one file is a correctness disaster, not a performance win.",
        "Sub-agents are off by default at GA and enabled per organization, since cost amplification is the main support risk.",
    ],
    contracts=[
        "Sub-agent spawn request and budget shape",
        "Nested run event envelope",
        "Aggregated result format",
    ],
    deps=["Phase 17 (plans to parallelize)", "Phase 11 (cost accounting)", "Phase 7 (approvals)"],
    risks=[
        ("Cost explosion", "Surprise bills", "Hard aggregate budgets, default off, visible live cost, and abort on cap"),
        ("Conflicting writes", "Corrupted work", "Path leases plus checkpoints per child; conflicts reported rather than merged"),
        ("Debuggability", "Users cannot follow what happened", "Nested timelines with per-child attribution and full transcripts"),
        ("Approval flooding", "Fatigue", "Approvals batched at the parent with the requesting child named"),
    ],
    tests=[
        ("Unit", "Budget accounting, lease acquisition, permission inheritance"),
        ("Integration", "Parallel children with overlapping file targets; cancellation propagation"),
        ("Eval", "Multi-agent task category compared against single-agent baseline for quality and cost"),
        ("Security", "A child cannot exceed parent permissions or self-approve"),
    ],
    metrics=[
        ("Speedup on parallelizable eval tasks", "≥ 1.5× vs. single agent"),
        ("Cost overhead", "< 2× single-agent for the same task"),
        ("Write conflicts reaching disk", "0"),
        ("Orphaned children after parent abort", "0"),
    ],
    exit=[
        "Sub-agents run with enforced budgets and narrowed permissions",
        "Write leases prevent concurrent writes to the same path",
        "Cancellation propagates to every child with no orphans",
        "Nested timeline makes each child's work attributable",
        "Multi-agent eval category shows measurable benefit without unsafe results",
    ],
    defer=[
        "Custom user-defined roles (post-GA)",
    ],
))

# ---------------------------------------------------------------- Track E

PHASES.append(dict(
    n=21, title="Background and Scheduled Work",
    track="E — Depth", size="2 weeks", milestone="—",
    goal="Let runs continue outside the foreground: queued runs, background execution on a "
         "separate worktree, notifications, and scheduled tasks.",
    why="Long tasks should not hold the UI hostage, and recurring maintenance work (dependency "
        "bumps, test triage) is a natural agent job. Requires git worktrees and checkpoints to "
        "be safe.",
    scope=[
        "Run queue with concurrency limits and priority",
        "Background runs on a dedicated git worktree so the user's working tree stays stable",
        "Progress in the status bar and the Runs surface; OS notifications on completion",
        "Scheduled runs (cron-like) with policy gating and a per-schedule budget",
        "Result review flow: diff the worktree result before merging into the main tree",
        "Resource governance: pause on battery, throttle under load, respect Do Not Disturb",
        "Approval handling for background runs: queue and notify, never auto-approve",
    ],
    out=[
        "Cloud-executed background runs",
        "Multi-machine scheduling",
    ],
    deliverables=[
        ("D21.1", "Run queue with limits, priority, and fair scheduling", "[agent runtime](../04-specs/01-agent-runtime-spec.md)"),
        ("D21.2", "Worktree-isolated background execution", "[feature catalog](../01-product/02-feature-catalog.md)"),
        ("D21.3", "Runs surface with live progress and history", "[app shell](../05-ui/02-app-shell-and-navigation.md)"),
        ("D21.4", "Scheduler with policy gating and per-schedule budgets", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
        ("D21.5", "Background result review and merge-into-main-tree flow", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D21.6", "OS notifications that never include file contents", "[approval UX](../05-ui/05-approval-and-trust-ux.md)"),
        ("D21.7", "Resource governance: battery, load, and DND awareness", "[latency budget](../02-architecture/05-latency-budget.md)"),
    ],
    decisions=[
        "Background runs never write to the user's working tree; they use a worktree and produce a reviewable result.",
        "Scheduled runs never auto-approve anything. A schedule that would need approval pauses and notifies.",
        "Notifications carry the tool name and run title only — never payloads.",
        "Background work yields to foreground work for CPU and model concurrency.",
    ],
    contracts=[
        "Run queue and schedule record schemas",
        "Background run result envelope",
    ],
    deps=["Phase 14 (git worktrees)", "Phase 18 (checkpoints)", "Phase 17 (plans as schedulable units)"],
    risks=[
        ("Unattended runs consuming budget", "Cost surprise", "Per-schedule budgets, global caps, and visible spend attribution per schedule"),
        ("Worktree merge conflicts", "User frustration", "Explicit review-and-merge step with conflict presentation; never auto-merge"),
        ("Background runs blocking on approval forever", "Wasted work", "Approval timeout with a resumable pause and clear notification"),
        ("Resource contention", "Sluggish machine", "Throttling, battery awareness, and a hard concurrency cap"),
    ],
    tests=[
        ("Unit", "Queue ordering, schedule parsing, budget accounting"),
        ("Integration", "Background run in a worktree, review, merge; conflicting-change case"),
        ("Security", "Scheduled run cannot auto-approve; policy gating enforced"),
        ("Endurance", "24-hour scheduler soak with no leaks"),
    ],
    metrics=[
        ("Foreground latency impact while a background run is active", "< 10%"),
        ("Scheduled run reliability over 7 days", "> 99%"),
        ("Auto-approvals in background runs", "0"),
    ],
    exit=[
        "Runs can be queued and executed in the background on a worktree",
        "Background results are reviewed before touching the working tree",
        "Scheduled runs respect policy and budgets and never auto-approve",
        "Resource governance keeps the foreground responsive",
    ],
    defer=[
        "Cloud-executed runs (post-GA)",
    ],
))

PHASES.append(dict(
    n=22, title="Preview Runtime",
    track="E — Depth", size="1–2 weeks", milestone="—",
    goal="An isolated preview panel for local dev servers and static output, with a safe path "
         "for feeding what it shows back to the agent.",
    why="Front-end work needs a visible result. Preview is straightforward except for isolation, "
        "which is why it comes after the trust and untrusted-content model is settled.",
    scope=[
        "Preview webview restricted to localhost and workspace `file://` origins",
        "Full isolation: separate session, no app IPC access, no shared storage",
        "Port detection from tool output with a suggestion to open preview",
        "Controls: reload, hard reload, validated address entry, device-width presets, open in browser",
        "Console error and warning capture, surfaced in the Activity panel",
        "Screenshot capture available to the agent only after approval",
        "Explicit sharing of console output or DOM extracts as untrusted content",
    ],
    out=[
        "Remote URL previews",
        "Full browser automation (post-GA)",
    ],
    deliverables=[
        ("D22.1", "Isolated preview webview with an origin allowlist", "[editor/terminal/preview](../05-ui/06-editor-terminal-preview.md)"),
        ("D22.2", "Port detection and preview suggestion", "[tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md)"),
        ("D22.3", "Preview controls and device-width presets", "[editor/terminal/preview](../05-ui/06-editor-terminal-preview.md)"),
        ("D22.4", "Console capture with untrusted-content marking", "[injection defense](../03-security/05-prompt-injection-defense.md)"),
        ("D22.5", "Approval-gated screenshot capture", "[approval UX](../05-ui/05-approval-and-trust-ux.md)"),
        ("D22.6", "`allowScreenshots` policy flag support", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
    ],
    decisions=[
        "The preview is untrusted content by definition. Nothing it produces can enter a prompt without being marked and, for screenshots, approved.",
        "Only localhost and workspace files are allowed; anything else opens in the real browser.",
        "The preview webview has no bridge to app IPC, so a malicious page cannot reach our commands.",
        "DevTools for the preview exist in dev builds only.",
    ],
    contracts=[
        "Preview IPC commands",
        "Console capture event shape",
    ],
    deps=["Phase 3 (pane shell)", "Phase 12 (detecting server output)"],
    risks=[
        ("Malicious page escaping isolation", "Compromise", "Separate session, no IPC bridge, strict origin allowlist, CSP, and a hostile-page test suite"),
        ("Injection via console output", "Agent misdirection", "Marked untrusted; sharing is explicit"),
        ("Screenshot leaking sensitive content", "Privacy", "Approval required, policy flag, and no automatic capture"),
        ("Memory from an extra webview", "Bloat", "Preview webview created lazily and destroyed on tab close; measured in the memory benchmark"),
    ],
    tests=[
        ("Integration", "Dev servers for React, Vite, and a static site"),
        ("Security", "Hostile page cannot reach IPC, storage, or non-allowlisted origins"),
        ("Policy", "`allowScreenshots = false` blocks capture"),
        ("Performance", "Preview open/close memory delta"),
    ],
    metrics=[
        ("Preview overhead on first paint", "< 100 ms"),
        ("Memory per open preview", "< 120 MB"),
        ("Isolation suite", "100% contained"),
    ],
    exit=[
        "Preview works for local dev servers and static files",
        "Isolation suite proves no IPC or storage access from the previewed page",
        "Console capture and screenshots are explicit and policy-gated",
        "Memory returns to baseline after closing a preview",
    ],
    defer=[
        "Browser automation for the agent (post-GA)",
    ],
))

PHASES.append(dict(
    n=23, title="Cloud Sync and Collaboration",
    track="E — Depth", size="2 weeks", milestone="M3 RC",
    goal="Optional, off-by-default sync of conversations and memory across a user's devices, "
         "plus shareable run transcripts. Reach release candidate.",
    why="Users with multiple machines expect continuity, but sync moves content to the server, "
        "so it must be opt-in, policy-gated, and clearly explained. It comes last in the depth "
        "track for exactly that reason.",
    scope=[
        "`/agent/v1/sync/*` endpoints with cursor-based delta sync",
        "Conflict resolution: last-write-wins per message, vector clock per conversation, conflicts surfaced",
        "Selective sync: choose which workspaces and whether memory is included",
        "Policy gating via `allowCloudSync` and residency routing",
        "Shareable, redacted run transcripts with an expiring link",
        "Clear data-handling disclosure in the UI at the moment of opt-in",
        "Sync status, last-synced time, and a manual sync trigger",
        "RC hardening pass: bug burn-down across all prior phases",
    ],
    out=[
        "Real-time multi-user collaboration",
        "Shared live sessions",
    ],
    deliverables=[
        ("D23.1", "Sync endpoints with cursors and idempotent upserts", "[cloud API](../04-specs/06-cloud-api-contract.md)"),
        ("D23.2", "`AgentSyncCursor` per device and resource", "[cloud data model](../04-specs/07-cloud-data-model.md)"),
        ("D23.3", "Conflict handling with user-visible resolution", "[offline and degraded modes](../02-architecture/06-offline-and-degraded-modes.md)"),
        ("D23.4", "Selective sync settings with an explicit disclosure screen", "[privacy](../03-security/07-privacy-and-data-residency.md)"),
        ("D23.5", "Policy-gated sync with residency routing", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
        ("D23.6", "Redacted, expiring shareable transcripts", "[privacy](../03-security/07-privacy-and-data-residency.md)"),
        ("D23.7", "RC bug burn-down and regression sweep", "[definition of done](../08-quality/05-definition-of-done.md)"),
    ],
    decisions=[
        "Sync is off by default and never enabled by an update. Turning it on is an explicit, informed action.",
        "Conversation content syncs into the existing platform tables via the Agent module; no new content tables are introduced.",
        "Conflicts are surfaced, not silently resolved, when two devices edited the same conversation.",
        "Shared transcripts are redacted by the same pipeline as audit payloads and expire by default.",
    ],
    contracts=[
        "Sync request/response and cursor semantics",
        "Transcript share payload and redaction rules",
    ],
    deps=["Phase 10 (identity)", "Phase 9 (local store)", "Phase 16 (memory to sync)"],
    risks=[
        ("Accidental content upload", "Privacy incident", "Off by default, explicit disclosure, policy override, and a test asserting no network calls when disabled"),
        ("Sync conflicts losing messages", "Data loss", "Append-only message model with conflict surfacing; property tests over interleaved edits"),
        ("Residency violations", "Compliance breach", "Residency is org-level and cannot be overridden downstream; routing tested per region"),
        ("Sync amplifying storage cost", "Operational cost", "Per-plan sync quotas and pruning"),
    ],
    tests=[
        ("Unit", "Cursor advancement, conflict detection, redaction"),
        ("Integration", "Two-device sync, offline edits on both, reconciliation"),
        ("Property", "Interleaved edits never drop a message"),
        ("Security", "Sync disabled means zero content leaves the machine"),
        ("Policy", "`allowCloudSync = false` disables the feature entirely"),
    ],
    metrics=[
        ("Sync round trip (100 messages)", "< 2 s p95"),
        ("Content leaving the device with sync off", "0 bytes"),
        ("Message loss in the interleaving property suite", "0"),
    ],
    exit=[
        "Two devices converge on the same conversation state",
        "Sync is provably inert when disabled",
        "Conflicts are surfaced with a user-resolvable choice",
        "Policy and residency controls enforced",
        "RC regression sweep complete",
        "M3 RC milestone declared",
    ],
    defer=[
        "Real-time collaboration (post-GA)",
    ],
))

# ---------------------------------------------------------------- Track F

PHASES.append(dict(
    n=24, title="Enterprise Identity and Governance",
    track="F — Enterprise and GA", size="2–3 weeks", milestone="—",
    goal="SSO, SCIM, personal access tokens, RBAC surfacing, and cloud policy distribution with "
         "signing — all inside the Agent module.",
    why="These are the requirements that gate enterprise deals. They come after the product works, "
        "because governance over a moving target is wasted effort.",
    scope=[
        "SAML 2.0 and OIDC login with domain discovery and enforcement",
        "SCIM 2.0 Users and Groups with complete deprovisioning",
        "Personal access tokens for headless and air-gapped use",
        "Agent permission derivation surfaced through `/agent/v1/me`",
        "Team-scoped policies and the read-only auditor role",
        "Cloud policy distribution with ETags, signing, and version monotonicity",
        "Admin pages: policies, identity, devices",
        "Strict revocation mode for instant access cutoff",
    ],
    out=[
        "Quotas and budgets (Phase 25)",
        "SIEM export (Phase 25)",
    ],
    deliverables=[
        ("D24.1", "SAML and OIDC flows bound to the device-code session", "[identity](../06-enterprise/01-identity-sso-scim.md)"),
        ("D24.2", "Domain discovery and SSO enforcement with DNS verification", "[identity](../06-enterprise/01-identity-sso-scim.md)"),
        ("D24.3", "SCIM endpoints with idempotent creates and full deprovisioning", "[identity](../06-enterprise/01-identity-sso-scim.md)"),
        ("D24.4", "Personal access tokens with scopes and revocation", "[identity](../06-enterprise/01-identity-sso-scim.md)"),
        ("D24.5", "Permission resolution returned explicitly to the client", "[RBAC](../06-enterprise/02-rbac-and-org-model.md)"),
        ("D24.6", "Policy distribution endpoint with ETag, signature, and monotonic versions", "[workspace policies](../06-enterprise/03-workspace-policies.md)"),
        ("D24.7", "Admin policy, identity, and device pages", "[admin console](../06-enterprise/06-admin-console-spec.md)"),
        ("D24.8", "Strict revocation mode with a configurable offline window", "[RBAC](../06-enterprise/02-rbac-and-org-model.md)"),
    ],
    decisions=[
        "SSO lives in the Agent module with its own session binding; the desktop never renders an IdP form.",
        "Deprovisioning completes within one request: refresh chains revoked, devices marked revoked, membership deactivated.",
        "Policies are signed for enterprises, and a lower version is rejected so an attacker cannot replay a permissive policy.",
        "Client-side permission enforcement is for UX; the server re-checks everything.",
    ],
    contracts=[
        "SSO discovery response",
        "SCIM subset per RFC 7644",
        "Policy distribution response with signature",
        "Personal access token format and scopes",
    ],
    deps=["Phase 10 (auth foundation)", "Phase 7 (local policy engine)"],
    risks=[
        ("IdP variation", "Integration failures", "Test matrix across Okta, Entra ID, Google, Keycloak; strict standards compliance"),
        ("Deprovisioning incomplete", "Access after offboarding", "Single-transaction revocation, audit event, and a residual-access test asserting the 15-minute ceiling"),
        ("Policy signing key handling", "Trust failure", "HSM storage, rotation planned a release ahead, verification test in the client"),
        ("SCIM surface exposed unintentionally", "Attack surface", "Routes 404 without a configured token"),
    ],
    tests=[
        ("Unit", "Assertion validation, attribute mapping, SCIM patch semantics"),
        ("Integration", "Full SSO login per IdP against test tenants"),
        ("Security", "Unsigned and replayed assertions rejected; state bound to device code; deprovision revokes everything"),
        ("Contract", "SCIM conformance subset; policy distribution ETag behavior"),
    ],
    metrics=[
        ("SSO login completion", "< 45 s median"),
        ("Deprovision to token revocation", "< 1 s"),
        ("Residual access after deprovision", "≤ 15 min (0 in strict mode)"),
        ("Policy fetch latency (cached)", "< 30 ms p95"),
    ],
    exit=[
        "SSO works against at least three IdPs",
        "SCIM provisioning and deprovisioning verified end to end",
        "Policy distribution works with ETags, signing, and downgrade rejection",
        "Strict revocation mode cuts access immediately",
        "Admin policy, identity, and device pages functional",
    ],
    defer=[
        "Multi-org membership (open question Q05)",
    ],
))

PHASES.append(dict(
    n=25, title="Policy, Audit, Quotas",
    track="F — Enterprise and GA", size="2–3 weeks", milestone="—",
    goal="Complete the enterprise control loop: cloud audit ingest with integrity verification, "
         "SIEM export, retention, quota and budget enforcement, and policy dry-run.",
    why="Governance is only credible when the evidence trail is verifiable and the spend controls "
        "actually stop spend. This phase turns the design from Phases 7 and 24 into an auditable system.",
    scope=[
        "`POST /agent/v1/audit/batch` with signature verification and idempotency",
        "Hash-chain verification tooling and gap detection alerts",
        "Retention tiers, nightly purge, legal hold, and archive export",
        "SIEM delivery: webhook, HEC, and bucket pull, with OCSF mapping",
        "Audit search, filtering, and queued large exports in the admin console",
        "`AgentUsageGuard`: seat, concurrency, daily, monthly, and budget enforcement with reservations",
        "Usage dashboards, budget controls, and cost reconciliation",
        "Policy dry-run against recent run metadata",
        "Module isolation check enforced as a release gate",
    ],
    out=[
        "Billing invoicing changes",
        "Custom compliance reports (post-GA)",
    ],
    deliverables=[
        ("D25.1", "Audit ingest with Ed25519 verification and duplicate handling", "[audit and retention](../06-enterprise/04-audit-and-retention.md)"),
        ("D25.2", "Chain verification per device with an evidence-quality report", "[audit and retention](../06-enterprise/04-audit-and-retention.md)"),
        ("D25.3", "Retention purge, legal hold, and archive to object storage", "[audit and retention](../06-enterprise/04-audit-and-retention.md)"),
        ("D25.4", "SIEM delivery in three modes with OCSF option", "[audit and retention](../06-enterprise/04-audit-and-retention.md)"),
        ("D25.5", "Quota and budget guard with token reservations", "[usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md)"),
        ("D25.6", "Usage dashboards and budget administration", "[admin console](../06-enterprise/06-admin-console-spec.md)"),
        ("D25.7", "Monthly cost reconciliation job with a drift alert", "[usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md)"),
        ("D25.8", "Policy dry-run analysis over recent runs", "[admin console](../06-enterprise/06-admin-console-spec.md)"),
        ("D25.9", "`dailyManagedRuns` enforcement implemented in the Agent module, with legacy seed translation at its boundary", "[usage and quotas](../06-enterprise/05-usage-quotas-and-budgets.md)"),
    ],
    decisions=[
        "Audit is append-only with no update or delete path; purge is the only deletion and is itself logged.",
        "A detected chain gap is a security event at critical severity, not a warning.",
        "Quota checks never fail open on a hard limit, even when Redis is down.",
        "In-flight runs are never killed by a quota boundary; overshoot is bounded by the reservation instead.",
        "The legacy plan limit that was never enforced is implemented here, in the Agent module, not by patching billing code.",
    ],
    contracts=[
        "Audit batch request/response and error codes",
        "SIEM payload formats",
        "Usage response shape and quota error details",
    ],
    deps=["Phase 7 (local audit queue)", "Phase 24 (policy distribution)", "Phase 11 (cost accounting)"],
    risks=[
        ("Audit ingest as a hot path", "Latency and cost", "Batching, idempotency, async archive, and load testing at 10× projected volume"),
        ("Quota races", "Limits exceeded", "Atomic Redis operations with a Lua script; concurrency race tests"),
        ("Purge deleting too much", "Compliance breach", "Dry-run counts before deletion, legal hold checks, and purge summaries"),
        ("Money math errors", "Billing disputes", "Integer micro-USD, versioned price table, and monthly reconciliation with alerting"),
    ],
    tests=[
        ("Unit", "Signature verification, chain verification, retention selection, quota math"),
        ("Contract", "Audit and usage endpoints against golden fixtures"),
        ("Concurrency", "Parallel requests cannot exceed a daily quota"),
        ("Security", "Tampered, replayed, and wrongly signed batches rejected; cross-org access denied"),
        ("Load", "10× projected audit volume; gateway guard chain under load"),
    ],
    metrics=[
        ("Audit batch (500 events)", "< 300 ms p95"),
        ("Guard chain overhead", "< 10 ms p95"),
        ("Quota overshoot under concurrency", "0"),
        ("Cost reconciliation drift", "< 2%"),
        ("Chain verification on 1M events", "< 60 s"),
    ],
    exit=[
        "Audit batches ingest with verified signatures and idempotency",
        "Chain verification detects tampering and produces an evidence report",
        "Retention, legal hold, and archive work with logged purges",
        "SIEM delivery works in all three modes",
        "Quotas and budgets enforce without races and without failing open",
        "Policy dry-run produces accurate would-have-blocked counts",
        "Module isolation check is a release gate",
    ],
    defer=[
        "Customer-specific compliance reports (post-GA)",
    ],
))

PHASES.append(dict(
    n=26, title="Observability and Reliability",
    track="F — Enterprise and GA", size="2 weeks", milestone="—",
    goal="Instrument everything: traces, metrics, structured logs, opt-in desktop telemetry, "
         "crash reporting, dashboards, alerts, and the runbook-per-alert rule.",
    why="Before GA we must be able to see and diagnose production. An SLO without instrumentation "
        "is a wish, and an alert without a runbook is a page nobody can action.",
    scope=[
        "OpenTelemetry tracing across every `/agent/v1/*` route with the defined span tree",
        "Metric set with cardinality guards and no per-user labels",
        "Structured cloud logging with enforced redaction",
        "Opt-in desktop telemetry with a transparent event viewer",
        "Crash reporting with symbolization and path scrubbing",
        "Dashboards for gateway, auth, quota, policy, audit, and release health",
        "Alerts with severities, routing, and a runbook link per alert",
        "Error-budget tracking and burn alerts",
        "`--export-logs` diagnostic bundle for support",
        "Correlated request ids surfaced in desktop error dialogs",
    ],
    out=[
        "Public status page automation (Phase 29)",
    ],
    deliverables=[
        ("D26.1", "Tracing with the documented span tree and attributes", "[observability](../07-ops/05-observability-and-slos.md)"),
        ("D26.2", "Metrics with cardinality guards", "[observability](../07-ops/05-observability-and-slos.md)"),
        ("D26.3", "Redaction-enforced structured logging with a unit test on secret shapes", "[telemetry spec](../04-specs/14-telemetry-spec.md)"),
        ("D26.4", "Opt-in desktop telemetry with a live sample viewer in Settings", "[telemetry spec](../04-specs/14-telemetry-spec.md)"),
        ("D26.5", "Crash reporting with symbolization and scrubbing", "[observability](../07-ops/05-observability-and-slos.md)"),
        ("D26.6", "Seven dashboards covering every SLI", "[observability](../07-ops/05-observability-and-slos.md)"),
        ("D26.7", "Alert rules with runbook links and error-budget burn alerts", "[runbooks](../07-ops/06-runbooks.md)"),
        ("D26.8", "`--export-logs` redacted diagnostic bundle", "[environments and config](../07-ops/01-environments-and-config.md)"),
        ("D26.9", "Request-id correlation from desktop dialog to cloud trace", "[error taxonomy](../04-specs/15-error-taxonomy.md)"),
    ],
    decisions=[
        "Telemetry is opt-in and transparent: the user can see the exact payload we would send.",
        "No metric is labeled by user or device id; per-user analysis belongs in the audit tables.",
        "Every alert links to a runbook entry; an alert without one is not allowed to page.",
        "Redaction is a shared, tested utility rather than a per-call-site habit.",
    ],
    contracts=[
        "Telemetry event schema and allowed event list",
        "Trace attribute names",
        "Metric names and labels",
    ],
    deps=["Phase 25 (surfaces to observe)", "Phase 11 (gateway)", "Phase 10 (auth)"],
    risks=[
        ("Telemetry leaking content", "Privacy incident", "Allowlisted event fields, redaction tests, and a fuzz corpus asserting no secret survives"),
        ("Metric cardinality explosion", "Cost and slow queries", "Cardinality guard in review and a CI check on label sets"),
        ("Alert fatigue", "Real incidents missed", "Severity discipline, tuned thresholds, and a quarterly alert review"),
        ("Trace overhead", "Latency", "Sampling with always-on error capture; overhead measured in the benchmark"),
    ],
    tests=[
        ("Unit", "Redaction utility against a secret corpus; event schema validation"),
        ("Integration", "End-to-end trace continuity from desktop request id to cloud spans"),
        ("Security", "No content or secret in logs, metrics, telemetry, or crash reports"),
        ("Ops", "Every alert fires in a staged failure and its runbook resolves it"),
    ],
    metrics=[
        ("Tracing overhead", "< 5 ms p95 per request"),
        ("Alerts without a runbook link", "0"),
        ("SLI dashboard coverage", "100% of defined SLOs"),
        ("Secrets found in telemetry corpus test", "0"),
    ],
    exit=[
        "Every SLI is instrumented and dashboarded",
        "Every alert has a tested runbook",
        "Desktop telemetry is opt-in with a visible payload sample",
        "Redaction tests pass across logs, metrics, telemetry, and crash reports",
        "Request-id correlation works end to end",
        "Error-budget burn alerts configured",
    ],
    defer=[
        "Status page automation (Phase 29)",
    ],
))

PHASES.append(dict(
    n=27, title="Packaging and Release Engineering",
    track="F — Enterprise and GA", size="2–3 weeks", milestone="—",
    goal="Signed, notarized installers on all platforms, a verified auto-update channel with "
         "rollback, release administration, and the enterprise deployment package.",
    why="A desktop app that cannot be trusted to install and update itself has no distribution. "
        "This is also where the air-gapped and self-hosted offerings get packaged.",
    scope=[
        "Full `agent-release.yml` pipeline: build, sign, notarize, SBOM, provenance, publish",
        "Signed update manifests with a root/channel key hierarchy",
        "Auto-update with integrity verification, safe-point apply, and health check",
        "Revert-to-previous-version with a pre-migration database snapshot",
        "Release admin pages: publish, activate, staged rollout, rollback, health",
        "Channels: stable, beta, canary with server-side rollout bucketing",
        "Air-gapped build profile with an egress verification report",
        "Self-hosted container image, `--verify-config` mode, and deployment guide",
        "Offline licensing with grace and read-only fallback",
    ],
    out=[
        "App store distribution",
        "Linux `.rpm` (deferred)",
    ],
    deliverables=[
        ("D27.1", "Release pipeline producing verified artifacts for all platforms", "[CI/CD](../07-ops/02-ci-cd-pipelines.md)"),
        ("D27.2", "Notarized macOS builds, Authenticode Windows builds, signed Linux artifacts", "[packaging](../07-ops/03-packaging-signing-notarization.md)"),
        ("D27.3", "Signed update manifests with an embedded channel public key", "[auto-update](../07-ops/04-auto-update-and-rollback.md)"),
        ("D27.4", "Auto-update with verification, health check, and auto safe mode", "[auto-update](../07-ops/04-auto-update-and-rollback.md)"),
        ("D27.5", "Revert path with database snapshot restore", "[auto-update](../07-ops/04-auto-update-and-rollback.md)"),
        ("D27.6", "Release admin pages with staged rollout and one-click rollback", "[admin console](../06-enterprise/06-admin-console-spec.md)"),
        ("D27.7", "SBOM, checksums, and provenance attached to every release", "[supply chain](../03-security/06-supply-chain-and-signing.md)"),
        ("D27.8", "Air-gapped profile with a CI egress report", "[air-gapped](../06-enterprise/07-air-gapped-and-self-hosted.md)"),
        ("D27.9", "Self-hosted image, config verification mode, and deployment guide", "[air-gapped](../06-enterprise/07-air-gapped-and-self-hosted.md)"),
        ("D27.10", "Offline license verification with grace and read-only fallback", "[air-gapped](../06-enterprise/07-air-gapped-and-self-hosted.md)"),
    ],
    decisions=[
        "The update manifest is signed independently of the artifacts, so a CDN compromise alone cannot push an update.",
        "CI never activates a stable release; activation is a deliberate human step after smoke tests.",
        "Downgrading across a schema migration without a snapshot is refused rather than risking corruption.",
        "The air-gapped claim is backed by a CI egress test, not by a promise in a document.",
    ],
    contracts=[
        "Update manifest format and signature scheme",
        "Release admin API",
        "License file format",
    ],
    deps=["Phase 0 (version consistency)", "Phase 26 (release health telemetry)", "Phase 24 (admin auth)"],
    risks=[
        ("Signing or notarization failures", "Cannot ship", "Pipeline exercised on every canary; credentials monitored for expiry with advance alerts"),
        ("Bad update bricking installs", "Severe", "Health check, auto safe mode, revert path, staged rollout, and a tested rollback"),
        ("Key compromise", "Trust loss", "HSM custody, split backup, rotation planned a release ahead, and RB-11"),
        ("macOS identity change invalidating keychain items", "Users must re-login", "Identity pinned per channel; rotation documented in release notes"),
    ],
    tests=[
        ("Pipeline", "Full release dry-run on a canary tag"),
        ("Integrity", "Tampered artifact and tampered manifest both rejected"),
        ("Update", "N−1 → N on all platforms; interrupted download and interrupted apply"),
        ("Rollback", "Revert restores the previous version and its snapshot"),
        ("Air-gapped", "Egress test shows no connection outside the configured endpoint"),
        ("Self-hosted", "Container boots with `--verify-config` and passes smoke tests"),
    ],
    metrics=[
        ("Release pipeline duration", "< 60 min end to end"),
        ("Update success rate in test fleet", "> 99%"),
        ("Installer size per platform", "< 40 MB"),
        ("Unverified artifacts published", "0"),
    ],
    exit=[
        "Signed, notarized installers produced for all platforms by CI",
        "Auto-update verified end to end including integrity rejection cases",
        "Revert restores both the app and its data snapshot",
        "Staged rollout and one-click rollback work from the admin console",
        "SBOM, checksums, and provenance published per release",
        "Air-gapped egress report produced; self-hosted image boots and passes smoke tests",
    ],
    defer=[
        "App store distribution (post-GA)",
        "Delta updates (post-GA)",
    ],
))

PHASES.append(dict(
    n=28, title="Quality and Evaluation",
    track="F — Enterprise and GA", size="2–3 weeks", milestone="—",
    goal="Bring quality infrastructure to GA standard: the full eval suite with gates, the "
         "complete E2E matrix, benchmark baselines, accessibility verification, and an external "
         "penetration test.",
    why="This is the phase where we find out whether the product is actually good, using our own "
        "measurements rather than impressions. Everything it gates on must already exist.",
    scope=[
        "Eval suite expanded to 80+ tasks across all categories",
        "LLM-judge rubrics with fixed judge versions and multi-sample scoring",
        "Nightly eval in CI with gating thresholds and per-model qualification",
        "E2E suite: ten flows across three platforms, including a keyboard-only pass",
        "Benchmark baselines committed per platform with a regression gate",
        "Load, spike, and soak tests for cloud and desktop",
        "Accessibility verification: automated gates plus manual screen-reader passes",
        "External penetration test of desktop, cloud, and pipeline, with remediation",
        "Independent verification of vendor performance claims in `bench/REPORT.md`",
        "Flaky-test quarantine process and a test-health dashboard",
    ],
    out=[
        "New product features",
    ],
    deliverables=[
        ("D28.1", "80+ eval tasks with assertions and rubrics", "[eval harness](../08-quality/02-agent-eval-harness.md)"),
        ("D28.2", "Nightly eval with gates on pass rate, unsafe count, and cost", "[eval harness](../08-quality/02-agent-eval-harness.md)"),
        ("D28.3", "Ten E2E flows green on macOS, Windows, and Linux", "[test strategy](../08-quality/01-test-strategy.md)"),
        ("D28.4", "Committed benchmark baselines and a CI regression gate", "[benchmarks](../08-quality/03-performance-benchmarks.md)"),
        ("D28.5", "Load, spike, and 8-hour soak suites", "[benchmarks](../08-quality/03-performance-benchmarks.md)"),
        ("D28.6", "WCAG 2.2 AA verification report including manual passes", "[accessibility](../05-ui/07-accessibility-and-i18n.md)"),
        ("D28.7", "External pentest report with all critical and high findings closed", "[security testing](../08-quality/04-security-testing.md)"),
        ("D28.8", "`bench/REPORT.md` with independently measured vendor claims", "[benchmarks](../08-quality/03-performance-benchmarks.md)"),
        ("D28.9", "Test-health dashboard and quarantine process", "[test strategy](../08-quality/01-test-strategy.md)"),
    ],
    decisions=[
        "An unsafe eval result blocks release regardless of pass rate. Safety and quality are scored separately and never traded.",
        "Vendor performance claims are never repeated externally without a matching entry in our own report.",
        "Flaky tests are quarantined with an owner and a deadline, never retried into green.",
        "Eval assertions are never weakened to make a release pass; the release waits instead.",
    ],
    contracts=[
        "Eval task and result schemas",
        "Benchmark baseline format",
    ],
    deps=["Phase 20 (features to evaluate)", "Phase 27 (releasable artifacts to test)", "Phase 26 (instrumentation)"],
    risks=[
        ("Eval cost", "Budget overrun", "Spend caps, cassette replay for parser-level checks, and tiered run frequency"),
        ("Eval nondeterminism", "Noisy gates", "Low temperature, multi-sample for borderline tasks, and trend-based judgment"),
        ("Pentest findings late", "Delayed GA", "Pentest scheduled at the start of the phase with buffer for remediation and retest"),
        ("Benchmark noise in CI", "False failures", "Runner variance measured and used as the noise allowance; reference hardware for published numbers"),
    ],
    tests=[
        ("Meta", "Harness correctness verified with the stub provider"),
        ("E2E", "Ten flows on three platforms, plus keyboard-only"),
        ("Performance", "Full benchmark suite against baselines"),
        ("Security", "Pentest plus the complete per-release security checklist"),
        ("A11y", "Automated gates plus VoiceOver, NVDA, and Orca passes"),
    ],
    metrics=[
        ("Eval pass rate (primary model)", "≥ 85%"),
        ("Unsafe eval results", "0"),
        ("Injection and tool-discipline categories", "100%"),
        ("E2E flake rate", "< 1%"),
        ("Critical/high pentest findings open", "0"),
    ],
    exit=[
        "Eval suite ≥ 80 tasks with all gates met on every supported model",
        "E2E suite green on all three platforms including keyboard-only",
        "Benchmark baselines committed with the regression gate active",
        "WCAG 2.2 AA verified with manual screen-reader passes",
        "Pentest complete with critical and high findings closed and retested",
        "`bench/REPORT.md` published with our own measurements",
    ],
    defer=[
        "Bug bounty program (Phase 29 / post-GA)",
    ],
))

PHASES.append(dict(
    n=29, title="GA Launch",
    track="F — Enterprise and GA", size="2 weeks", milestone="M4 GA",
    goal="Ship 1.0: documentation, migration path off the deprecated app, support readiness, "
         "pricing live, legacy sunset communicated, and a staged public rollout.",
    why="Launch is an operational project, not a build step. Everything technical is done; this "
        "phase makes it supportable, purchasable, and survivable.",
    scope=[
        "Public documentation site generated from these specs, with a getting-started path",
        "Migration guide from `code-old`, tested by someone who did not write it",
        "Marketing download page at `/{locale}/eury-agent` with checksums",
        "Pricing and checkout live for all tiers with entitlement verification",
        "Support readiness: runbooks, on-call rotation, escalation, game-day exercise",
        "Status page automation wired to SLO alerts",
        "Legacy sunset announcement with dates and deprecation headers on `/code/*`",
        "Staged public rollout with go/no-go criteria at each step",
        "Post-launch review, roadmap for 1.1, and a bug bounty program",
    ],
    out=[
        "New features",
    ],
    deliverables=[
        ("D29.1", "Public docs site with getting started, guides, and reference", "[doc conventions](../00-overview/04-doc-conventions.md)"),
        ("D29.2", "Tested migration guide from the deprecated app", "[naming map](../00-overview/05-naming-and-migration-map.md)"),
        ("D29.3", "Download page with per-platform artifacts and verifiable checksums", "[packaging](../07-ops/03-packaging-signing-notarization.md)"),
        ("D29.4", "Pricing and checkout live with entitlements enforced end to end", "[pricing](../01-product/04-pricing-and-packaging.md)"),
        ("D29.5", "Support runbooks, rotation, and a completed game-day", "[runbooks](../07-ops/06-runbooks.md)"),
        ("D29.6", "Status page automation driven by SLO alerts", "[observability](../07-ops/05-observability-and-slos.md)"),
        ("D29.7", "Legacy sunset announcement and deprecation headers on `/code/*`", "[naming map](../00-overview/05-naming-and-migration-map.md)"),
        ("D29.8", "Staged rollout executed with documented go/no-go decisions", "[release management](../07-ops/08-release-management.md)"),
        ("D29.9", "Security disclosure policy and bug bounty launch", "[incident response](../03-security/09-incident-response.md)"),
        ("D29.10", "Post-launch review and the 1.1 roadmap", "[roadmap overview](00-roadmap-overview.md)"),
    ],
    decisions=[
        "Docs are generated from these specs so documentation drift is a build failure rather than a discovery.",
        "The migration guide is validated by someone uninvolved in writing it — self-tested instructions are not tested.",
        "Legacy `/code/*` keeps working for six months after GA; the sunset date is announced at launch, not later.",
        "Rollout is staged with explicit go/no-go criteria, and stopping is a normal outcome rather than a failure.",
    ],
    contracts=[
        "Public documentation URLs",
        "Download page artifact naming",
        "Deprecation header format on legacy endpoints",
    ],
    deps=["Phase 27 (signed releases)", "Phase 28 (quality gates)", "Phase 25 (billing controls)"],
    risks=[
        ("Launch-day load", "Outage", "Load tested at 2× projected peak; staged rollout; scaling headroom pre-provisioned"),
        ("Support volume", "Slow responses", "Runbooks, diagnostic bundle command, and a triaged FAQ from beta feedback"),
        ("Migration friction", "Users stranded on the old app", "Tested guide, export/import path, and six months of overlap"),
        ("Pricing or entitlement bugs", "Revenue and trust", "End-to-end entitlement tests per tier before launch"),
        ("Security report on day one", "Reputation", "Disclosure policy, bounty, and an incident process rehearsed in the game-day"),
    ],
    tests=[
        ("End to end", "Purchase → install → login → first successful run, per tier"),
        ("Migration", "Deprecated app data exported and the new app set up following only the written guide"),
        ("Load", "2× projected launch peak on auth, manifest, and gateway"),
        ("Ops", "Game-day covering a gateway outage, a bad release, and a security report"),
    ],
    metrics=[
        ("First-run success rate", "> 95%"),
        ("Install to first successful run", "< 5 min median"),
        ("Launch-week SEV1 count", "0"),
        ("Support contacts per 1000 active users", "< 20"),
        ("Crash-free sessions", "> 99.5%"),
    ],
    exit=[
        "Docs site published and accurate against shipped behavior",
        "Migration guide validated by an independent tester",
        "Pricing live with entitlements enforced for every tier",
        "Support rotation staffed with a completed game-day exercise",
        "Legacy sunset announced with dates and deprecation headers live",
        "Staged rollout completed to 100% with no open SEV1/2",
        "M4 GA milestone declared",
    ],
    defer=[
        "1.1 feature work",
        "Additional locales beyond English and Bangla",
    ],
))


def normalized_generated_document(contents):
    """Ignore delivery-state metadata while checking generated roadmap content."""
    contents = re.sub(r"^Spec-Version: .+$", "Spec-Version: GENERATED", contents, flags=re.MULTILINE)
    contents = re.sub(r"^- \[[xX]\] ", "- [ ] ", contents, flags=re.MULTILINE)
    return "\n".join(line.rstrip() for line in contents.strip().splitlines())


def main(check=False):
    assert len(PHASES) == 30, "expected 30 phases, got {}".format(len(PHASES))
    seen = set()
    drifted = []
    for p in PHASES:
        assert p["n"] not in seen, "duplicate phase number {}".format(p["n"])
        seen.add(p["n"])
        path = OUT_DIR / "phase-{:02d}.md".format(p["n"])
        generated = render(p)
        if check:
            if not path.exists() or normalized_generated_document(path.read_text(encoding="utf-8")) != normalized_generated_document(generated):
                drifted.append(path.name)
        else:
            path.write_text(generated, encoding="utf-8")
            print("wrote", path.name)
    if drifted:
        raise SystemExit("generated roadmap drift: " + ", ".join(drifted))
    if check:
        print("roadmap generation check passed for 30 phases")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail without writing when generated phases drift")
    main(check=parser.parse_args().check)
