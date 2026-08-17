## Summary

- 

## Validation

- [ ] Relevant unit, integration, and contract checks pass
- [ ] Visual change includes a screenshot or short clip
- [ ] Documentation and traceability updates are included when behavior changed

## Security impact

- Assets / actors / boundaries affected (`A-*`, `ACT-*`, `B-*`): 
- Threats and controls (`T-*`, `C-*`, `TEST-*`), or `none` with rationale: 
- [ ] New/changed privileged behavior uses `allow | needsApproval | deny`; `deny` is not represented as a grant scope
- [ ] Filesystem, process, egress, MCP, trust/provenance, secret, telemetry, and audit boundaries use canonical schemas
- [ ] Untrusted/semi-trusted content cannot widen policy or reuse a standing grant after an injection finding
- [ ] Dependency/action/tool changes include transitive, lifecycle-script, license, advisory, pin, and rollback review
- [ ] Any exception has an owner, approver, compensating control, expiry, and removal target

## Feature-done controls

- [ ] New tool, IPC, event, HTTP, or database contract has fixtures and error codes
- [ ] Security/cost-relevant behavior has policy, audit, and redaction coverage
- [ ] No direct filesystem or process access bypasses `agent-sandbox`
- [ ] No legacy `code` identifiers or dependencies were added
- [ ] Required owners reviewed security-sensitive changes
- [ ] Added or changed security rule has positive and negative fixtures
- [ ] Security-sensitive evidence and residual risk are linked; no secret/content is pasted into the PR
