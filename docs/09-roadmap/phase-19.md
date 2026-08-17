# Phase 19 — MCP

Spec-Version: 1.1.0

**Track:** D — Intelligence · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Support Model Context Protocol servers as first-class, policy-governed tool sources with a trust model that matches their risk.

## Why this phase exists here

MCP is how the product extends without us writing every integration. It is also the largest new attack surface in the product, so it lands after policy, approvals, and sandboxing are mature.

## In scope

- MCP client supporting stdio and HTTP/SSE transports
- Server lifecycle: spawn, health, restart with backoff, shutdown
- Tool discovery mapped into the tool registry with namespaced ids
- Manifest hashing, optional signature verification, and an approval registry
- Per-server permission scoping and org allowlists
- Resource and prompt support where the server provides it
- Sandboxing of local server processes
- Server management UI with logs, tool listing, and enable/disable

## Feature IDs

`F-046`, `F-066`

## Out of scope

- A public MCP marketplace (post-GA)
- Authoring MCP servers ourselves beyond examples

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D19.1 | MCP client with both transports and robust framing | [MCP integration](../04-specs/10-mcp-integration-spec.md) |
| D19.2 | Server supervisor with restart backoff and health reporting | [MCP integration](../04-specs/10-mcp-integration-spec.md) |
| D19.3 | Namespaced tool registration (`mcp__<server>__<tool>`) | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D19.4 | Manifest hash pinning and re-approval on change | [admin console](../06-enterprise/06-admin-console-spec.md) |
| D19.5 | Per-server policy scoping and org allowlist enforcement | [workspace policies](../06-enterprise/03-workspace-policies.md) |
| D19.6 | Local server process sandboxing | [sandbox model](../03-security/02-sandbox-model.md) |
| D19.7 | Server management UI with per-server logs | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D19.8 | MCP message parser fuzz target | [security testing](../08-quality/04-security-testing.md) |

## Key decisions and design notes

- MCP tools are untrusted by default: their results are untrusted content, and their calls require the same approval discipline as any other tool class.
- Manifests are pinned by hash. A server that changes its tool surface requires re-approval rather than silently gaining capability.
- Local MCP servers are disabled by default for Enterprise policy presets.
- A misbehaving server is isolated and disabled, never allowed to hang a run.

## Contracts touched

- MCP server configuration schema
- Namespaced tool id format
- MCP audit events

## Dependencies

- Phase 6 (tool registry)
- Phase 7 (policy and approvals)
- Phase 5 (sandbox)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Malicious MCP server | Data exfiltration or code execution | Approval registry, hash pinning, sandboxing, network policy, untrusted results, and audit |
| Prompt injection via MCP results | Agent misdirection | Results marked untrusted; privileged actions still require approval |
| Server instability | Hung or flaky runs | Timeouts, health checks, backoff, and automatic disable after repeated failure |
| Tool name collisions | Wrong tool invoked | Mandatory namespacing; collision detection at registration |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Protocol framing, capability negotiation, error mapping |
| Integration | Reference servers over both transports; restart and failure paths |
| Security | Hostile server suite: oversized responses, injection payloads, unauthorized tool claims |
| Fuzz | MCP message parser |
| Policy | Org allowlist and local-server denial enforced |

## Metrics and targets

| Metric | Target |
|---|---|
| Server startup | < 2 s p95 |
| Tool call overhead vs. native | < 20 ms added p95 |
| Hostile server suite | 100% contained |
| Unapproved server invocations | 0 |

## Exit criteria

- [ ] MCP servers connect over stdio and HTTP/SSE and register namespaced tools
- [ ] Manifest hash pinning forces re-approval on change
- [ ] Local servers are sandboxed and policy-gated
- [ ] Hostile server suite fully contained
- [ ] Server management UI shows health, tools, and logs

## Deferred from this phase

- MCP marketplace and discovery (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
