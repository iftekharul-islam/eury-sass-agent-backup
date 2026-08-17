# Security Testing

Spec-Version: 1.2.0

Every control in the [threat model](../03-security/01-threat-model.md) maps to a
stable `TEST-*` contract. Phase 2 proves schemas, corpora, static rules, and CI
wiring; runtime assertions become executable only in their owning phases.

| Evidence maturity | Meaning |
|---|---|
| contract | Machine-readable case and expected result exist |
| static | Repository/rule/config can be checked without runtime feature |
| executable | Implemented control is exercised on supported platforms |
| release | Signed artifact/system evidence is captured |

## Verification contracts (TEST-001 to TEST-016)

| ID | Control verified | Verification contract | Maturity |
|---|---|---|---|
| TEST-001 | C-001 Path guard | Canonical path guard, scoped roots, symlink/TOCTOU safe | contract / static |
| TEST-002 | C-002 Command & OS sandbox | Parsed command policy and OS sandbox containment | contract / static |
| TEST-003 | C-003 Policy engine | Canonical decision engine; no implicit privileged allow | contract / static |
| TEST-004 | C-004 Approval UX | Anti-spoofing; forbidden actions never prompt | contract / static |
| TEST-005 | C-005 Injection defense | Structured provenance, monotonic trust, grant invalidation | contract / static |
| TEST-006 | C-006 Secret management | Keychain storage, least lifetime, redaction, no-plaintext-fallback | contract / static |
| TEST-007 | C-007 Policy versioning | Signed/versioned policy, no-widen merge, downgrade/staleness | contract / static |
| TEST-008 | C-008 MCP security | MCP manifest fingerprinting, approval, transport restrictions | contract / static |
| TEST-009 | C-009 Supply chain | Locked dependencies, advisory/license gates, SBOM/provenance | executable / static |
| TEST-010 | C-010 Release signing | Signed artifacts, protected root keyset, rollback protection | contract / release |
| TEST-011 | C-011 Identity & Auth | PKCE, token rotation, tenant isolation, IDOR prevention | contract / static |
| TEST-012 | C-012 Audit integrity | Device-signed hash chain, idempotent append, gap detection | contract / static |
| TEST-013 | C-013 Quotas & Budgets | Atomic quotas, reservations, integer money, rate limits | contract / static |
| TEST-014 | C-015 Desktop UI / IPC | Strict CSP, sanitized rendering, constrained IPC | contract / static |
| TEST-015 | C-015 Egress control | Execute egress defaults off; network approval is separate | contract / static |
| TEST-016 | C-016 Privacy & Residency | Purpose/residency/retention checks fail closed | contract / static |

## Static analysis

| Tool | Scope | Gate |
|---|---|---|
| `cargo clippy -D warnings` | Rust lints incl. correctness | PR blocking |
| `cargo audit` | RustSec advisories | PR blocking on high/critical |
| `cargo deny` | Licenses, duplicates, yanked crates | PR blocking |
| `pnpm audit` | npm advisories | PR blocking on high/critical |
| Semgrep | Custom Rust/TS rules (below) | PR blocking |
| gitleaks | Secret scanning, full history on schedule | PR blocking |
| `tsc --noEmit` + strict ESLint | Type and unsafe-pattern checks | PR blocking |
| Prisma migration diff | Proves Agent migrations are additive | PR blocking |

### Custom Semgrep rules

| Rule | Rationale |
|---|---|
| No `std::fs` outside `agent-sandbox` | All filesystem access must pass the path guard |
| No `Command::new` outside `agent-sandbox` | All execution must pass the command guard |
| No `unwrap`/`expect`/`panic!` in tool or IPC paths | A panic in the core is a denial of service |
| No `dangerouslySetInnerHTML` outside the sanitized markdown component | XSS in the webview |
| No `invoke` with unvalidated user input | IPC boundary discipline |
| No secret-shaped string literals | Accidental key commits |
| No `console.log` of message or tool payloads | Content leakage |
| Agent module must not import other feature modules | Isolation rule R3 ([module structure](../04-specs/16-backend-module-structure.md)) |
| No `process.env` reads outside `AgentConfigService` | Config discipline |
| No raw SQL string interpolation | Injection |

## Fuzzing

| Target | Corpus seed | Frequency |
|---|---|---|
| Path normalization / guard | Traversal corpus, unicode, long paths, UNC, symlinks | Continuous nightly, 1 h per target |
| Command parser / allowlist matcher | Shell metacharacter corpus | Nightly |
| Event protocol deserializer | Golden events, mutated | Nightly |
| IPC command deserializer | Command fixtures, mutated | Nightly |
| Patch/diff applier | Malformed and adversarial patches | Nightly |
| Plan file parser | Malformed markdown/frontmatter | Weekly |
| MCP message parser | Malformed JSON-RPC | Weekly |
| Manifest/signature verifier | Truncated and tampered manifests | Weekly |

`cargo-fuzz` with persisted corpora in the repo; a new crash becomes a unit test before it is fixed.

## Dynamic and integration security tests

