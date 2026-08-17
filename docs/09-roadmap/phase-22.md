# Phase 22 — Preview Runtime

Spec-Version: 1.1.0

**Track:** E — Depth · **Estimated size:** 1–2 weeks · **Milestone:** —

## Goal

An isolated preview panel for local dev servers and static output, with a safe path for feeding what it shows back to the agent.

## Why this phase exists here

Front-end work needs a visible result. Preview is straightforward except for isolation, which is why it comes after the trust and untrusted-content model is settled.

## In scope

- Preview webview restricted to localhost and workspace `file://` origins
- Full isolation: separate session, no app IPC access, no shared storage
- Port detection from tool output with a suggestion to open preview
- Controls: reload, hard reload, validated address entry, device-width presets, open in browser
- Console error and warning capture, surfaced in the Activity panel
- Screenshot capture available to the agent only after approval
- Explicit sharing of console output or DOM extracts as untrusted content

## Feature IDs

`F-054`

## Out of scope

- Remote URL previews
- Full browser automation (post-GA)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D22.1 | Isolated preview webview with an origin allowlist | [editor/terminal/preview](../05-ui/06-editor-terminal-preview.md) |
| D22.2 | Port detection and preview suggestion | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D22.3 | Preview controls and device-width presets | [editor/terminal/preview](../05-ui/06-editor-terminal-preview.md) |
| D22.4 | Console capture with untrusted-content marking | [injection defense](../03-security/05-prompt-injection-defense.md) |
| D22.5 | Approval-gated screenshot capture | [approval UX](../05-ui/05-approval-and-trust-ux.md) |
| D22.6 | `allowScreenshots` policy flag support | [workspace policies](../06-enterprise/03-workspace-policies.md) |

## Key decisions and design notes

- The preview is untrusted content by definition. Nothing it produces can enter a prompt without being marked and, for screenshots, approved.
- Only localhost and workspace files are allowed; anything else opens in the real browser.
- The preview webview has no bridge to app IPC, so a malicious page cannot reach our commands.
- DevTools for the preview exist in dev builds only.

## Contracts touched

- Preview IPC commands
- Console capture event shape

## Dependencies

- Phase 3 (pane shell)
- Phase 12 (detecting server output)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Malicious page escaping isolation | Compromise | Separate session, no IPC bridge, strict origin allowlist, CSP, and a hostile-page test suite |
| Injection via console output | Agent misdirection | Marked untrusted; sharing is explicit |
| Screenshot leaking sensitive content | Privacy | Approval required, policy flag, and no automatic capture |
| Memory from an extra webview | Bloat | Preview webview created lazily and destroyed on tab close; measured in the memory benchmark |

## Test plan

| Layer | Coverage |
|---|---|
| Integration | Dev servers for React, Vite, and a static site |
| Security | Hostile page cannot reach IPC, storage, or non-allowlisted origins |
| Policy | `allowScreenshots = false` blocks capture |
| Performance | Preview open/close memory delta |

## Metrics and targets

| Metric | Target |
|---|---|
| Preview overhead on first paint | < 100 ms |
| Memory per open preview | < 120 MB |
| Isolation suite | 100% contained |

## Exit criteria

- [ ] Preview works for local dev servers and static files
- [ ] Isolation suite proves no IPC or storage access from the previewed page
- [ ] Console capture and screenshots are explicit and policy-gated
- [ ] Memory returns to baseline after closing a preview

## Deferred from this phase

- Browser automation for the agent (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
