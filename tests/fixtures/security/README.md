# Phase 2 security contract fixtures

These files are machine-readable contracts for future security implementations.
They define expected decisions and errors; they do **not** claim that a runtime
test or implementation currently passes.

Consumers should load `manifest.json`, validate it and each listed corpus
against `security-fixture.schema.json`, and reject duplicate case IDs.

## Files

- `security-fixture.schema.json` — JSON Schema draft 2020-12.
- `manifest.json` — corpus index and expected case counts.
- `path-traversal-symlink.json`
- `command-metacharacter-egress.json`
- `prompt-injection-provenance-delimiters.json`
- `secrets-redaction.json`
- `ssrf-residency.json`
- `hostile-mcp.json`
- `malformed-policy.json`

## Identifier contract

`TEST-xxx` is the stable fixture-case namespace. Linked IDs use the planned
`A-xxx`, `T-xxx`, and `C-xxx` shapes. This local registry gives those links
stable meanings until a canonical project-wide registry supersedes it; IDs
must be mapped, not silently renumbered.

### Assets

- `A-001` — workspace source and filesystem data
- `A-002` — credentials, tokens, and private keys
- `A-003` — conversations, prompts, and model/tool context
- `A-004` — logs, telemetry, and audit records
- `A-005` — effective organization/workspace policy

### Threats

- `T-001` — lexical or canonical path traversal
- `T-002` — symlink, junction, or reparse-point escape
- `T-003` — command injection or destructive execution
- `T-004` — credential or source-data exfiltration
- `T-005` — prompt injection and privilege escalation
- `T-006` — provenance or delimiter confusion
- `T-007` — secret disclosure through context, UI, or logs
- `T-008` — SSRF into loopback, private, link-local, or metadata services
- `T-009` — malicious or malformed MCP server behavior
- `T-010` — data residency policy violation
- `T-011` — malformed, stale, forged, or widening policy

### Controls

- `C-001` — decode, normalize, canonicalize, and enforce workspace boundary
- `C-002` — resolve and reject escaping links/reparse points
- `C-003` — deny by default and fail closed
- `C-004` — structured argv and explicit shell/metacharacter handling
- `C-005` — execute-time network policy and egress isolation
- `C-006` — host-owned provenance and collision-safe content framing
- `C-007` — static policy/instruction separation
- `C-008` — mandatory secret redaction before model, UI, and logs
- `C-009` — scheme, address, and network-range SSRF filtering
- `C-010` — DNS and redirect target revalidation
- `C-011` — authoritative residency routing
- `C-012` — MCP approval, registry, and fingerprint enforcement
- `C-013` — untrusted marking for dynamic and third-party content
- `C-014` — MCP protocol, size, capability, and resource limits
- `C-015` — strict policy validation, monotonicity, and signature checks

Payload values marked `synthetic` are intentionally fake and exist only to
exercise redaction-shape contracts.
