# Approval and Trust UX

Spec-Version: 3.0.0

Approval is the product's most security-critical surface. It must be fast enough to use dozens of times a day and honest enough that a rushed click is still a safe click.

## Principles

1. **Deny by default.** No write, execute, or network tool runs without a grant ([ADR-0006](../02-architecture/adr/0006-deny-by-default-permissions.md)).
2. **Show the exact operation.** Full path, full command, full host — never a paraphrase.
3. **A card in the conversation, not a modal.** The request appears where the work is, so the surrounding context stays visible and the rest of the app stays usable. Only `critical` risk escalates to a modal.
4. **The safe choice is the obvious choice.** The narrowest allow is the primary button; deny is always present and always leftmost.
5. **Escalation is explicit.** Broadening scope is a separate split button, never a side effect of approving.
6. **Every decision is auditable.** Grants, denials, revocations, and expirations are recorded.
7. **No dark patterns.** No "always allow" as the primary button, no countdown auto-approve, no card that moves under the cursor.

## Approval card

Matches [visual language](00-visual-language.md) wireframes 3 and 4, implemented in [`agent/mockups/`](../../mockups/README.md).

```
┌ ⚠ Approval required ─────────────────────────── Elevated risk ──┐
│ ⌘ Run command                                                   │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ pnpm migrate:reset                                          │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ Directory   ~/dev/acme-api                                      │
│ Reason      Step 3 of plan: reset local DB before re-seeding     │
│ Risk        Matches destructive pattern: migrate:reset           │
│ Policy      Allowed up to session scope by acme-corp (v14)       │
│                                                                 │
│ [ Deny ]  [ Allow once ]  [ Allow for this session ▾ ]           │
│                                          Esc to deny            │
└─────────────────────────────────────────────────────────────────┘
```

| Field | Source |
|---|---|
| Header | Risk level, stated in words as well as color |
| Tool | Human-readable tool name with its class icon |
| Payload | Exact command, path, or URL. Monospace, never truncated; it wraps |
| Directory | Resolved absolute working directory |
| Reason | Agent-provided rationale, plus the plan step when one applies |
| Risk | Matched risk rules from the policy engine |
| Policy | Which policy is in force and what scope ceiling it sets |
| Diff | For write tools, a `DiffView` with line numbers, word-level marks, and open-in-editor replaces the payload block (wireframe 4) |

### Button construction

Buttons are generated from the grant scopes policy currently permits, in fixed order:

| Position | Button | Style | Present when |
|---|---|---|---|
| 1 | Deny | `secondary` | Always |
| 2 | Allow once, or Apply once for writes | `primary` | Always |
| 3 | Allow for this run / session / always, as a split button | `secondary` | Policy allows a scope wider than `once`, and risk is not `critical` |

The split button's default action is the **narrowest** scope the policy allows; its menu offers the wider ones. Scopes the policy caps are absent rather than disabled, and the Policy row states which policy narrowed them, so the absence is explained rather than mysterious ([policy engine](../03-security/03-permission-and-policy-engine.md)).

Replying in the composer instead of deciding is always available: it denies the request and passes the reply to the agent as redirection.

## Risk levels

| Level | Badge | Examples | Presentation and default focus |
|---|---|---|---|
| `low` | none | read outside the index, small edit inside the workspace | Card; focus on Allow once |
| `medium` | info | new file, dependency install, network fetch | Card; focus on Allow once |
| `elevated` | warning | multi-file rewrite, dependency install, shell metacharacters | Card with warning header; focus on Deny |
| `critical` | danger | write to an approved outside-workspace capability root, recursive delete inside workspace, approved production-shaped command | **Modal** dialog; focus on Deny, plus typed confirmation |
| `forbidden` | blocked | `sudo`, root/device destruction, credential-store access, pipe-to-shell, Git push/force/reset-hard/clean | No approval UI; render a policy-denied explanation |

`critical` is the only approvable risk level that becomes a modal. It requires
typing the last path segment or command name before Allow enables, offers no
scope wider than `once`, and uses the `danger` style. `forbidden` is a
non-overridable decision category, not an approvable risk level.

## Scope options

| Scope | Lifetime | Persisted |
|---|---|---|
| `once` | This single call | No |
| `run` | Until the run ends | No |
| `session` | Until the app quits | No |
| `always` | Until revoked, scoped to the workspace | Yes, `grants` table ([local data model](../04-specs/05-local-data-model.md)) |

Scopes are filtered by organization policy, which may cap the maximum grantable scope per tool class ([policy engine](../03-security/03-permission-and-policy-engine.md)).

Grant matching for `always` is by **normalized shape**, not raw string: tool id plus argument pattern (path prefix, or `argv[0]` plus flags), so `pnpm test -- foo` does not silently grant `pnpm test -- $(curl …)`.

## Queueing and batching

