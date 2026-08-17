# Mockups

Spec-Version: 2.1.0

The clickable design mockup for the Eury Agent desktop application. This is a **Phase 0, day-one deliverable**: it exists before any UI code, and it is the visual source of truth for [`docs/05-ui/`](../docs/05-ui/00-visual-language.md).

## Open it

```bash
open agent/mockups/index.html        # macOS
xdg-open agent/mockups/index.html    # Linux
start agent\mockups\index.html       # Windows
```

One self-contained HTML file. No build step, no server, no network access, no dependencies — it works offline and on an air-gapped machine. A design artifact that needs a toolchain to view stops getting looked at.

The mockup renders the app at its true default window size (1360 × 820) and scales it to fit your browser, so you always see the intended layout instead of the responsive breakpoints.

## What is in it

Ten screens, matching the wireframes in [visual language](../docs/05-ui/00-visual-language.md) one to one:

| # | Screen | Shows |
|---|---|---|
| 1 | Home | General chat, Projects, Artifacts, Customize, account history, and entry to Code |
| 2 | Code — agent run | Compact project sidebar, centered conversation, tool cards, and composer |
| 3 | Approve command | Approval card for a destructive shell command |
| 4 | Approve diff | Approval card with a reviewable diff and word-level marks |
| 5 | Changes | Diff pane: file list, unified diff, per-hunk and per-file revert |
| 6 | Plan | Plan card with steps, risk, estimates, and build actions |
| 7 | Palette | Command palette |
| 8 | Trust | Untrusted project modal |
| 9 | Runs | Run history with a timeline, decisions, and restore points |
| 10 | Settings | Permissions: defaults, standing grants, locked organization policy |

## Controls

| Input | Effect |
|---|---|
| `←` `→` | Previous and next screen |
| `1`–`0` | Jump to a screen |
| `Esc` | Back to the agent run screen |
| Click | Home/Code switch, navigator buttons, and sidebar rows that lead to Runs, Plans, Approvals, and Settings |
| `theme` button | Toggle dark and light |
| `accent` button | Cycle the five accent tokens |

The theme and accent toggles exist to prove the token layer: both switch by changing one attribute on `<html>`, with no component re-render, which is the contract in the [design system](../docs/05-ui/01-design-system.md).

## How it relates to the specs

| Artifact | Role |
|---|---|
| [visual language](../docs/05-ui/00-visual-language.md) | What we clone from Claude Code, what a desktop GUI does better, plus the normative wireframes |
| [design system](../docs/05-ui/01-design-system.md) | Token names and values, mirrored literally in this file's `:root` block |
| [app shell](../docs/05-ui/02-app-shell-and-navigation.md) | Region sizes, pane model, overlay and modal rules |
| [chat and streaming UX](../docs/05-ui/03-chat-and-streaming-ux.md) | Turn and tool-card anatomy |
| [approval and trust UX](../docs/05-ui/05-approval-and-trust-ux.md) | Button order, styles, risk presentation, trust modal |

The CSS custom properties here are copied from the design system's token table. When a token changes, it changes in both places in the same commit, and the `agent-ci` workflow fails if the two drift ([CI](../docs/07-ops/02-ci-cd-pipelines.md)).

## What it is not

- Not React, and not the beginning of the implementation. It is a reference that stays useful as a review target.
- Not a component library. Real components live in `apps/desktop/src/components/` from Phase 2 onward.
- Not interactive beyond navigation. There is no state, no streaming simulation, and no data.
- Not a substitute for visual regression tests, which run against the real components.

## When to update it

Update the mockup **before** changing UI code whenever a layout, token, or interaction pattern changes. Reviewing a picture costs a minute; reading a React diff to imagine a layout costs an hour and usually fails.

Once the real UI exists, the mockup is retired screen by screen: each screen is deleted when its live counterpart is covered by a visual regression fixture. Tracking that retirement is a Phase 5 exit criterion ([roadmap](../docs/09-roadmap/phase-05.md)).
