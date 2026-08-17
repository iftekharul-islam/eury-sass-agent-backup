import { invoke } from "@tauri-apps/api/core";

export type PlatformType = "macos" | "windows" | "linux" | "unknown";

let detectedPlatform: PlatformType = (() => {
  if (typeof navigator === "undefined") return "macos";
  if (/Mac|iPhone|iPad|iPod/.test(navigator.platform) || /Macintosh/.test(navigator.userAgent)) {
    return "macos";
  }
  if (/Win/.test(navigator.platform) || /Windows/.test(navigator.userAgent)) {
    return "windows";
  }
  if (/Linux/.test(navigator.platform) && !/Android/.test(navigator.userAgent)) {
    return "linux";
  }
  return "macos";
})();

// Query native backend platform asynchronously on startup
try {
  invoke<string>("app_get_platform")
    .then((p) => {
      if (p === "macos" || p === "windows" || p === "linux") {
        detectedPlatform = p;
      }
    })
    .catch(() => {});
} catch {
  // Not in Tauri runtime
}

export const Platform = {
  get(): PlatformType {
    return detectedPlatform;
  },
  isMac(): boolean {
    return detectedPlatform === "macos";
  },
  isWindows(): boolean {
    return detectedPlatform === "windows";
  },
  isLinux(): boolean {
    return detectedPlatform === "linux";
  },
};

export async function getTauriWindow() {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    return getCurrentWindow();
  } catch {
    return null;
  }
}

export async function windowClose(): Promise<void> {
  try {
    const win = await getTauriWindow();
    if (win) {
      await win.close();
      return;
    }
  } catch {
    // fallback to invoke
  }
  try {
    await invoke("window_close");
  } catch {
    // Outside Tauri
  }
}

export async function windowMinimize(): Promise<void> {
  try {
    const win = await getTauriWindow();
    if (win) {
      await win.minimize();
      return;
    }
  } catch {
    // fallback to invoke
  }
  try {
    await invoke("window_minimize");
  } catch {
    // Outside Tauri
  }
}

export async function windowToggleMaximize(): Promise<void> {
  try {
    const win = await getTauriWindow();
    if (win) {
      await win.toggleMaximize();
      return;
    }
  } catch {
    // fallback to invoke
  }
  try {
    await invoke("window_toggle_maximize");
  } catch {
    // Outside Tauri
  }
}

export async function windowToggleFullscreen(): Promise<void> {
  try {
    const win = await getTauriWindow();
    if (win) {
      const isFull = await win.isFullscreen();
      await win.setFullscreen(!isFull);
      return;
    }
  } catch {
    // fallback to invoke
  }
  try {
    await invoke("window_toggle_fullscreen");
  } catch {
    // Outside Tauri
  }
}

export async function windowIsMaximized(): Promise<boolean> {
  try {
    const win = await getTauriWindow();
    if (win) {
      return await win.isMaximized();
    }
  } catch {
    // fallback to invoke
  }
  try {
    return await invoke<boolean>("window_is_maximized");
  } catch {
    return false;
  }
}

export async function windowStartDragging(): Promise<void> {
  try {
    const win = await getTauriWindow();
    if (win) {
      await win.startDragging();
    }
  } catch {
    // Outside Tauri
  }
}
