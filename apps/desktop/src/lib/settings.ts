import { useState, useEffect } from "react";
import { Theme, Accent, Density, ThemeManager } from "./theme";
import { ipcClient } from "./ipc";
import { refreshTerminalThemes } from "./terminal-session";

export type UserPlan = "Free" | "Pro" | "Team" | "Enterprise";

export interface UserProfile {
  name: string;
  preferredName: string;
  email: string;
  role: string;
  plan: UserPlan;
  avatar: string;
  instructions: string;
}

export interface ModelConfig {
  activeModelId: string;
  activeModelLabel: string;
  activeProvider: string;
  defaultMaxTokens: number;
}

export interface PrivacyConfig {
  telemetryEnabled: boolean;
  shareCrashReports: boolean;
  dataResidency: "US-East (Virginia)" | "EU-Central (Frankfurt)" | "Local Only (Air-gapped)";
  retentionDays: number;
}

export interface BillingConfig {
  plan: UserPlan;
  monthlySpentUsd: number;
  budgetLimitUsd: number;
  tokensUsedThisMonth: number;
  tokensLimitThisMonth: number;
  cycleRenewalDate: string;
}

export interface CapabilitiesConfig {
  enableWebTools: boolean;
  enableImageInspection: boolean;
  enableImageGeneration: boolean;
  enableTerminalExecution: boolean;
  enableBackgroundWatchers: boolean;
  enableSubagents: boolean;
}

export interface TimeFocusConfig {
  autoSummarizeThresholdTurns: number;
  turnTimeoutSeconds: number;
  soundAlerts: boolean;
  compactTimeline: boolean;
}

export interface DesktopAppConfig {
  chatFont: "System UI" | "JetBrains Mono" | "Inter" | "SF Pro";
  motionPreference: "System Default" | "Standard" | "Reduced Motion";
  startupScreen: "Home" | "Code";
  defaultWorkspace: string;
}

export interface PermissionsConfig {
  autoApproveRead: boolean;
  requireDiffApproval: boolean;
  allowNetworkInSandbox: boolean;
  standingGrants: Array<{ id: string; command: string; scope: string }>;
}

export interface MemoryConfig {
  autoIndexWorkspace: boolean;
  maxContextTokens: number;
  includeGitHistory: boolean;
  persistedSnippetsCount: number;
}

export interface McpServerItem {
  id: string;
  name: string;
  description: string;
  toolsCount: number;
  status: "ready" | "idle" | "error";
  command?: string;
}

export interface ExtensionItem {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  enabled: boolean;
}

/// This is the authoritative settings schema. The index signature exists so
/// the object satisfies `SettingsBlob` at the IPC boundary, where the Rust
/// side round-trips unknown fields opaquely.
export interface AppSettings {
  profile: UserProfile;
  model: ModelConfig;
  privacy: PrivacyConfig;
  billing: BillingConfig;
  capabilities: CapabilitiesConfig;
  time: TimeFocusConfig;
  app: DesktopAppConfig;
  permissions: PermissionsConfig;
  memory: MemoryConfig;
  mcpServers: McpServerItem[];
  extensions: ExtensionItem[];
  theme: Theme;
  accent: Accent;
  density: Density;
  [key: string]: unknown;
}

/**
 * Shipping defaults. These are neutral on purpose: anything that describes the
 * user, their spend, their servers or their extensions starts empty and is
 * filled in from the signed-in account or the user's own configuration.
 */