Approvals arrive in a FIFO queue and are answered one at a time. Only the head of the queue renders a card; the status bar shows `N pending`, the sidebar shows a badge, and the full queue is inspectable in the Approvals pane. Identical-shape requests within one run collapse into one card that states the count and lists each instance on expand. Approving a batch approves exactly the listed instances — never a pattern beyond them.

The run pauses while an approval is pending. The waiting time is excluded from run latency metrics but recorded as `approval_wait_ms`.

## Interaction rules

| Input | Result |
|---|---|
| Click | Activates that button |
| `Tab` | Moves between buttons in visual order |
| `Enter` | Activates the focused button, which starts on the safest one |
| `Esc` | Deny, always, from anywhere in the app |
| `⌘⏎` | Allow with the primary button's scope |
| Typing in the composer | Denies and passes the reply to the agent as redirection |
| Scrolling away or opening another pane | Allowed; the card stays where it is and the status bar keeps showing `N pending` |
| Window unfocused | Native notification after 2 s; the card waits |
| Timeout (default 10 min) | Auto-deny with `EURY_APPROVAL_TIMEOUT`; the run pauses recoverably rather than failing |

Buttons are inert for 400 ms after the card renders, which defeats a stray `Enter` already in flight from the composer without feeling laggy. Because approval is a card rather than a modal, a pending request never blocks reading the conversation, opening the Changes pane, or inspecting the diff it refers to.

## Workspace trust

On first open of a folder, a modal — this is the one case where nothing else should proceed until the question is answered (wireframe 8):

```
┌ Trust this project? ─────────────────────────────────────────┐
│ ~/dev/unknown-repo                                           │
│ Files       1,284                                            │
│ Git remote  github.com/acme/unknown-repo                     │
│ Found       EURY.md · .eury/policy.json · 3 MCP configs      │
│                                                              │
│ Until you trust it, Eury runs read-only in this project: no  │
│ shell, no network, and no MCP servers. The project's own      │
│ instructions and MCP configuration are shown for your review  │
│ rather than applied.                                          │
│                                                              │
│        [ Cancel ]  [ Open read-only ]  [ Trust project ]      │
└──────────────────────────────────────────────────────────────┘
```

`Open read-only` is the primary button, because it is the safe option that still lets the user get work done.

| Trust state | Capabilities |
|---|---|
| `untrusted` | Read-only tools inside the root; no execute, no network, no MCP, and the project's own config is displayed for review rather than honored |
| `trusted` | Full tool catalog subject to policy; the project's `.eury/policy.json` may only **narrow** permissions |
| `revoked` | Returns to untrusted; grants cleared |

Workspace-level config can never widen permissions beyond org policy or user
settings. Trust is per canonical absolute path, stored in
`workspaces.trust_state`, and re-prompted if the path's git remote changes.

## Approvals surface

The Approvals pane shows pending requests plus a searchable decision history: timestamp, tool, payload, scope, decision, run link, policy version. Standing grants are listed with revoke buttons, use counts, and last-used timestamps (wireframe 10). Revoking takes effect immediately, including mid-run.

## Notifications

When the window is unfocused and an approval is pending, send an OS notification after 2 s ("Eury Agent needs approval: run_command"). Clicking focuses the window and the card. Notification bodies never contain file contents or command arguments beyond the tool name.

## Policy-denied UX

A policy denial is not an approval request, so it never renders buttons that imply a choice. The tool card shows a `Denied` status, names the rule and policy version, and offers "Request exception", which copies an admin-ready summary (organization, rule, tool, payload, justification field). The agent receives a structured denial and is expected to propose an alternative.

## Audit

Every decision emits `permission_granted` / `permission_denied` / `permission_revoked` / `approval_timeout` with tool id, argument hash, scope, risk level, policy version, and decision latency ([audit and retention](../06-enterprise/04-audit-and-retention.md)). Payloads are hashed, not stored raw, unless the org opts into full-payload audit.

## Accessibility

The card is a `role="group"` with an accessible name that includes the risk level in words, announced through the conversation's live region when it appears. Focus moves to the safest button once, without stealing focus from an in-progress composer edit. `critical` uses `role="alertdialog"` with a focus trap that returns focus to the originating tool card on close, and its typed confirmation has a visible label and an accessible error message. Risk is never conveyed by color alone ([accessibility](07-accessibility-and-i18n.md)).

## Anti-goals

- No "trust all workspaces" global setting.
- No auto-approve on a timer.
- No YOLO mode without an explicit, session-scoped, audited, org-policy-gated toggle that is off by default and visibly banner-marked while active.

## Related documents

- [Visual language](00-visual-language.md)
- [Permission and policy engine](../03-security/03-permission-and-policy-engine.md)
- [Sandbox model](../03-security/02-sandbox-model.md)
- [Tool activity and diff UX](04-tool-activity-and-diff-ux.md)
- [Workspace policies](../06-enterprise/03-workspace-policies.md)
- [Mockups](../../mockups/README.md)
