export const AGENT_AUTH_COMPLETE_URL = "eury-agent://auth/complete";

/** Focus the desktop window when the browser deep-links back after authorization. */
export async function setupDeepLinks(onAuthComplete?: () => void): Promise<(() => void) | void> {
  try {
    const { getCurrent, onOpenUrl } = await import("@tauri-apps/plugin-deep-link");
    const { getCurrentWindow } = await import("@tauri-apps/api/window");

    const handleUrls = async (urls: string[]) => {
      for (const url of urls) {
        if (!url.includes("auth/complete")) continue;
        const win = getCurrentWindow();
        await win.show();
        await win.unminimize();
        await win.setFocus();
        onAuthComplete?.();
      }
    };

    const initial = await getCurrent();
    if (initial?.length) {
      void handleUrls(initial);
    }

    return await onOpenUrl((urls) => {
      void handleUrls(urls);
    });
  } catch {
    // Outside Tauri runtime
  }
}
