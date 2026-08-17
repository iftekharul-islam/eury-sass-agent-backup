# Accessibility and Internationalization

Spec-Version: 1.2.0

Target: **WCAG 2.2 Level AA** for the desktop UI. Accessibility is a release gate, not a backlog item ([definition of done](../08-quality/05-definition-of-done.md)).

## Keyboard

| Requirement | Rule |
|---|---|
| No mouse-only affordance | Every action reachable by keyboard, including hunk apply, tool card expand, and approvals |
| Focus visible | 2px accent ring, 2px offset, never `outline: none` |
| Focus order | Matches visual order; no positive `tabIndex` |
| Focus restore | Closing a dialog/popover returns focus to the trigger |
| Focus traps | Only in modal dialogs; `Esc` always exits (and denies, for approvals) |
| Skip links | "Skip to composer", "Skip to activity" as the first focusables |
| Region cycling | `F6` moves between sidebar, the active pane, composer, and status bar |

Full keymap: [keyboard and command palette](08-keyboard-and-command-palette.md).

## Screen readers

Tested with VoiceOver (macOS), NVAPI/NVDA (Windows), Orca (Linux, best effort).

| Surface | Semantics |
|---|---|
| Message list | `role="log"`, `aria-relevant="additions text"`, `aria-label="Conversation"` |
| Streaming turn | Announced via a debounced live region, ≤ 1 announcement/second, sentence-boundary chunks |
| Tool card | `role="listitem"`; name = `"<tool> <summary>, <status>, <duration>"` |
| Approval card | `role="group"`, announced through the conversation live region, risk in the accessible name |
| Critical approval dialog | `role="alertdialog"`, `aria-describedby` the payload, focus trapped until deny or typed confirmation |
| Diff | Per-line `aria-label` prefix `"added"` / `"removed"` / `"context"` |
| Status bar | `role="status"`, polite; connection changes announced once |
| Progress | `role="progressbar"` with `aria-valuetext` for index and run progress |
| Toasts | `role="status"` for info, `role="alert"` for errors |

Announcements are throttled centrally by a single live-region manager so concurrent streams cannot flood the screen reader.

## Visual

| Requirement | Rule |
|---|---|
| Contrast | ≥ 4.5:1 body text, ≥ 3:1 large text and UI boundaries, in both themes and all five accents |
| Color independence | Every status has icon + text; diffs have gutter glyphs (`+`, `−`) |
| Zoom | 200% page zoom with no loss of function; `Cmd/Ctrl +/-/0` |
| Text resize | Respects OS text scale up to 200% |
| Reflow | Usable at 900px width without horizontal scrolling of the shell |
| Reduced motion | `prefers-reduced-motion` disables all transitions and the streaming cursor blink |
| Reduced transparency | No blur/vibrancy effects when the OS requests reduced transparency |
| High contrast | Windows high-contrast and macOS increase-contrast map to a dedicated token override set |
| Target size | ≥ 24 × 24 px for all interactive targets (WCAG 2.2 AA) |

## Cognitive and error handling

- Every error states what failed, the `EURY_*` code, and one recovery action.
- Destructive confirmations name the object being destroyed.
- No time-limited interaction except the approval timeout, which is generous (10 min), configurable, and fails closed.
- Session state (drafts, scroll, open tabs) survives restart so an interrupted user loses nothing.

## Internationalization

| Aspect | Decision |
|---|---|
| Library | `i18next` + `react-i18next` |
| Bundled locales at GA | `en` (source), `bn` |
| Planned | `es`, `pt-BR`, `ja`, `de`, `fr`, `hi` |
| Catalog format | JSON per namespace: `common`, `chat`, `tools`, `approvals`, `settings`, `errors` |
| Key style | `namespace:dot.case.key`, English default value inline in code |
| Extraction | `i18next-parser` in CI; a missing or orphaned key fails the build |
| Interpolation | Named placeholders only (`{{count}}`); no string concatenation |
| Plurals | ICU plural rules via i18next; never `n === 1 ? "x" : "xs"` |
| Numbers, dates, currency | `Intl.*` with the app locale; relative times via `Intl.RelativeTimeFormat` |
| Locale source | OS locale, overridable in Settings → Appearance; persisted as `ui.locale` |
| Fallback | Missing key → English, logged in dev, never shows a raw key |

### RTL

Layout is logical-property based (`margin-inline-start`, `padding-inline-end`, `inset-inline`). `dir="rtl"` on `<html>` for RTL locales. Exceptions that stay LTR regardless of locale: code blocks, diffs, terminal, file paths, and command payloads — mirroring them would misrepresent the content. Directional icons (chevrons, back/forward) mirror; semantic icons (shield, terminal) do not.

### Content that is never translated

Tool ids, `EURY_*` error codes, file paths, commands, model ids, and log messages. User-facing error *messages* are translated; the code shown alongside them is not.

### Text expansion

Layouts must absorb +40% string length without clipping. Pseudo-localization (`--pseudo-loc`) runs in dev and in one visual regression suite.

## Bangla specifics

- Font: bundled Noto Sans Bengali for `bn`, with Inter fallback for Latin runs.
- Line-height increased to 1.6 for Bangla body text to avoid conjunct clipping.
- Bangla docs exist for the executive summary and onboarding, not for specs ([doc conventions](../00-overview/04-doc-conventions.md)).

## Testing

| Layer | Tool / check |
|---|---|
| Static | `eslint-plugin-jsx-a11y`, no-`outline:none` lint rule |
| Unit | `jest-axe` on every component fixture, zero violations |
| Integration | Playwright keyboard-only walkthroughs of the five core flows |
| Contrast | Automated token-pair contrast test across 2 themes × 5 accents |
| Screen reader | Manual VoiceOver + NVDA script per release, recorded in the release checklist |
| i18n | Missing/orphan key check, pseudo-loc visual diff, RTL screenshot suite |

Gate: zero `jest-axe` violations, zero contrast failures, keyboard walkthroughs green. Details in [test strategy](../08-quality/01-test-strategy.md).

## Related documents

- [Design system](01-design-system.md)
- [Keyboard and command palette](08-keyboard-and-command-palette.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
