import * as React from "react";
import { Icon } from "./Icons";
import { useAppSettings, UserPlan, PrivacyConfig, DesktopAppConfig } from "../lib/settings";
import { Theme, Accent, Density } from "../lib/theme";
import { cloudApi, formatMicrosUsd, type AgentUsageResponse } from "../lib/cloud";
import { ipcClient, type Capabilities } from "../lib/ipc";

export interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface NavItem {
  id: string;
  label: string;
  icon: string;
  section: "Profile" | "Desktop app" | "Code";
}

const NAV_ITEMS: NavItem[] = [
  { id: "profile-general", label: "General", icon: "settings", section: "Profile" },
  { id: "account", label: "Models & Gateway", icon: "spark", section: "Profile" },
  { id: "privacy", label: "Privacy & Telemetry", icon: "shield", section: "Profile" },
  { id: "billing", label: "Billing & Quotas", icon: "circle", section: "Profile" },
  { id: "capabilities", label: "Capabilities", icon: "spark", section: "Profile" },
  { id: "time", label: "Time and focus", icon: "clock", section: "Profile" },

  { id: "app-general", label: "General", icon: "settings", section: "Desktop app" },
  { id: "extensions", label: "Extensions", icon: "plug", section: "Desktop app" },
  { id: "developer", label: "Developer & Logs", icon: "terminal", section: "Desktop app" },

  { id: "permissions", label: "Permissions & Policy", icon: "shield", section: "Code" },
  { id: "memory", label: "Memory & Context", icon: "book", section: "Code" },
  { id: "mcp", label: "MCP servers", icon: "plug", section: "Code" },
  { id: "appearance", label: "Appearance", icon: "eye", section: "Code" },
];

