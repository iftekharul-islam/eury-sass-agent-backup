import * as React from "react";
import { Icon } from "./Icons";
import { useAppSettings } from "../lib/settings";
import { UserMenu } from "./UserMenu";

export interface ProjectHistoryGroup {
  name: string;
  projectPath?: string;
  conversations: Array<{
    id: string;
    title: string;
    active?: boolean;
    updatedAt?: number;
  }>;
}

export interface CodeSidebarProps {
  projects?: ProjectHistoryGroup[];
  activeConvoId?: string;
  activeProjectPath?: string;
  onSelectConvo?: (id: string) => void;
  onSelectProject?: (projectPath: string) => void;
  onPickProject?: () => void;
  onNewChat?: () => void;
  onOpenSettings?: () => void;
  onLogout?: () => void;
}

const CONVERSATIONS_PAGE = 5;

export function CodeSidebar({
  projects = [],
  activeConvoId,
  activeProjectPath,
  onSelectConvo,
  onSelectProject,
  onPickProject,
  onNewChat,
  onOpenSettings,
  onLogout,
}: CodeSidebarProps) {
  const [settings, settingsStore] = useAppSettings();
  const [visibleCounts, setVisibleCounts] = React.useState<Record<string, number>>({});

  const showMore = (projectName: string) => {
    setVisibleCounts((prev) => ({
      ...prev,
      [projectName]: (prev[projectName] ?? CONVERSATIONS_PAGE) + CONVERSATIONS_PAGE,
    }));
  };

  return (
    <aside className="sidebar">
      <div className="code-nav">
        <button
          type="button"
          className="sb-item on"
          onClick={onNewChat}
          aria-label="New Agent Chat"
        >
          <Icon name="plus" />
          <span className="t">New</span>
        </button>
      </div>

      <div className="code-project">
        <span className="label">Project history</span>
        <div className="code-project-actions">
          <button
            type="button"
            className="code-project-action"
            title="Open project folder"
            aria-label="Open project folder"
            onClick={onPickProject}
          >
            <Icon name="folder" size={14} />
          </button>
        </div>
      </div>

      <div className="code-project-list">
        {projects.length === 0 ? (
          <div className="history-empty">Open a project to start coding conversations.</div>
        ) : null}
        {projects.map((project) => {
          const limit = visibleCounts[project.name] ?? CONVERSATIONS_PAGE;
          const visibleConvos = project.conversations.slice(0, limit);
          const remaining = Math.max(0, project.conversations.length - limit);

          return (
            <section key={project.projectPath ?? project.name} className="history-project">
              <button
                type="button"
                className={`history-title ${
                  activeProjectPath && project.projectPath === activeProjectPath ? "on" : ""
                }`}
                onClick={() => project.projectPath && onSelectProject?.(project.projectPath)}
              >
                <Icon name="folder" />
                <span className="t">{project.name}</span>
                <span>{project.conversations.length}</span>
              </button>
              <div className="history-list">
                {visibleConvos.length === 0 ? (
                  <div className="history-empty-inline">No conversations yet</div>
                ) : (
                  visibleConvos.map((convo) => {
                    const isActive = activeConvoId === convo.id;
                    return (
                      <button
                        key={convo.id}
                        type="button"
                        className={`sb-item ${isActive ? "on" : ""}`}
                        onClick={() => onSelectConvo?.(convo.id)}
                      >
                        <span
                          className="dot"
                          style={{
                            background: isActive
                              ? "var(--color-accent)"
                              : "var(--color-fg-faint)",
                          }}
                        />
                        <span className="t">{convo.title}</span>
                      </button>
                    );
                  })
                )}
              </div>
              {remaining > 0 && (
                <button
                  type="button"
                  className="history-more"
                  onClick={() => showMore(project.name)}
                >
                  <Icon name="chev-d" />
                  Show more
                  <span className="more-count">{remaining} more</span>
                </button>
              )}
            </section>
          );
        })}
      </div>

      <UserMenu
        name={settings.profile.preferredName || "You"}
        plan={settings.profile.plan || "Free"}
        avatar={settings.profile.avatar}
        theme={settings.theme}
        onOpenSettings={onOpenSettings}
        onSetTheme={(theme) => settingsStore.set({ theme })}
        onLogout={onLogout}
      />
    </aside>
  );
}

export interface HomeSidebarProps {
  conversations?: Array<{ id: string; title: string }>;
  activeChatId?: string | null;
  /** The chat list could not be loaded from the platform. */
  isOffline?: boolean;
  onNewChat?: () => void;
  onOpenSettings?: () => void;
  onSelectChat?: (id: string) => void;
  onLogout?: () => void;
}

export function HomeSidebar({
  conversations = [],
  activeChatId = null,
  isOffline = false,
  onNewChat,
  onOpenSettings,
  onSelectChat,
  onLogout,
}: HomeSidebarProps) {
  const [settings, settingsStore] = useAppSettings();

  return (
    <aside className="home-nav">
      <div className="primary-links">
        <button type="button" className="sb-item on" onClick={onNewChat}>
          <Icon name="plus" />
          <span className="t">New</span>
        </button>
      </div>

      <div className="sb-label">
        Chat{isOffline ? " (offline)" : ""}
      </div>

      <div className="home-chat-list">
        {conversations.length === 0 && (
          <div className="history-empty">
            {isOffline ? "Chat history unavailable." : "No chats yet."}
          </div>
        )}
        {conversations.map((chat) => (
          <button
            key={chat.id}
            type="button"
            className={`sb-item ${activeChatId === chat.id ? "on" : ""}`}
            onClick={() => onSelectChat && onSelectChat(chat.id)}
          >
            <span
              className="dot"
              style={{
                background:
                  activeChatId === chat.id
                    ? "var(--color-accent)"
                    : "var(--color-fg-faint)",
              }}
            />
            <span className="t">{chat.title}</span>
          </button>
        ))}
      </div>

      <UserMenu
        name={settings.profile.preferredName || "You"}
        plan={settings.profile.plan || "Free"}
        avatar={settings.profile.avatar}
        theme={settings.theme}
        onOpenSettings={onOpenSettings}
        onSetTheme={(theme) => settingsStore.set({ theme })}
        onLogout={onLogout}
      />
    </aside>
  );
}