const DEFAULT_SETTINGS: AppSettings = {
  profile: {
    name: "",
    preferredName: "",
    email: "",
    role: "",
    plan: "Free",
    avatar: "",
    instructions: "",
  },
  model: {
    activeModelId: "gpt-4o-mini",
    activeModelLabel: "GPT-4o mini",
    activeProvider: "OpenAI",
    defaultMaxTokens: 200000,
  },
  privacy: {
    telemetryEnabled: true,
    shareCrashReports: true,
    dataResidency: "US-East (Virginia)",
    retentionDays: 30,
  },
  billing: {
    plan: "Free",
    monthlySpentUsd: 0,
    budgetLimitUsd: 0,
    tokensUsedThisMonth: 0,
    tokensLimitThisMonth: 0,
    cycleRenewalDate: "",
  },
  capabilities: {
    enableWebTools: true,
    enableImageInspection: true,
    enableImageGeneration: false,
    enableTerminalExecution: true,
    enableBackgroundWatchers: true,
    enableSubagents: true,
  },
  time: {
    autoSummarizeThresholdTurns: 15,
    turnTimeoutSeconds: 120,
    soundAlerts: false,
    compactTimeline: false,
  },
  app: {
    chatFont: "System UI",
    motionPreference: "System Default",
    startupScreen: "Code",
    defaultWorkspace: "",
  },
  permissions: {
    autoApproveRead: true,
    requireDiffApproval: true,
    allowNetworkInSandbox: false,
    standingGrants: [],
  },
  memory: {
    autoIndexWorkspace: true,
    maxContextTokens: 200000,
    includeGitHistory: true,
    persistedSnippetsCount: 0,
  },
  mcpServers: [],
  extensions: [],
  theme: "light",
  accent: "ember",
  density: "default",
};

/// Legacy plaintext key. Settings now live in the encrypted local store; this
/// is only read once, to migrate an existing install, and then cleared.
const LEGACY_STORAGE_KEY = "eury_app_settings_v1";

type SettingsListener = (settings: AppSettings) => void;
const listeners = new Set<SettingsListener>();

function mergeWithDefaults(parsed: Partial<AppSettings>): AppSettings {
  return {
    ...DEFAULT_SETTINGS,
    ...parsed,
    profile: { ...DEFAULT_SETTINGS.profile, ...(parsed.profile || {}) },
    model: { ...DEFAULT_SETTINGS.model, ...(parsed.model || {}) },
    privacy: { ...DEFAULT_SETTINGS.privacy, ...(parsed.privacy || {}) },
    billing: { ...DEFAULT_SETTINGS.billing, ...(parsed.billing || {}) },
    capabilities: { ...DEFAULT_SETTINGS.capabilities, ...(parsed.capabilities || {}) },
    time: { ...DEFAULT_SETTINGS.time, ...(parsed.time || {}) },
    app: { ...DEFAULT_SETTINGS.app, ...(parsed.app || {}) },
    permissions: { ...DEFAULT_SETTINGS.permissions, ...(parsed.permissions || {}) },
    memory: { ...DEFAULT_SETTINGS.memory, ...(parsed.memory || {}) },
    theme: parsed.theme || ThemeManager.getTheme(),
    accent: parsed.accent || ThemeManager.getAccent(),
    density: parsed.density || ThemeManager.getDensity(),
  };
}

function applyThemeFromSettings(next: AppSettings, prev?: AppSettings): void {
  if (!prev || next.theme !== prev.theme) ThemeManager.setTheme(next.theme);
  if (!prev || next.accent !== prev.accent) ThemeManager.setAccent(next.accent);
  if (!prev || next.density !== prev.density) ThemeManager.setDensity(next.density);
  if (
    !prev ||
    next.theme !== prev.theme ||
    next.accent !== prev.accent
  ) {
    refreshTerminalThemes();
  }
}

/// Reads any settings left in plaintext `localStorage` by an older build, so
/// upgrading doesn't silently reset the user's configuration.
function readLegacySettings(): AppSettings | null {
  try {
    const raw = localStorage.getItem(LEGACY_STORAGE_KEY);
    if (!raw) return null;
    return mergeWithDefaults(JSON.parse(raw));
  } catch {
    return null;
  }
}

let currentSettings: AppSettings = readLegacySettings() ?? DEFAULT_SETTINGS;

// Apply cached settings before async hydrate so the first React paint matches
// the user's preference instead of index.html's static defaults.
ThemeManager.init(
  currentSettings.theme,
  currentSettings.accent,
  currentSettings.density,
);

