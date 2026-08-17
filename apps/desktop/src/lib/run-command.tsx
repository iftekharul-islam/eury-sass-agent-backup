import * as React from "react";

/**
 * How a shell snippet in the transcript reaches a real shell.
 *
 * The transcript renders markdown several components deep, and the terminal is
 * owned by the app shell, so this is a context rather than a prop threaded
 * through `MarkdownRenderer` — which is memoized on its content and would have
 * to re-render on every callback identity change.
 *
 * `null` means no terminal is available (the Home chat has no workspace), and
 * the Run affordance stays hidden rather than offering something that cannot
 * work.
 */
export type RunCommandHandler = ((command: string) => void) | null;

const RunCommandContext = React.createContext<RunCommandHandler>(null);

export function RunCommandProvider({
  onRunCommand,
  children,
}: {
  onRunCommand: RunCommandHandler;
  children: React.ReactNode;
}) {
  return (
    <RunCommandContext.Provider value={onRunCommand}>{children}</RunCommandContext.Provider>
  );
}

export function useRunCommand(): RunCommandHandler {
  return React.useContext(RunCommandContext);
}

const SHELL_LANGUAGES = new Set(["bash", "sh", "shell", "zsh", "console", "terminal"]);

export function isShellLanguage(language?: string): boolean {
  return SHELL_LANGUAGES.has((language ?? "").toLowerCase());
}

/**
 * Strips the decoration a model puts in front of a command it is showing —
 * `$` or `>` prompts — and drops comment-only and blank lines, so what runs is
 * what the block shows minus the typography.
 */
export function toRunnableCommand(code: string): string {
  return code
    .split("\n")
    .map((line) => line.replace(/^\s*[$>]\s+/, "").trimEnd())
    .filter((line) => line.trim() && !line.trim().startsWith("#"))
    .join("\n")
    .trim();
}
