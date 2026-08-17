# Keyboard and Command Palette

Spec-Version: 1.3.0

`Cmd` on macOS, `Ctrl` on Windows/Linux. Written below as `Mod`. Every shortcut here is an accelerator for something that is also reachable by mouse, per [visual language](00-visual-language.md).

## Global

| Keys | Command |
|---|---|
| `Mod+K` | Command palette |
| `Mod+P` | Quick open file |
| `Mod+Shift+P` | Command palette (commands only) |
| `Mod+Shift+F` | Search across workspace |
| `Mod+,` | Open Settings; close it when already open |
| `Mod+Shift+N` | New window |
| `Mod+W` | Close the current pane, or the window when the conversation is the only pane |
| `Mod+O` | Open project folder |
| `Mod+Shift+O` | Recent projects |
| `Mod+Alt+H` | Switch to Home |
| `Mod+Alt+C` | Switch to Code |
| `Mod+1..9` | Switch to the conversation at that position in the sidebar |
| `Mod+Alt+←` / `Mod+Alt+→` | Back and forward through pane history |
| `F6` / `Shift+F6` | Cycle regions: sidebar, active pane, composer, status bar |
| `Mod+B` | Toggle the sidebar |
| `Mod+Shift+G` | Changes pane |
| `Mod+Shift+T` | Toggle theme |
| `Mod++` / `Mod+-` / `Mod+0` | Zoom in / out / reset |
| `Mod+Shift+I` | DevTools (dev builds only) |

## Chat and runs

| Keys | Command |
|---|---|
| `Enter` | Send (or queue if a run is active) |
| `Shift+Enter` | Newline |
| `Mod+Enter` | Send as steering message during a run |
| `Esc` `Esc` | Abort the current run |
| `Mod+L` | Focus composer |
| `Mod+Shift+L` | Clear composer |
| `Mod+/` | Slash-command menu |
| `@` | File/symbol mention picker |
| `Mod+Shift+M` | Model picker |
| `Mod+Shift+D` | Mode switcher |
| `Mod+Shift+C` | Copy last assistant message |
| `Mod+R` | Retry last assistant turn |
| `Mod+Shift+R` | Retry with a different model |
| `Mod+Shift+K` | New conversation |
| `Mod+Shift+H` | Conversation history |
| `Alt+↑` / `Alt+↓` | Previous / next turn |
| `End` | Jump to latest, re-attach autoscroll |
| `Mod+Shift+Backspace` | Delete last turn (confirm) |

## Approvals

| Keys | Command |
|---|---|
| `Esc` | Deny, from anywhere in the app |
| `Enter` | Activate the focused button, which starts on the safest one |
| `Mod+Enter` | Allow with the primary button's scope |
| `Tab` / `Shift+Tab` | Move between the card's buttons in visual order |
| `Mod+Shift+A` | Open the Approvals pane |

No single-key allow shortcut exists, by design, and buttons stay inert for 400 ms after the card renders. Runs and Plans have no dedicated shortcut: they open from the sidebar or the palette, which keeps the global map small enough to remember.

## Diffs and changes

| Keys | Command |
|---|---|
| `Mod+Shift+U` | Toggle unified / split view |
| `Alt+↑` / `Alt+↓` | Previous / next hunk |
| `Mod+Alt+Y` | Apply focused hunk |
| `Mod+Alt+N` | Skip focused hunk |
| `Mod+Alt+A` | Apply all hunks in file |
| `Mod+Shift+Z` | Restore the checkpoint for the focused turn (confirm) |

## Editor

| Keys | Command |
|---|---|
| `Mod+S` | Save |
| `Mod+F` / `Mod+Alt+F` | Find / find and replace |
| `Mod+G` | Go to line |
| `Mod+D` | Add cursor at next occurrence |
| `Mod+]` / `Mod+[` | Indent / outdent |
| `Mod+Shift+E` | Reveal file in OS file manager |
| `Mod+Shift+Y` | Ask about the current selection |

## Terminal

| Keys | Command |
|---|---|
| `Mod+`` ` | New / focus terminal |
| `Mod+Shift+`` ` | New terminal session |
| `Mod+Shift+X` | Kill focused terminal |
| `Mod+Shift+V` | Send selection to composer |
| `Mod+K` (terminal focused) | Clear terminal (does not open palette) |

When the terminal has focus, all other key input goes to the PTY except the shortcuts listed here and the global window shortcuts.

## Command palette

`Mod+K` opens a single fuzzy input over multiple providers.

| Prefix | Scope |
|---|---|
| *(none)* | Mixed: commands, files, runs, plans, settings |
| `>` | Commands only |
| `@` | Symbols in the active file |
| `#` | Workspace symbols |
| `:` | Go to line in the active file |
| `?` | Help — lists prefixes |
| `!` | Recent runs |
| `~` | Recent workspaces |

| Behavior | Rule |
|---|---|
| Ranking | Fuzzy score × recency × frequency (per-user MRU stored in `settings`) |
| Result cap | 50 rendered, virtualized |
| Latency | ≤ 50 ms to first results on a 100k-file workspace (index-backed) |
| Keyboard | `↑/↓` move, `Enter` run, `Mod+Enter` run in a new window, `Esc` close |
| Preview | Files show a 10-line peek; commands show their keybinding |
| Dangerous commands | Marked with a risk badge; still require the normal approval flow |
| Registry | Every command is declared in a single `commands.ts` registry with id, title, category, keybinding, `when` clause, and handler |

Commands are the only way features are invoked from the palette; there is no free-text command execution.

## Registry contract

```typescript
interface Command {
  id: string;                    // "chat.retryTurn"
  title: string;                 // i18n key resolved at render
  category: "Chat" | "Run" | "File" | "Diff" | "Terminal" | "Approvals" | "Settings" | "Window";
  keybinding?: string;           // "Mod+R"
  when?: string;                 // "chatFocused && !runActive"
  risk?: "low" | "medium" | "elevated";
  run(ctx: CommandContext): void | Promise<void>;
}
```

`when` clauses evaluate against a context key set (`runActive`, `workspaceTrusted`, `editorFocused`, `terminalFocused`, `approvalPending`, `offline`). Disabled commands remain visible in the palette with the reason shown, so users learn the model instead of hunting for missing entries.

## Customization

Settings → Keyboard lists every command with its binding, supports rebinding, conflict detection (blocking), and reset-to-default. Bindings persist in `settings` as `ui.keybindings`. Reserved and non-rebindable: `Esc` in approvals, `Mod+K`, `Mod+,`, and OS-level window shortcuts.

## Discoverability

Tooltips show bindings using platform glyphs (`⌘⇧K`). A first-run tour highlights palette, mode switcher, and approvals. `Mod+K` then `?` documents every prefix.

## Related documents

- [App shell and navigation](02-app-shell-and-navigation.md)
- [Accessibility and i18n](07-accessibility-and-i18n.md)
- [Approval and trust UX](05-approval-and-trust-ux.md)
