# Sandbox Model

Spec-Version: 2.0.0

**Owner:** Security + Desktop · **Lifecycle:** approved design contract

## Layers and invariant

Defense in depth for all tool execution:

```
Layer 4: OS sandbox (macOS Seatbelt / Linux Landlock+seccomp / Windows restricted-token+Job-object)
Layer 3: Parsed command policy + process/egress broker
Layer 2: Path guard + capability roots
Layer 1: Mode registry + policy decision + exact-shape approval
```

Each layer independently denies. An approval cannot override a forbidden
operation or compensate for an unavailable required containment capability.
All sandbox layers operate fail-closed.

## Path guard (`agent-sandbox`)

All filesystem tools MUST use an open-then-verify API owned by
`agent-sandbox`:

1. Reject empty/NUL/control input and platform-reserved names.
2. Resolve lexical relative paths against a named capability root, never the
   process current directory.
3. Normalize `.`/`..`; reject an escape before touching disk.
4. Open the existing target or nearest existing parent without following the
   final symlink.
5. Resolve the opened handle and every existing parent; verify they remain
   beneath the root immediately before the syscall.
6. For create/write, create a same-directory temporary through the verified
   parent handle, then `fsync` and atomically rename.
7. Refuse devices, FIFOs, sockets, junction/reparse escapes, hard-link
   surprises, and deny-listed paths.
8. Re-check the target identity/hash at mutation time to prevent stale writes.

Operation sizes, result caps, timeouts, and deny globs are defined only in the
[tool catalog](../04-specs/02-tool-catalog-spec.md).

### Capability roots

The workspace root is the only implicit root. Outside-workspace access requires
a user-selected directory capability `{ capabilityId, canonicalRoot, access,
workspaceId, expiresAt }`. The model receives only `capabilityId` and a display
label. `write_outside_workspace` policy and an exact-shape approval are both
required; no grant can target `/`, a home directory, a secret directory, or an
arbitrary absolute path.

## Command allowlist (`agent-sandbox`)

Every command is `execute`, including tests and builds: repository scripts are
arbitrary code. The parser produces argv, cwd, executable identity, shell mode,
declared mutations, requested egress, and environment names; this normalized
shape is what policy and approval bind.

| Category | Examples | Decision |
|---|---|---|
| Inspect-only native Git | `git status`, `git diff`, `git log` through typed Git tools | `read` tool contract, not shell |
| Execute, ordinary | tests, linters, compilers, local dev servers | `needsApproval`; egress off by default |
| Execute, elevated | dependency install, Docker build, shell metacharacters | elevated/critical approval; egress is a separate shape field |
| Forbidden | `sudo`, privilege/ACL weakening, destructive root/device commands, fork bombs, pipe-to-shell, force/reset/clean/push operations | non-overridable `deny`; never show approval |

Cersei classification is advisory metadata only; local policy and OS
containment remain authoritative.

### Execute-time egress

Child-process egress defaults off. `networkDuringExecute` cannot silently
default true. When an operation needs network, approval and audit include
`egress: true`, protocol, host rules, and duration; the OS sandbox enforces the
same shape. A standing execute grant without egress never implies Network.

## OS sandbox

| OS | Required baseline | Capability probe | Degradation |
|---|---|---|---|
| macOS | Seatbelt profile scoped to capability roots/temp plus process-group controls | Spawn a probe denied outside root and denied egress | Probe/profile failure disables write, execute, MCP-local, and outside-root tools |
| Linux | Landlock ABI with filesystem rules plus seccomp/no-new-privileges and process group | Report kernel ABI and run denied filesystem/syscall/egress probes | Missing required ABI disables privileged tools; no “guard-only” execute mode |
| Windows | Restricted token, Job object kill/process limits, brokered handles and network rule | Probe token, job membership, outside-root handle, process tree, and egress | Missing restricted token/broker enforcement disables privileged tools |

`SandboxCapabilities` records OS, mechanism versions, verified probes, egress
enforcement, outside-root support, and reason codes. Local Standard and
Enterprise/Regulated profiles both fail closed for privileged tools; Regulated
may additionally refuse to open the workspace when its minimum capability set
is not met. History/export and non-workspace Chat remain available.

## Network tools

`web_fetch`, `web_search`, image generation, and network-capable MCP use
separate Network/MCP policy decisions. Read-only MCP is `read` only after
approved executable/manifest fingerprinting.

**SSRF protection:** block RFC1918, metadata IPs (169.254.169.254), localhost unless user enables "allow localhost" for preview.

## PTY terminal (user-initiated)

User terminal is user-operated and clearly separated from agent execution.
Agent `run_command` always uses the sandboxed process broker, including when its
output is displayed in a PTY.

## Testing

- Contract now: Phase 2 traversal/command corpora load and map to C-001/C-002/C-015.
- Implementation: Phase 5 path/OS containment and Phase 12 process tests.
- Fuzzing: Phase 28 persisted path/command corpora.

## Related documents

- [03-permission-and-policy-engine.md](03-permission-and-policy-engine.md)
- [ADR-0006](../02-architecture/adr/0006-deny-by-default-permissions.md)