export const SettingsStore = {
  get(): AppSettings {
    return currentSettings;
  },

  /// Loads settings from the encrypted store into the in-memory cache. The
  /// cache stays synchronous so callers don't have to become async; this runs
  /// once at startup. If a legacy plaintext blob exists it is migrated into
  /// the encrypted store and then removed.
  async hydrate(): Promise<AppSettings> {
    const legacy = readLegacySettings();
    try {
      const stored = await ipcClient.settings.get();
      currentSettings = mergeWithDefaults(stored as Partial<AppSettings>);
    } catch {
      // Store unavailable (e.g. running outside Tauri): keep what we have.
      return currentSettings;
    }

    if (legacy) {
      // Legacy wins on first migration only — the encrypted store has no
      // prior value for this install yet.
      currentSettings = legacy;
      try {
        await ipcClient.settings.set(currentSettings);
        localStorage.removeItem(LEGACY_STORAGE_KEY);
      } catch {
        // Leave the legacy blob in place so the migration retries next launch.
      }
    }

    applyThemeFromSettings(currentSettings);
    listeners.forEach((listener) => listener(currentSettings));
    return currentSettings;
  },

  set(partial: Partial<AppSettings> | ((prev: AppSettings) => AppSettings)): AppSettings {
    const prev = currentSettings;
    const next = typeof partial === "function" ? partial(prev) : { ...prev, ...partial };
    currentSettings = next;
    // Write through to the encrypted store. Fire-and-forget keeps `set`
    // synchronous for its many call sites; a failed write leaves the
    // in-memory value in place rather than reverting under the user.
    void ipcClient.settings.set(next).catch(() => {});

    applyThemeFromSettings(next, prev);

    listeners.forEach((listener) => listener(next));
    return next;
  },

  updateProfile(profile: Partial<UserProfile>) {
    return this.set((prev) => ({
      ...prev,
      profile: { ...prev.profile, ...profile },
    }));
  },

  updateModel(model: Partial<ModelConfig>) {
    return this.set((prev) => ({
      ...prev,
      model: { ...prev.model, ...model },
    }));
  },

  updateBilling(billing: Partial<BillingConfig>) {
    return this.set((prev) => ({
      ...prev,
      billing: { ...prev.billing, ...billing },
      profile: billing.plan ? { ...prev.profile, plan: billing.plan } : prev.profile,
    }));
  },

  updatePrivacy(privacy: Partial<PrivacyConfig>) {
    return this.set((prev) => ({
      ...prev,
      privacy: { ...prev.privacy, ...privacy },
    }));
  },

  updateCapabilities(capabilities: Partial<CapabilitiesConfig>) {
    return this.set((prev) => ({
      ...prev,
      capabilities: { ...prev.capabilities, ...capabilities },
    }));
  },

  updateTime(time: Partial<TimeFocusConfig>) {
    return this.set((prev) => ({
      ...prev,
      time: { ...prev.time, ...time },
    }));
  },

  updateApp(app: Partial<DesktopAppConfig>) {
    return this.set((prev) => ({
      ...prev,
      app: { ...prev.app, ...app },
    }));
  },

  updatePermissions(permissions: Partial<PermissionsConfig>) {
    return this.set((prev) => ({
      ...prev,
      permissions: { ...prev.permissions, ...permissions },
    }));
  },

  updateMemory(memory: Partial<MemoryConfig>) {
    return this.set((prev) => ({
      ...prev,
      memory: { ...prev.memory, ...memory },
    }));
  },

  addMcpServer(server: Omit<McpServerItem, "id">) {
    const newServer: McpServerItem = {
      ...server,
      id: "mcp-" + Date.now(),
    };
    return this.set((prev) => ({
      ...prev,
      mcpServers: [...prev.mcpServers, newServer],
    }));
  },

  removeMcpServer(id: string) {
    return this.set((prev) => ({
      ...prev,
      mcpServers: prev.mcpServers.filter((s) => s.id !== id),
    }));
  },

  toggleExtension(id: string) {
    return this.set((prev) => ({
      ...prev,
      extensions: prev.extensions.map((ext) => (ext.id === id ? { ...ext, enabled: !ext.enabled } : ext)),
    }));
  },

  subscribe(listener: SettingsListener): () => void {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  },
};

export function useAppSettings(): [AppSettings, typeof SettingsStore] {
  const [settings, setSettings] = useState<AppSettings>(() => SettingsStore.get());

  useEffect(() => {
    return SettingsStore.subscribe((updated) => {
      setSettings({ ...updated });
    });
  }, []);

  return [settings, SettingsStore];
}