export function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const [settings, store] = useAppSettings();
  const [search, setSearch] = React.useState("");
  const [activeTab, setActiveTab] = React.useState("profile-general");
  const [newMcpName, setNewMcpName] = React.useState("");
  const [newMcpCommand, setNewMcpCommand] = React.useState("");
  const [showAddMcp, setShowAddMcp] = React.useState(false);
  const [liveUsage, setLiveUsage] = React.useState<AgentUsageResponse | null>(null);
  const [usageError, setUsageError] = React.useState<string | null>(null);
  const [capabilities, setCapabilities] = React.useState<Capabilities | null>(null);

  React.useEffect(() => {
    if (!isOpen || activeTab !== "developer") return;
    let cancelled = false;
    ipcClient.capabilities
      .get()
      .then((caps) => {
        if (!cancelled) setCapabilities(caps);
      })
      .catch(() => {
        if (!cancelled) setCapabilities(null);
      });
    return () => {
      cancelled = true;
    };
  }, [isOpen, activeTab]);

  React.useEffect(() => {
    if (!isOpen || activeTab !== "billing") return;
    let cancelled = false;
    cloudApi
      .getUsage()
      .then((usage) => {
        if (!cancelled) {
          setLiveUsage(usage);
          setUsageError(null);
        }
      })
      .catch((e: Error) => {
        if (!cancelled) setUsageError(e.message);
      });
    return () => {
      cancelled = true;
    };
  }, [isOpen, activeTab]);

  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const handleThemeChange = (newTheme: Theme) => {
    store.set({ theme: newTheme });
  };

  const handleAccentChange = (newAccent: Accent) => {
    store.set({ accent: newAccent });
  };

  const handleDensityChange = (newDensity: Density) => {
    store.set({ density: newDensity });
  };

  const filteredItems = NAV_ITEMS.filter(
    (item) =>
      item.label.toLowerCase().includes(search.toLowerCase()) ||
      item.section.toLowerCase().includes(search.toLowerCase())
  );

  const sections: Array<"Profile" | "Desktop app" | "Code"> = ["Profile", "Desktop app", "Code"];

  const handleAddMcp = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newMcpName.trim()) return;
    store.addMcpServer({
      name: newMcpName.trim(),
      description: "Custom user-configured MCP server",
      toolsCount: 1,
      status: "ready",
      command: newMcpCommand.trim() || undefined,
    });
    setNewMcpName("");
    setNewMcpCommand("");
    setShowAddMcp(false);
  };

  return (
    <div className="scrim" onClick={onClose}>
      <section
        className="settings-modal"
        role="dialog"
        aria-modal="true"
        aria-label="Settings"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Left Searchable Nav */}
        <aside className="settings-nav">
          <div className="settings-search">
            <Icon name="search" />
            <input
              placeholder="Search settings"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              autoFocus
            />
          </div>

          {sections.map((sec) => {
            const items = filteredItems.filter((i) => i.section === sec);
            if (items.length === 0) return null;
            return (
              <React.Fragment key={sec}>
                <div className="sec-label">{sec}</div>
                {items.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    className={`row ${activeTab === item.id ? "on" : ""}`}
                    onClick={() => setActiveTab(item.id)}
                  >
                    <Icon name={item.icon} />
                    <span className="t">{item.label}</span>
                  </button>
                ))}
              </React.Fragment>
            );
          })}
        </aside>

        {/* Right Settings Form Body */}
        <main className="settings-body">
          <div className="settings-top">
            <h2>
              {NAV_ITEMS.find((i) => i.id === activeTab)?.section} —{" "}
              {NAV_ITEMS.find((i) => i.id === activeTab)?.label}
            </h2>
            <button
              type="button"
              className="tb-icon settings-close"
              aria-label="Close settings"
              onClick={onClose}
            >
              <Icon name="x" size={16} />
            </button>
          </div>

          {/* Tab: Profile General */}
          {activeTab === "profile-general" && (
            <>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>User Profile Avatar</div>
                  <div className="sub">Derived from your preferred name</div>
                </div>
                <span className="avatar-preview">
                  {settings.profile.avatar || settings.profile.preferredName[0] || "U"}
                </span>
              </div>

              <label className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Full name</div>
                  <div className="sub">Used for account and team workspaces</div>
                </div>
                <input
                  className="form-input"
                  value={settings.profile.name}
                  onChange={(e) => store.updateProfile({ name: e.target.value })}
                />
              </label>

              <label className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>What should Eury call you?</div>
                  <div className="sub">Assistant conversational greeting name</div>
                </div>
                <input
                  className="form-input"
                  value={settings.profile.preferredName}
                  onChange={(e) => {
                    const preferredName = e.target.value;
                    store.updateProfile({
                      preferredName,
                      avatar: preferredName.trim() ? preferredName.trim()[0].toUpperCase() : "U",
                    });
                  }}
                />
              </label>

              <label className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Email Address</div>
                  <div className="sub">Identity associated with your cloud session</div>
                </div>
                <input
                  className="form-input"
                  type="email"
                  value={settings.profile.email}
                  onChange={(e) => store.updateProfile({ email: e.target.value })}
                />
              </label>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Primary Role</div>
                  <div className="sub">Helps Eury calibrate code suggestions</div>
                </div>
                <select
                  className="form-input"
                  value={settings.profile.role}
                  onChange={(e) => store.updateProfile({ role: e.target.value })}
                >
                  <option value="Software Engineer">Software Engineer</option>
                  <option value="Senior Staff Engineer">Senior Staff Engineer</option>
                  <option value="Security Engineer">Security Engineer</option>
                  <option value="Full-Stack Developer">Full-Stack Developer</option>
                  <option value="Product Architect">Product Architect</option>
                </select>
              </div>

              <div style={{ padding: "14px 0", borderBottom: "1px solid var(--color-border)" }}>
                <div style={{ fontSize: "12px", fontWeight: 500, marginBottom: "3px" }}>
                  Instructions for Eury
                </div>
                <div className="hint" style={{ marginBottom: "8px" }}>
                  Eury keeps these instructions in mind across Home chats, Plan mode, and Code work.
                </div>
                <textarea
                  className="form-area"
                  placeholder="e.g. I prefer direct answers, and clean typed modular code."
                  value={settings.profile.instructions}
                  onChange={(e) => store.updateProfile({ instructions: e.target.value })}
                />
              </div>
            </>
          )}

          {/* Tab: Models & Gateway */}
          {activeTab === "account" && (
            <>
              <div className="sec-label">Eury Managed Gateway</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Active Foundation Model</div>
                  <div className="sub">
                    All inference routes through the Eury cloud gateway. Select models in the composer.
                  </div>
                </div>
                <div className="sub" style={{ textAlign: "right" }}>
                  {settings.model.activeModelLabel}
                  <br />
                  <span style={{ opacity: 0.7 }}>
                    {settings.model.activeProvider}/{settings.model.activeModelId}
                  </span>
                </div>
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Connection Mode</div>
                  <div className="sub">Managed gateway only — no provider API keys on device</div>
                </div>
                <div className="seg">
                  <span className="on">Gateway</span>
                </div>
              </div>

              <div className="sec-label" style={{ marginTop: "18px" }}>Capabilities</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Web search & fetch</div>
                  <div className="sub">Proxied through Eury with SSRF protections</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableWebTools}
                  onChange={(e) => store.updateCapabilities({ enableWebTools: e.target.checked })}
                />
              </div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Vision attachments</div>
                  <div className="sub">Pass screenshots and images into model context</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableImageInspection}
                  onChange={(e) => store.updateCapabilities({ enableImageInspection: e.target.checked })}
                />
              </div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Image generation</div>
                  <div className="sub">Consent-gated — agent may call generate_image when enabled</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableImageGeneration}
                  onChange={(e) => {
                    if (e.target.checked) {
                      const ok = window.confirm(
                        "Enable image generation? Generated images are untrusted until you explicitly save them to your workspace.",
                      );
                      if (!ok) return;
                    }
                    store.updateCapabilities({ enableImageGeneration: e.target.checked });
                  }}
                />
              </div>
            </>
          )}

          {/* Tab: Privacy & Telemetry */}
          {activeTab === "privacy" && (
            <>
              <div className="sec-label">Data Protection & Telemetry</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Anonymous Usage Telemetry</div>
                  <div className="sub">Help improve tool latency and crash diagnostics</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.privacy.telemetryEnabled}
                  onChange={(e) => store.updatePrivacy({ telemetryEnabled: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Share Automated Crash Reports</div>
                  <div className="sub">Submit sanitized stack traces when tool execution panics</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.privacy.shareCrashReports}
                  onChange={(e) => store.updatePrivacy({ shareCrashReports: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Data Residency Region</div>
                  <div className="sub">Regional endpoint for index synchronization and audit log signing</div>
                </div>
                <select
                  className="form-input"
                  value={settings.privacy.dataResidency}
                  onChange={(e) =>
                    store.updatePrivacy({
                      dataResidency: e.target.value as PrivacyConfig["dataResidency"],
                    })
                  }
                >
                  <option value="US-East (Virginia)">US-East (Virginia)</option>
                  <option value="EU-Central (Frankfurt)">EU-Central (Frankfurt)</option>
                  <option value="Local Only (Air-gapped)">Local Only (Air-gapped)</option>
                </select>
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Local Checkpoint & Run Retention</div>
                  <div className="sub">Days to retain past runs and rollback snapshots in SQLite</div>
                </div>
                <select
                  className="form-input"
                  value={settings.privacy.retentionDays}
                  onChange={(e) => store.updatePrivacy({ retentionDays: Number(e.target.value) })}
                >
                  <option value={7}>7 Days</option>
                  <option value={14}>14 Days</option>
                  <option value={30}>30 Days (Recommended)</option>
                  <option value={90}>90 Days</option>
                  <option value={365}>1 Year</option>
                </select>
              </div>
            </>
          )}

          {/* Tab: Billing & Quotas */}
          {activeTab === "billing" && (
            <>
              <div className="sec-label">Active Subscription Tier</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Current Plan Tier</div>
                  <div className="sub">Changes entitlements, cloud quota, and gateway access</div>
                </div>
                <div className="seg">
                  {(["Free", "Pro", "Team", "Enterprise"] as UserPlan[]).map((p) => (
                    <span
                      key={p}
                      className={settings.billing.plan === p ? "on" : ""}
                      onClick={() => store.updateBilling({ plan: p })}
                    >
                      {p}
                    </span>
                  ))}
                </div>
              </div>

              <div className="sec-label" style={{ marginTop: "18px" }}>Resource Quotas & Spend</div>
              {usageError && (
                <div className="form-row">
                  <div className="sub" style={{ color: "var(--color-warning)" }}>
                    Could not load live usage: {usageError}
                  </div>
                </div>
              )}
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Monthly Model Spend</div>
                  <div className="sub">
                    {liveUsage
                      ? `${formatMicrosUsd(liveUsage.limits.monthlyBudgetUsdMicros.used)} of ${formatMicrosUsd(liveUsage.limits.monthlyBudgetUsdMicros.limit)} monthly ceiling`
                      : `$${settings.billing.monthlySpentUsd.toFixed(2)} of $${settings.billing.budgetLimitUsd.toFixed(2)} monthly ceiling`}
                    {liveUsage?.estimatesAreApproximate ? " (approx.)" : ""}
                  </div>
                </div>
                <div style={{ width: "100%", background: "var(--color-bg-inset)", height: "8px", borderRadius: "4px", overflow: "hidden" }}>
                  <div
                    style={{
                      width: `${(() => {
                        const used = liveUsage
                          ? liveUsage.limits.monthlyBudgetUsdMicros.used
                          : settings.billing.monthlySpentUsd;
                        const limit = liveUsage
                          ? liveUsage.limits.monthlyBudgetUsdMicros.limit
                          : settings.billing.budgetLimitUsd;
                        // No known ceiling yet (signed out, or usage not loaded)
                        // — an empty bar beats a NaN one.
                        return limit > 0 ? Math.min(100, (used / limit) * 100) : 0;
                      })()}%`,
                      height: "100%",
                      background: "var(--color-accent)",
                    }}
                  />
                </div>
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Token Consumption This Cycle</div>
                  <div className="sub">
                    {liveUsage
                      ? `${(liveUsage.limits.monthlyManagedTokens.used / 1000).toFixed(1)}k / ${(liveUsage.limits.monthlyManagedTokens.limit / 1000).toFixed(0)}k tokens`
                      : `${(settings.billing.tokensUsedThisMonth / 1000).toFixed(1)}k / ${(settings.billing.tokensLimitThisMonth / 1000).toFixed(0)}k tokens`}
                  </div>
                </div>
                <div style={{ width: "100%", background: "var(--color-bg-inset)", height: "8px", borderRadius: "4px", overflow: "hidden" }}>
                  <div
                    style={{
                      width: `${Math.min(
                        100,
                        liveUsage
                          ? (liveUsage.limits.monthlyManagedTokens.used / liveUsage.limits.monthlyManagedTokens.limit) * 100
                          : (settings.billing.tokensUsedThisMonth / settings.billing.tokensLimitThisMonth) * 100,
                      )}%`,
                      height: "100%",
                      background: "var(--color-success)",
                    }}
                  />
                </div>
              </div>

              {liveUsage && (
                <div className="form-row">
                  <div>
                    <div style={{ fontWeight: 500 }}>Daily Managed Runs</div>
                    <div className="sub">
                      {liveUsage.limits.dailyManagedRuns.used} / {liveUsage.limits.dailyManagedRuns.limit} today
                    </div>
                  </div>
                </div>
              )}

              <label className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Monthly Spend Cap (USD)</div>
                  <div className="sub">Automatic guardrail preventing runaway model loops</div>
                </div>
                <input
                  className="form-input"
                  type="number"
                  value={settings.billing.budgetLimitUsd}
                  onChange={(e) => store.updateBilling({ budgetLimitUsd: Number(e.target.value) || 10 })}
                />
              </label>
            </>
          )}

          {/* Tab: Capabilities */}
          {activeTab === "capabilities" && (
            <>
              <div className="sec-label">Agent Tool Capabilities</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Web Search & Browsing Tools</div>
                  <div className="sub">Enables google search, cdp navigation, and web fetch</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableWebTools}
                  onChange={(e) => store.updateCapabilities({ enableWebTools: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Multimodal Image Inspection</div>
                  <div className="sub">Allows passing screenshots and images into model context</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableImageInspection}
                  onChange={(e) => store.updateCapabilities({ enableImageInspection: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Direct Terminal Bridge (PTY)</div>
                  <div className="sub">Enables running bash / zsh subprocesses in sandbox namespaces</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableTerminalExecution}
                  onChange={(e) => store.updateCapabilities({ enableTerminalExecution: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Background File & CI Watchers</div>
                  <div className="sub">Listens for test failures and rebuild events automatically</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableBackgroundWatchers}
                  onChange={(e) => store.updateCapabilities({ enableBackgroundWatchers: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Autonomous Subagent Delegation</div>
                  <div className="sub">Allows spawning specialized subagents for parallel research</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.capabilities.enableSubagents}
                  onChange={(e) => store.updateCapabilities({ enableSubagents: e.target.checked })}
                />
              </div>
            </>
          )}

          {/* Tab: Time and Focus */}
          {activeTab === "time" && (
            <>
              <div className="sec-label">Focus & Context Compression</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Auto-summarize conversation turns</div>
                  <div className="sub">Compact old history after N turns to keep token cost low</div>
                </div>
                <select
                  className="form-input"
                  value={settings.time.autoSummarizeThresholdTurns}
                  onChange={(e) => store.updateTime({ autoSummarizeThresholdTurns: Number(e.target.value) })}
                >
                  <option value={10}>Every 10 turns</option>
                  <option value={15}>Every 15 turns (Recommended)</option>
                  <option value={25}>Every 25 turns</option>
                  <option value={50}>Every 50 turns</option>
                </select>
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Turn Execution Timeout</div>
                  <div className="sub">Maximum allowed time for a single agent reasoning block</div>
                </div>
                <select
                  className="form-input"
                  value={settings.time.turnTimeoutSeconds}
                  onChange={(e) => store.updateTime({ turnTimeoutSeconds: Number(e.target.value) })}
                >
                  <option value={60}>60 seconds</option>
                  <option value={120}>120 seconds (Standard)</option>
                  <option value={300}>5 minutes (Complex build)</option>
                </select>
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Completion Sound Effect</div>
                  <div className="sub">Play subtle chime when long build completes in background</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.time.soundAlerts}
                  onChange={(e) => store.updateTime({ soundAlerts: e.target.checked })}
                />
              </div>
            </>
          )}

          {/* Tab: Desktop App General */}
          {activeTab === "app-general" && (
            <>
              <div className="sec-label">Desktop Environment</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Default Startup View</div>
                  <div className="sub">Area opened when application starts</div>
                </div>
                <div className="seg">
                  <span
                    className={settings.app.startupScreen === "Home" ? "on" : ""}
                    onClick={() => store.updateApp({ startupScreen: "Home" })}
                  >
                    Home
                  </span>
                  <span
                    className={settings.app.startupScreen === "Code" ? "on" : ""}
                    onClick={() => store.updateApp({ startupScreen: "Code" })}
                  >
                    Code
                  </span>
                </div>
              </div>

              <label className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Default Workspace</div>
                  <div className="sub">Workspace directory to load by default</div>
                </div>
                <input
                  className="form-input"
                  value={settings.app.defaultWorkspace}
                  onChange={(e) => store.updateApp({ defaultWorkspace: e.target.value })}
                />
              </label>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Chat & UI Font Family</div>
                  <div className="sub">Typography used throughout conversations</div>
                </div>
                <select
                  className="form-input"
                  value={settings.app.chatFont}
                  onChange={(e) =>
                    store.updateApp({ chatFont: e.target.value as DesktopAppConfig["chatFont"] })
                  }
                >
                  <option value="System UI">System UI (Native OS)</option>
                  <option value="Inter">Inter UI</option>
                  <option value="JetBrains Mono">JetBrains Mono</option>
                  <option value="SF Pro">SF Pro Display</option>
                </select>
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Animation Motion Preference</div>
                  <div className="sub">Smooth transitions vs reduced accessibility motion</div>
                </div>
                <select
                  className="form-input"
                  value={settings.app.motionPreference}
                  onChange={(e) =>
                    store.updateApp({
                      motionPreference: e.target.value as DesktopAppConfig["motionPreference"],
                    })
                  }
                >
                  <option value="System Default">System Default</option>
                  <option value="Standard">Standard Smooth</option>
                  <option value="Reduced Motion">Reduced Motion</option>
                </select>
              </div>
            </>
          )}

          {/* Tab: Extensions */}
          {activeTab === "extensions" && (
            <>
              <div className="sec-label">Installed Desktop Extensions</div>
              {settings.extensions.length === 0 && (
                <div className="sub">No extensions installed.</div>
              )}
              {settings.extensions.map((ext) => (
                <div key={ext.id} className="setting" style={{ marginBottom: "10px" }}>
                  <Icon name="plug" style={{ color: ext.enabled ? "var(--color-accent)" : "var(--color-fg-muted)" }} />
                  <div className="t">
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <b>{ext.name}</b>
                      <span className="mono" style={{ fontSize: "11px", color: "var(--color-fg-subtle)" }}>
                        v{ext.version}
                      </span>
                    </div>
                    <div className="sub">{ext.description}</div>
                  </div>
                  <button
                    type="button"
                    className={`btn sm ${ext.enabled ? "primary" : "ghost"}`}
                    onClick={() => store.toggleExtension(ext.id)}
                  >
                    {ext.enabled ? "Enabled" : "Disabled"}
                  </button>
                </div>
              ))}
            </>
          )}

          {/* Tab: Developer & Logs */}
          {activeTab === "developer" && (
            <>
              <div className="sec-label">Diagnostic & IPC Logs</div>
              <div className="form-row">
                <span>IPC Event Logging</span>
                <span className="status ok"><Icon name="check" />Active (RAF Batching)</span>
              </div>
              <div className="form-row">
                <span>Local Sandbox Engine</span>
                {capabilities === null ? (
                  <span className="mono">Checking…</span>
                ) : capabilities.sandbox.available ? (
                  <span className="status ok">
                    <Icon name="check" />
                    Verified ({capabilities.sandbox.kind})
                  </span>
                ) : (
                  <span className="status err">
                    <Icon name="alert" />
                    Not verified ({capabilities.sandbox.kind})
                  </span>
                )}
              </div>
              <div className="form-row">
                <span>Rust Core State</span>
                <span className="status ok"><Icon name="check" />Cersei embedded v0.2.6</span>
              </div>
              <div className="form-row">
                <span>Local Vault Storage</span>
                <span className="mono">~/.eury/vault.db (SQLite encrypted)</span>
              </div>

              <div style={{ display: "flex", gap: "10px", marginTop: "18px" }}>
                <button
                  type="button"
                  className="btn sm ghost"
                  onClick={() => alert("Local caches cleared.")}
                >
                  Clear Local Cache
                </button>
                <button
                  type="button"
                  className="btn sm ghost"
                  onClick={() => alert("Diagnostic bundle exported to ~/Downloads/eury-diagnostics.json")}
                >
                  Export Diagnostic Bundle
                </button>
              </div>
            </>
          )}

          {/* Tab: Permissions & Policy */}
          {activeTab === "permissions" && (
            <>
              <div className="sec-label">Execution Controls</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Auto-approve read operations</div>
                  <div className="sub">Allows file reads, grep, and directory listings without prompting</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.permissions.autoApproveRead}
                  onChange={(e) => store.updatePermissions({ autoApproveRead: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Require approval for file diffs</div>
                  <div className="sub">Shows unified/split diff card before modifying project files</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.permissions.requireDiffApproval}
                  onChange={(e) => store.updatePermissions({ requireDiffApproval: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Allow outbound network in sandbox</div>
                  <div className="sub">Strictly restrict shell sub-processes to local network or air-gapped</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.permissions.allowNetworkInSandbox}
                  onChange={(e) => store.updatePermissions({ allowNetworkInSandbox: e.target.checked })}
                />
              </div>

              <div className="sec-label" style={{ marginTop: "18px" }}>Standing Grants</div>
              {settings.permissions.standingGrants.length === 0 && (
                <div className="sub">No standing grants — every action is asked for individually.</div>
              )}
              {settings.permissions.standingGrants.map((g) => (
                <div key={g.id} className="setting" style={{ marginBottom: "8px" }}>
                  <Icon name="check" style={{ color: "var(--color-success)" }} />
                  <div className="t">
                    <span className="mono">{g.command}</span>
                    <div className="sub">{g.scope}</div>
                  </div>
                  <button
                    type="button"
                    className="btn sm ghost"
                    onClick={() =>
                      store.updatePermissions({
                        standingGrants: settings.permissions.standingGrants.filter((x) => x.id !== g.id),
                      })
                    }
                  >
                    Revoke
                  </button>
                </div>
              ))}
            </>
          )}

          {/* Tab: Memory & Context */}
          {activeTab === "memory" && (
            <>
              <div className="sec-label">Workspace Memory Index</div>
              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Auto-Index Project Workspace</div>
                  <div className="sub">Maintains AST embeddings and symbol index for low-latency lookups</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.memory.autoIndexWorkspace}
                  onChange={(e) => store.updateMemory({ autoIndexWorkspace: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Context Window Ceiling</div>
                  <div className="sub">Maximum context tokens assembled per turn before compaction</div>
                </div>
                <select
                  className="form-input"
                  value={settings.memory.maxContextTokens}
                  onChange={(e) => store.updateMemory({ maxContextTokens: Number(e.target.value) })}
                >
                  <option value={100000}>100,000 tokens</option>
                  <option value={200000}>200,000 tokens (Standard)</option>
                  <option value={500000}>500,000 tokens (Long context)</option>
                  <option value={1000000}>1,000,000 tokens</option>
                </select>
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Include Git Commit History</div>
                  <div className="sub">Attaches recent commits to help understand project design decisions</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.memory.includeGitHistory}
                  onChange={(e) => store.updateMemory({ includeGitHistory: e.target.checked })}
                />
              </div>

              <div className="form-row">
                <div>
                  <div style={{ fontWeight: 500 }}>Persisted Memory Snippets</div>
                  <div className="sub">{settings.memory.persistedSnippetsCount} persistent facts saved across sessions</div>
                </div>
                <button
                  type="button"
                  className="btn sm ghost"
                  onClick={() => {
                    store.updateMemory({ persistedSnippetsCount: 0 });
                    alert("Workspace memory cleared.");
                  }}
                >
                  Clear Memory
                </button>
              </div>
            </>
          )}

          {/* Tab: MCP Servers */}
          {activeTab === "mcp" && (
            <>
              <div className="sec-label">Configured MCP Servers</div>
              {settings.mcpServers.length === 0 && (
                <div className="sub">No MCP servers configured yet.</div>
              )}
              {settings.mcpServers.map((server) => (
                <div key={server.id} className="setting" style={{ marginBottom: "8px" }}>
                  <Icon
                    name="plug"
                    style={{
                      color: server.status === "ready" ? "var(--color-success)" : "var(--color-fg-muted)",
                    }}
                  />
                  <div className="t">
                    <b>{server.name}</b>
                    <div className="sub">
                      {server.description} · {server.toolsCount} tools active
                    </div>
                  </div>
                  <span className={`status ${server.status === "ready" ? "ok" : ""}`}>
                    <Icon name={server.status === "ready" ? "check" : "circle"} />
                    {server.status === "ready" ? "Ready" : "Idle"}
                  </span>
                  <button
                    type="button"
                    className="tb-icon"
                    title="Remove server"
                    onClick={() => store.removeMcpServer(server.id)}
                    style={{ marginLeft: "8px" }}
                  >
                    <Icon name="x" size={14} />
                  </button>
                </div>
              ))}

              {showAddMcp ? (
                <form
                  onSubmit={handleAddMcp}
                  style={{
                    padding: "12px",
                    background: "var(--color-bg-inset)",
                    borderRadius: "var(--r-md)",
                    border: "1px solid var(--color-border)",
                    marginTop: "12px",
                    display: "flex",
                    flexDirection: "column",
                    gap: "8px",
                  }}
                >
                  <div style={{ fontSize: "12px", fontWeight: 600 }}>Add MCP Server</div>
                  <input
                    className="form-input"
                    placeholder="Server Name (e.g. postgres-local)"
                    value={newMcpName}
                    onChange={(e) => setNewMcpName(e.target.value)}
                    required
                  />
                  <input
                    className="form-input"
                    placeholder="Command (e.g. npx -y @mcp/server-sqlite --db app.db)"
                    value={newMcpCommand}
                    onChange={(e) => setNewMcpCommand(e.target.value)}
                  />
                  <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end", marginTop: "4px" }}>
                    <button
                      type="button"
                      className="btn sm ghost"
                      onClick={() => setShowAddMcp(false)}
                    >
                      Cancel
                    </button>
                    <button type="submit" className="btn sm primary">
                      Save MCP Server
                    </button>
                  </div>
                </form>
              ) : (
                <button
                  type="button"
                  className="btn sm primary"
                  style={{ marginTop: "12px" }}
                  onClick={() => setShowAddMcp(true)}
                >
                  <Icon name="plus" /> Add MCP Server
                </button>
              )}
            </>
          )}

          {/* Tab: Appearance */}
          {activeTab === "appearance" && (
            <>
              <div className="sec-label">Theme and Palette</div>
              <div className="form-row">
                <span>Theme</span>
                <div className="seg">
                  <span
                    className={settings.theme === "light" ? "on" : ""}
                    onClick={() => handleThemeChange("light")}
                  >
                    Light
                  </span>
                  <span
                    className={settings.theme === "dark" ? "on" : ""}
                    onClick={() => handleThemeChange("dark")}
                  >
                    Dark
                  </span>
                </div>
              </div>

              <div className="form-row">
                <span>Density</span>
                <div className="seg">
                  <span
                    className={settings.density === "default" ? "on" : ""}
                    onClick={() => handleDensityChange("default")}
                  >
                    Default
                  </span>
                  <span
                    className={settings.density === "compact" ? "on" : ""}
                    onClick={() => handleDensityChange("compact")}
                  >
                    Compact
                  </span>
                </div>
              </div>

              <div className="form-row">
                <span>Accent Color</span>
                <div className="appearance-pills">
                  {(["ember", "teal", "blue", "violet", "rose"] as Accent[]).map((a) => (
                    <button
                      key={a}
                      type="button"
                      className={`tb-icon ${settings.accent === a ? "on" : ""}`}
                      onClick={() => handleAccentChange(a)}
                      title={`${a} accent`}
                      style={{ padding: "4px" }}
                    >
                      <span
                        style={{
                          display: "inline-block",
                          width: "14px",
                          height: "14px",
                          borderRadius: "50%",
                          background: `var(--${a}-500)`,
                        }}
                      />
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}
        </main>
      </section>
    </div>
  );
}
