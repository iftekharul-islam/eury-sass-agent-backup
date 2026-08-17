import * as React from "react";
import { useState, useEffect, useMemo } from "react";
import { SvgSprite } from "./components/Icons";
import { ThemeManager } from "./lib/theme";
import { TitleBar } from "./components/TitleBar";
import { CodeSidebar, HomeSidebar } from "./components/Sidebar";
import { HomeLauncher } from "./components/HomeLauncher";
import { ConversationCanvas } from "./components/ConversationCanvas";
import { ChangesView } from "./components/ChangesView";
import { RunsView } from "./components/RunsView";
import { ApprovalsView } from "./components/ApprovalsView";
import { TerminalPane, type TerminalPaneSeed } from "./components/TerminalPane";
import { SettingsModal } from "./components/SettingsModal";
import { CommandPalette } from "./components/CommandPalette";
import { TrustModal } from "./components/TrustModal";
import { ErrorBoundary } from "./components/ui/ErrorBoundary";
import { SandboxSecurityBanner } from "./components/ui/SandboxSecurityBanner";

type ScreenType = "launcher" | "run" | "approve-cmd" | "approve-diff" | "changes" | "plan" | "runs" | "approvals" | "terminal";

import { LoginModal } from "./components/LoginModal";
import { HomeChatCanvas } from "./components/HomeChatCanvas";
import { authIpc, handleUnauthorized, SESSION_EXPIRED_EVENT } from "./lib/auth";
import { bootstrapDefaultModel } from "./lib/bootstrap";
import { cloudApi } from "./lib/cloud";
import { useHomeChatHistory } from "./lib/useHomeChatHistory";
import { ipcClient } from "./lib/ipc";
import { addUserProject, getRecentUserProjects } from "./lib/projects";
import {
  createCodeConversation,
  ensureDefaultCodeConversation,
  listCodeConversations,
  registerCodeConversationMessage,
} from "./lib/code-conversations";
import { SettingsStore } from "./lib/settings";
import {
  selectChangedFiles,
  startSessionEventBridge,
  useSessionState,
} from "./lib/session-store";
import type { ProjectHistoryGroup } from "./components/Sidebar";
import type { Capabilities } from "./lib/ipc";