| Test | Assertion |
|---|---|
| Sandbox escape suite | No tool can read or write outside the workspace root without an explicit grant, on all three platforms |
| Symlink escape | A symlink pointing outside the root is refused, both for read and write |
| TOCTOU | Path swapped between check and use is detected (open-then-verify, not verify-then-open) |
| OS sandbox active | Seatbelt/Landlock/Job object is verifiably applied; a test tool attempting a blocked syscall fails |
| Egress control | With `networkDuringExecute = false`, a spawned process cannot reach the network |
| No implicit privileged allow | Fresh profile: write/execute/network/MCP returns `needsApproval` or non-overridable `deny`, never `allow` |
| Grant scope | A `once` grant cannot be reused; a `run` grant expires at run end; `always` matches only the normalized shape |
| Policy fail-closed | Corrupt/unsigned/rollback policy disables privileged tools; stale policy follows the canonical hard-limit conditions |
| Policy no-widen | Property test over random policy pairs |
| Keychain | Secrets never appear in SQLite, logs, telemetry, crash dumps, or exports |
| Redaction | Fuzzed secret corpus never survives into any outbound payload |
| IPC surface | Every command validates input and rejects oversized and malformed payloads |
| CSP | Webview cannot load remote scripts, fonts, or frames |
| Prompt injection corpus | Instructions in files, tool output, web results, MCP results, and image alt text never cause an unapproved privileged action |
| MCP trust | Unapproved server is refused; signature is required when policy says so; any executable/manifest fingerprint change requires re-approval |

## Cloud security tests

| Test | Assertion |
|---|---|
| Unauthenticated access | Every non-public `/agent/v1/*` route returns 401 without a token |
| No-config-open-endpoint | Missing `AGENT_JWT_SECRET` fails boot; no route becomes public |
| Token validation | Expired, wrong audience, wrong signature, and `alg:none` are rejected |
| Refresh rotation | Reuse revokes the chain and emits an audit event |
| PKCE | `plain` method rejected; mismatched verifier rejected; device code single-use |
| Authorization | Member cannot reach admin routes; cross-org access denied on every route |
| IDOR sweep | Every id-bearing route tested with another org's id |
| Rate limits | Device-code polling, auth, and gateway limits enforced |
| Audit signature | Wrong key, tampered body, and replayed batch all rejected |
| SCIM | Absent token → 404; wrong token → 401; deprovision revokes tokens and devices |
| SSO | Unsigned assertion rejected; replayed assertion rejected; `state` bound to the device code |
| SSRF | Web-search and fetch proxies refuse internal addresses and link-local metadata endpoints |
| Injection | SQL and NoSQL injection attempts on every filter parameter |
| Headers | Security headers present; no stack traces in production responses |
| DAST | OWASP ZAP baseline scan against staging `/agent/v1` |

## Penetration testing

| Scope | Frequency |
|---|---|
| Desktop app (sandbox escape, local privilege, keychain, update channel) | Annual + before enterprise GA |
| Cloud Agent API (auth, authorization, tenancy isolation) | Annual + before enterprise GA |
| Supply chain / build pipeline review | Annual |

Findings are tracked with severity and a fix SLA: critical 7 days, high 30 days, medium 90 days. Enterprise GA requires all critical and high findings closed and a retest report.

## Phase 2 executable evidence

- `security:check` validates A-/ACT-/B-/T-/C-/TEST- coverage, canonical enums
  and policy fields, numeric parity, sandbox platform/degradation text, corpus
  schemas, and required CI commands.
- Every custom Semgrep rule has at least one positive and one negative fixture.
- Traversal, command/egress, injection/provenance, secret, SSRF/residency,
  hostile-MCP, and malformed-policy corpora parse against their manifest schema.
- `agent-security.yml` contains PR/main/weekly gates. A green local check proves
  configuration; GitHub run, required-check, reviewer, and negative-control
  evidence remain external until recorded.
- Sandbox escape, policy property, keychain, fuzz, cloud, signing/SBOM, DAST,
  and pentest rows are future executable evidence, not Phase 2 completion claims.

## Per-release security checklist

- [ ] No critical/high CVEs in Rust or npm dependencies
- [ ] Semgrep, gitleaks, `cargo deny` clean
- [ ] Fuzz targets ran for the release-defined duration with corpus version, sanitizer, crash deduplication, and completion artifact recorded
- [ ] Sandbox escape suite green on all three platforms
- [ ] Deny-by-default and grant-scope suites green
- [ ] Policy fail-closed and no-widen suites green
- [ ] Prompt injection corpus 100% pass
- [ ] Cloud negative-auth and IDOR sweeps green
- [ ] No unauthenticated bypass path exists (legacy `CODE_API_TOKEN`-style fallback confirmed absent)
- [ ] Signing pipeline verified; manifest signature validated by a client build
- [ ] SBOM generated and reviewed for unexpected additions
- [ ] Secrets redaction test green
- [ ] Threat model reviewed for new surfaces introduced this release
- [ ] Open pentest findings within SLA

## Vulnerability handling

Internal findings become private issues with a severity and owner. External reports go to `security@` with a 72-hour acknowledgement, coordinated disclosure, and a CVE where warranted. A bug bounty opens post-GA. Process detail: [incident response](../03-security/09-incident-response.md).

## Related documents

- [Threat model](../03-security/01-threat-model.md)
- [Sandbox model](../03-security/02-sandbox-model.md)
- [Prompt injection defense](../03-security/05-prompt-injection-defense.md)
- [Supply chain and signing](../03-security/06-supply-chain-and-signing.md)
