/**
 * Schemes the OS can be asked to open on the user's behalf.
 *
 * A link in a transcript or in terminal output is model- or program-authored
 * text, so the scheme is checked before it is handed anywhere: `javascript:`
 * and `data:` would execute, and `file:` would open something local that no
 * link in a conversation has any business reaching.
 */
const OPENABLE_SCHEMES = new Set(["http:", "https:", "mailto:"]);

export function isOpenableUrl(url: string): boolean {
  try {
    return OPENABLE_SCHEMES.has(new URL(url).protocol);
  } catch {
    return false;
  }
}

/** Whether the Tauri runtime is present, as opposed to a plain browser. */
function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Open a URL in the system default browser (Tauri) or a new tab (web). */
export async function openExternalUrl(url: string): Promise<void> {
  if (!isOpenableUrl(url)) {
    throw new Error("That link cannot be opened.");
  }

  // Tested on the runtime, not on whether the import resolves: outside Tauri
  // the plugin module still loads fine and only fails when called, and
  // catching that call would report "could not open browser" for what is
  // really a denied permission.
  if (isTauri()) {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(url);
    return;
  }

  const opened = window.open(url, "_blank", "noopener,noreferrer");
  if (!opened) {
    throw new Error("Could not open browser. Please open the link manually.");
  }
}