function AppContent() {
  const [area, setArea] = useState<"home" | "code">("code");
  const [screen, setScreen] = useState<ScreenType>("run");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isPaletteOpen, setIsPaletteOpen] = useState(false);
  const [isTrustModalOpen, setIsTrustModalOpen] = useState(false);
  const [workspaceName, setWorkspaceName] = useState<string | undefined>(undefined);
  const [workspaceRoot, setWorkspaceRoot] = useState<string | undefined>(undefined);
  const [mode, setMode] = useState<"Chat" | "Agent" | "Plan" | "Ask" | "Build">("Agent");
  // Until a project is opened there is still a conversation to type into; it
  // just isn't attached to a workspace yet.
  const [activeConvoId, setActiveConvoId] = useState<string>(() => crypto.randomUUID());
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [pendingHomePrompt, setPendingHomePrompt] = useState<string | null>(null);
  const [codeHistoryVersion, setCodeHistoryVersion] = useState(0);
  const [terminalTabs, setTerminalTabs] = useState<string[]>([]);
  const [activeTerminalId, setActiveTerminalId] = useState<string | null>(null);
  const [terminalSeed, setTerminalSeed] = useState<TerminalPaneSeed | null>(null);
  const [capabilities, setCapabilities] = useState<Capabilities | null>(null);
  /** Real trust state of the open workspace; drives the read-only affordances. */
  const [isWorkspaceTrusted, setIsWorkspaceTrusted] = useState(false);
  /** A file path the user pulled from the tree, to be inserted in the composer. */
  const [pendingComposerMention, setPendingComposerMention] = useState<string | null>(null);

  const refreshCodeHistory = () => setCodeHistoryVersion((v) => v + 1);

  const homeChat = useHomeChatHistory(isAuthenticated);

  const handleAuthenticated = () => {
    setIsAuthenticated(true);
    void bootstrapDefaultModel();
  };

  // Check auth status on load
  useEffect(() => {
    authIpc
      .getTokens()
      .then(() => setIsAuthenticated(true))
      .catch(() => setIsAuthenticated(false));
  }, []);

  // A session the platform rejected is cleared by `handleUnauthorized`; the
  // app has to follow it back to sign-in rather than sitting on dead calls.
  useEffect(() => {
    const onExpired = () => setIsAuthenticated(false);
    window.addEventListener(SESSION_EXPIRED_EVENT, onExpired);
    return () => window.removeEventListener(SESSION_EXPIRED_EVENT, onExpired);
  }, []);

  // The UI calls capabilities_get first on every launch and refuses to
  // render features the core does not advertise; a missing sandbox surfaces
  // a persistent security banner rather than being silently ignored.
  useEffect(() => {
    ipcClient.capabilities
      .get()
      .then(setCapabilities)
      .catch(() => setCapabilities(null));
  }, []);

  // Load settings out of the encrypted store (migrating any legacy plaintext
  // localStorage blob on first run) before the UI settles on defaults.
  useEffect(() => {
    void SettingsStore.hydrate();
  }, []);

  // Reopen the project the user was last in. Without this the app started
  // with no workspace attached — the sidebar still listed the project, but
  // runs went out with no workspace root, so the agent had no tools and
  // answered "I can't write files directly in this mode".
  const restoredWorkspace = React.useRef(false);
  useEffect(() => {
    if (!isAuthenticated || workspaceRoot || restoredWorkspace.current) return;
    const [recent] = getRecentUserProjects();
    if (!recent) return;
    restoredWorkspace.current = true;
    void openProjectFolder(recent.path);
  }, [isAuthenticated, workspaceRoot]);

  // Every conversation, changes, runs and approvals surface reads from the
  // session store, which is fed here by the core's event topic.
  useEffect(() => {
    let stop: (() => void) | undefined;
    void startSessionEventBridge().then((teardown) => {
      stop = teardown;
    });
    return () => stop?.();
  }, []);

  useEffect(() => {
    if (!isAuthenticated) return;
    let cancelled = false;
    cloudApi
      .getUsage()
      .then((usage) => {
        if (cancelled) return;
        // Keep the billing panel on the platform's numbers rather than a
        // locally cached guess.
        SettingsStore.updateBilling({
          monthlySpentUsd: usage.limits.monthlyBudgetUsdMicros.used / 1_000_000,
          budgetLimitUsd: usage.limits.monthlyBudgetUsdMicros.limit / 1_000_000,
          tokensUsedThisMonth: usage.limits.monthlyManagedTokens.used,
          tokensLimitThisMonth: usage.limits.monthlyManagedTokens.limit,
          cycleRenewalDate: usage.limits.monthlyBudgetUsdMicros.resetsAt,
        });
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [isAuthenticated]);

  // Sync the signed-in account's email into the profile. Display name stays
  // user-controlled via preferredName — we don't auto-fill from the account.
  useEffect(() => {
    if (!isAuthenticated) return;
    let cancelled = false;
    cloudApi
      .getMe()
      .then((me) => {
        if (cancelled || !me.user) return;
        SettingsStore.updateProfile({ email: me.user.email });
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [isAuthenticated]);

  // Bring app to foreground when browser completes authorization
  useEffect(() => {
    let unlisten: (() => void) | void;
    void import("./lib/deep-link").then(({ setupDeepLinks }) => {
      void setupDeepLinks().then((cleanup) => {
        unlisten = cleanup;
      });
    });
    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    // Sync theme with ThemeManager defaults or persisted settings
    ThemeManager.init("light", "ember", "default");

    const handleKeyDown = (e: KeyboardEvent) => {
      // Mod+K or Mod+P: Open Command Palette
      if ((e.metaKey || e.ctrlKey) && (e.key === "k" || e.key === "p")) {
        e.preventDefault();
        setIsPaletteOpen((prev) => !prev);
        return;
      }
      // Mod+,: Open Settings
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        setIsSettingsOpen(true);
        return;
      }
      // Mod+Shift+G: Open Changes
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === "g" || e.key === "G")) {
        e.preventDefault();
        setArea("code");
        setScreen("changes");
        return;
      }
      // Mod+`: Open Terminal — placed above the input-focus early return so
      // it still works while the composer is focused.
      if ((e.metaKey || e.ctrlKey) && e.key === "`") {
        e.preventDefault();
        setArea("code");
        setScreen("terminal");
        return;
      }

      // Check if user is typing in an input/textarea
      const target = e.target as HTMLElement;
      const isInput = target && ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
      if (isInput) return;

      // Quick 1-9, 0 screen switching for rapid preview of all 10 mockup screens
      if (!e.metaKey && !e.ctrlKey && !e.altKey) {
        if (e.key === "1") {
          setArea("home");
          setScreen("launcher");
          setMode("Chat");
        } else if (e.key === "2") {
          setArea("code");
          setScreen("run");
          setMode("Agent");
        } else if (e.key === "3") {
          setArea("code");
          setScreen("approve-cmd");
          setMode("Build");
        } else if (e.key === "4") {
          setArea("code");
          setScreen("approve-diff");
          setMode("Agent");
        } else if (e.key === "5") {
          setArea("code");
          setScreen("changes");
        } else if (e.key === "6") {
          setArea("code");
          setScreen("plan");
          setMode("Plan");
        } else if (e.key === "7") {
          setIsPaletteOpen(true);
        } else if (e.key === "8") {
          setIsTrustModalOpen(true);
        } else if (e.key === "9") {
          setArea("code");
          setScreen("runs");
        } else if (e.key === "0") {
          setIsSettingsOpen(true);
        } else if (e.key === "Escape") {
          // xterm's own hidden <textarea> already trips the isInput guard
          // above when the terminal has keyboard focus, so this branch only
          // runs when focus is elsewhere — Escape stays usable inside vim.
          if (screen === "changes" || screen === "runs" || screen === "plan" || screen === "terminal") {
            setScreen("run");
          }
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [screen]);

  const handleAreaChange = (newArea: "home" | "code") => {
    setArea(newArea);
    if (newArea === "home") {
      setScreen("launcher");
      setMode("Chat");
    } else {
      setScreen("run");
      setMode("Agent");
    }
  };

  const openProjectFolder = async (existingPath?: string, conversationId?: string) => {
    try {
      const picked = existingPath ?? (await ipcClient.workspace.pickFolder());
      if (!picked) return;
      const info = await ipcClient.workspace.open(picked);
      addUserProject(info.path);
      setWorkspaceRoot(info.path);
      setWorkspaceName(info.name);
      // Trust persists across restarts, so a previously trusted folder should
      // not be re-prompted — only ask when it is genuinely untrusted.
      setIsWorkspaceTrusted(info.is_trusted);
      if (conversationId) {
        setActiveConvoId(conversationId);
      } else {
        const conversation = ensureDefaultCodeConversation(info.path);
        setActiveConvoId(conversation.id);
      }
      refreshCodeHistory();
      if (!info.is_trusted) setIsTrustModalOpen(true);
      handleAreaChange("code");
    } catch (err) {
      console.error("Failed to open project:", err);
      alert(err instanceof Error ? err.message : "Failed to open project");
    }
  };

  const handleHomeSubmitPrompt = async (prompt: string) => {
    const trimmed = prompt.trim();
    if (!trimmed) return;
    try {
      const model = SettingsStore.get().model;
      await homeChat.createConversation(
        model.activeModelId,
        "#a34054",
      );
      setPendingHomePrompt(trimmed);
      setArea("home");
      setScreen("run");
      setMode("Chat");
    } catch (err) {
      console.error("Failed to start home chat:", err);
      alert(err instanceof Error ? err.message : "Failed to start chat");
    }
  };

  // Project history: real folders the user opened, with the conversations
  // actually started inside each of them.
  const codeSidebarProjects = useMemo(
    (): ProjectHistoryGroup[] =>
      getRecentUserProjects().map((project) => ({
        name: project.name,
        projectPath: project.path,
        conversations: listCodeConversations(project.path).map((convo) => ({
          id: convo.id,
          title: convo.title,
          updatedAt: convo.updatedAt,
          active: activeConvoId === convo.id,
        })),
      })),
    [activeConvoId, workspaceRoot, codeHistoryVersion],
  );

  const handleNewCodeChat = () => {
    setArea("code");
    if (!workspaceRoot) {
      void openProjectFolder();
      return;
    }
    const created = createCodeConversation(workspaceRoot);
    setActiveConvoId(created.id);
    setScreen("run");
    setMode("Agent");
    refreshCodeHistory();
  };

  /**
   * Runs a shell snippet from the transcript. The answer usually ends in the
   * command that starts what was just built, and retyping it in another app is
   * the one step the desktop can take off the user.
   */
  const runCommandInTerminal = (command: string) => {
    if (!workspaceRoot || !isWorkspaceTrusted) return;
    setArea("code");
    setScreen("terminal");
    setTerminalSeed({ cwd: workspaceRoot, command });
  };

  const keepTerminalAlive =
    Boolean(workspaceRoot) &&
    area === "code" &&
    (screen === "terminal" || terminalTabs.length > 0 || terminalSeed != null);

  const handleSelectCodeProject = (projectPath: string) => {
    void openProjectFolder(projectPath);
  };

  const handleSelectCodeConversation = (id: string) => {
    const match = codeSidebarProjects
      .flatMap((group) =>
        group.conversations.map((convo) => ({ ...convo, projectPath: group.projectPath })),
      )
      .find((convo) => convo.id === id);
    if (match?.projectPath && match.projectPath !== workspaceRoot) {
      void openProjectFolder(match.projectPath, id);
      return;
    }
    setActiveConvoId(id);
    setScreen("run");
    setMode("Agent");
  };

  const handleCodeConversationMessage = (message: string) => {
    if (!workspaceRoot || !activeConvoId) return;
    registerCodeConversationMessage(workspaceRoot, activeConvoId, message);
    refreshCodeHistory();
  };

  const session = useSessionState();
  const changedFileCount = useMemo(
    () => selectChangedFiles(session, workspaceRoot).length,
    [session, workspaceRoot],
  );

  // Mode is how the next run behaves — not a navigation control. Switching
  // between Home and Code is the title bar's area tabs; a mode change used to
  // teleport the user out of the conversation they were in.
  const handleModeChange = (newMode: "Chat" | "Agent" | "Plan" | "Ask" | "Build") => {
    setMode(newMode);
  };

  const handlePaletteAction = (actionId: string) => {
    if (actionId === "view-changes" || actionId === "changes-cmd") {
      setArea("code");
      setScreen("changes");
    } else if (actionId === "view-plan" || actionId === "plan-cmd") {
      setArea("code");
      setScreen("plan");
      setMode("Plan");
    } else if (actionId === "view-runs") {
      setArea("code");
      setScreen("runs");
    } else if (actionId === "view-terminal") {
      setArea("code");
      setScreen("terminal");
    } else if (actionId === "trust-project") {
      setIsTrustModalOpen(true);
    } else if (actionId === "settings" || actionId === "context-settings") {
      setIsSettingsOpen(true);
    } else if (actionId === "new-chat" || actionId === "clear") {
      setArea("code");
      setScreen("run");
    }
  };

  // Determine app layout classes
  const isAuth = !isAuthenticated;
  const isLauncher = area === "home" && !homeChat.activeId;
  const isHomeArea = area === "home";
  const isCodeLayout = area === "code" && isAuthenticated;

  const appClasses = [
    "app",
    isAuth ? "is-auth" : "",
    isHomeArea ? "home-area" : "",
    isLauncher && !isAuth ? "is-launcher" : "",
    isCodeLayout ? "code-layout" : "",
    isCodeLayout ? "no-context" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={appClasses}>
      <SvgSprite />

      <TitleBar
        area={area}
        onAreaChange={handleAreaChange}
        workspaceName={workspaceName}
        isDirty={changedFileCount > 0}
        mode={mode}
        onModeChange={handleModeChange}
        onOpenCommands={() => setIsPaletteOpen(true)}
        onOpenChanges={() => {
          setArea("code");
          setScreen("changes");
        }}
        onOpenBrowser={() => {
          setArea("code");
          setScreen("changes");
        }}
        onOpenFiles={() => {
          setIsTrustModalOpen(true);
        }}
        onOpenSearch={() => setIsPaletteOpen(true)}
        isAuthenticated={isAuthenticated}
      />

      <SandboxSecurityBanner sandboxAvailable={capabilities?.sandbox.available} />

      <div className="body">
        {!isAuthenticated ? (
          <main className="main" style={{ height: "100%", width: "100%" }}>
            <LoginModal onLoginSuccess={handleAuthenticated} />
          </main>
        ) : area === "home" ? (
          <main className="main" id="main">
            <div className="home-shell">
              <HomeSidebar
                conversations={homeChat.conversations}
                activeChatId={homeChat.activeId}
                isOffline={!!homeChat.error}
                onNewChat={() => {
                  homeChat.clearActive();
                  setPendingHomePrompt(null);
                  setArea("home");
                  setScreen("launcher");
                  setMode("Chat");
                }}
                onOpenSettings={() => setIsSettingsOpen(true)}
                onLogout={() => void handleUnauthorized()}
                onSelectChat={(id) => {
                  void homeChat.selectConversation(id);
                  setPendingHomePrompt(null);
                  setArea("home");
                  setScreen("run");
                  setMode("Chat");
                }}
              />
              {homeChat.activeId ? (
                <div className="home-chat-pane">
                  <HomeChatCanvas
                    conversationId={homeChat.activeId}
                    messages={homeChat.messages}
                    conversationTitle={
                      homeChat.conversations.find((c) => c.id === homeChat.activeId)?.title
                    }
                    initialPrompt={pendingHomePrompt ?? undefined}
                    onInitialPromptConsumed={() => setPendingHomePrompt(null)}
                    onBeginTurn={homeChat.beginTurn}
                    onAppendAssistantDelta={homeChat.updateLastAssistant}
                    onRemoveLastAssistant={homeChat.removeLastAssistantTurn}
                    onRunError={(msg) => console.error("Home chat error:", msg)}
                  />
                </div>
              ) : (
                <HomeLauncher
                  onSubmitPrompt={(prompt) => void handleHomeSubmitPrompt(prompt)}
                />
              )}
            </div>
          </main>
        ) : (
          <>
              <CodeSidebar
              projects={codeSidebarProjects}
              activeConvoId={activeConvoId}
              activeProjectPath={workspaceRoot}
              onNewChat={handleNewCodeChat}
              onSelectProject={handleSelectCodeProject}
              onSelectConvo={handleSelectCodeConversation}
              onPickProject={() => void openProjectFolder()}
              onOpenSettings={() => setIsSettingsOpen(true)}
              onLogout={() => void handleUnauthorized()}
            />

            <main className="main" id="main">
              {screen === "changes" ? (
                <ChangesView
                  workspaceRoot={workspaceRoot}
                  workspaceName={workspaceName}
                  onBack={() => setScreen("run")}
                  onOpenRun={() => setScreen("runs")}
                  onOpenInEditor={(path) => setPendingComposerMention(path)}
                />
              ) : screen === "runs" ? (
                <RunsView
                  workspaceRoot={workspaceRoot}
                  workspaceName={workspaceName}
                  onOpenConversation={(conversationId) => {
                    setActiveConvoId(conversationId);
                    setScreen("run");
                  }}
                  onViewChanges={() => setScreen("changes")}
                />
              ) : screen === "approvals" ? (
                <ApprovalsView
                  workspaceRoot={workspaceRoot}
                  onBack={() => setScreen("run")}
                />
              ) : screen === "terminal" ? null : (
                <ConversationCanvas
                  onScreenChange={(s) => setScreen(s)}
                  conversationId={activeConvoId}
                  workspaceRoot={workspaceRoot}
                  workspaceName={workspaceName}
                  mode={mode}
                  onModeChange={(next) =>
                    handleModeChange(next as "Chat" | "Agent" | "Plan" | "Ask" | "Build")
                  }
                  onOpenProject={() => void openProjectFolder()}
                  isWorkspaceTrusted={isWorkspaceTrusted}
                  onTrustWorkspace={() => {
                    if (!workspaceRoot) return;
                    void ipcClient.workspace.setTrust(workspaceRoot, true);
                    setIsWorkspaceTrusted(true);
                  }}
                  onSubmitMessage={handleCodeConversationMessage}
                  mention={pendingComposerMention}
                  onMentionConsumed={() => setPendingComposerMention(null)}
                  onRunCommand={
                    workspaceRoot && isWorkspaceTrusted ? runCommandInTerminal : undefined
                  }
                />
              )}

              {keepTerminalAlive && (
                <TerminalPane
                  workspaceRoot={workspaceRoot}
                  tabs={terminalTabs}
                  activeId={activeTerminalId}
                  onTabsChange={setTerminalTabs}
                  onActiveChange={setActiveTerminalId}
                  seed={terminalSeed}
                  onSeedConsumed={() => setTerminalSeed(null)}
                  offscreen={screen !== "terminal"}
                />
              )}
            </main>
          </>
        )}
      </div>

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
      />

      <CommandPalette
        isOpen={isPaletteOpen}
        onClose={() => setIsPaletteOpen(false)}
        onSelectAction={handlePaletteAction}
      />

      <TrustModal
        isOpen={isTrustModalOpen}
        onClose={() => setIsTrustModalOpen(false)}
        projectPath={workspaceRoot ?? ""}
        onTrust={() => {
          if (workspaceRoot) void ipcClient.workspace.setTrust(workspaceRoot, true);
          setIsWorkspaceTrusted(true);
          setIsTrustModalOpen(false);
        }}
        onOpenReadOnly={() => {
          // Explicitly revoke rather than just closing: "open read-only" is a
          // decision, and a previously trusted path must drop back to
          // untrusted when the user picks it.
          if (workspaceRoot) void ipcClient.workspace.setTrust(workspaceRoot, false);
          setIsWorkspaceTrusted(false);
          setIsTrustModalOpen(false);
        }}
      />
    </div>
  );
}

export default function App() {
  return (
    <ErrorBoundary>
      <AppContent />
    </ErrorBoundary>
  );
}
