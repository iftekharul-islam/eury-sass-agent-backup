import * as React from "react";
import { Icon } from "./Icons";

export interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectAction?: (actionId: string) => void;
}

interface CommandItem {
  id: string;
  title: string;
  desc?: string;
  icon: string;
  shortcut?: string;
  section: string;
}

const COMMAND_ITEMS: CommandItem[] = [
  {
    id: "compact",
    title: "/compact",
    desc: "Summarize older turns to free context",
    icon: "slash",
    section: "Commands",
  },
  {
    id: "clear",
    title: "/clear",
    desc: "Start a fresh conversation",
    icon: "slash",
    shortcut: "⌘⇧K",
    section: "Commands",
  },
  {
    id: "plan-cmd",
    title: "/plan",
    desc: "Create or view an architectural plan",
    icon: "list",
    shortcut: "⌘P",
    section: "Commands",
  },
  {
    id: "changes-cmd",
    title: "/changes",
    desc: "Inspect working tree diffs",
    icon: "diff",
    shortcut: "⌘⇧G",
    section: "Commands",
  },
  {
    id: "compact-now",
    title: "Compact now",
    desc: "Runs compaction immediately",
    icon: "spark",
    section: "Actions",
  },
  {
    id: "new-chat",
    title: "New Agent Chat",
    desc: "Start a new conversation in this workspace",
    icon: "plus",
    shortcut: "⌘N",
    section: "Actions",
  },
  {
    id: "view-changes",
    title: "View Changes",
    desc: "Inspect active working tree diffs",
    icon: "diff",
    shortcut: "⌘⇧G",
    section: "Actions",
  },
  {
    id: "view-plan",
    title: "View Current Plan",
    desc: "Open the active 5-step implementation plan",
    icon: "list",
    section: "Actions",
  },
  {
    id: "view-runs",
    title: "View Run History",
    desc: "Inspect past execution runs and restore points",
    icon: "clock",
    section: "Actions",
  },
  {
    id: "view-terminal",
    title: "Open Terminal",
    desc: "A real PTY-backed shell for this workspace",
    icon: "terminal",
    shortcut: "⌘`",
    section: "Actions",
  },
  {
    id: "trust-project",
    title: "Trust Project Settings",
    desc: "Review sandbox policy and workspace trust",
    icon: "shield",
    section: "Actions",
  },
  {
    id: "settings",
    title: "Settings",
    desc: "Open application preferences",
    icon: "settings",
    shortcut: "⌘,",
    section: "Settings",
  },
  {
    id: "context-settings",
    title: "Context and compaction",
    desc: "Configure the threshold",
    icon: "settings",
    section: "Settings",
  },
];

export function CommandPalette({ isOpen, onClose, onSelectAction }: CommandPaletteProps) {
  const [query, setQuery] = React.useState("");
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const inputRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 20);
      setQuery("");
      setSelectedIndex(0);
    }
  }, [isOpen]);

  const prefix = query.charAt(0);
  const isPrefixed = [">", "@", "#", ":", "?", "!", "~", "/"].includes(prefix);
  const search = isPrefixed ? query.slice(1).trim() : query.trim();

  const filtered = COMMAND_ITEMS.filter((item) =>
    item.title.toLowerCase().includes(search.toLowerCase()) ||
    (item.desc && item.desc.toLowerCase().includes(search.toLowerCase())) ||
    item.section.toLowerCase().includes(search.toLowerCase())
  );

  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return;

      if (e.key === "Escape") {
        onClose();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => (filtered.length > 0 ? (prev + 1) % filtered.length : 0));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) =>
          filtered.length > 0 ? (prev - 1 + filtered.length) % filtered.length : 0
        );
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (filtered[selectedIndex]) {
          if (onSelectAction) onSelectAction(filtered[selectedIndex].id);
          onClose();
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose, onSelectAction, filtered, selectedIndex]);

  if (!isOpen) return null;

  const sections = Array.from(new Set(filtered.map((i) => i.section)));

  return (
    <div className="scrim" onClick={onClose}>
      <div className="palette" onClick={(e) => e.stopPropagation()}>
        <div className="pin">
          <Icon name="search" size={16} style={{ color: "var(--color-fg-subtle)" }} />
          <input
            ref={inputRef}
            placeholder="Type a command or search..."
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
          />
        </div>

        <div className="plist">
          {sections.map((sec) => {
            const items = filtered.filter((i) => i.section === sec);
            return (
              <React.Fragment key={sec}>
                <div className="sec-label" style={{ margin: "10px 8px 6px" }}>
                  {sec}
                </div>
                {items.map((item) => {
                  const globalIdx = filtered.indexOf(item);
                  return (
                    <div
                      key={item.id}
                      className={`row ${selectedIndex === globalIdx ? "on" : ""}`}
                      onClick={() => {
                        if (onSelectAction) onSelectAction(item.id);
                        onClose();
                      }}
                      onMouseEnter={() => setSelectedIndex(globalIdx)}
                    >
                      <Icon name={item.icon} />
                      <span className="t">{item.title}</span>
                      {item.desc && <span className="m">{item.desc}</span>}
                      {item.shortcut && <span className="kbd">{item.shortcut}</span>}
                    </div>
                  );
                })}
              </React.Fragment>
            );
          })}
          {filtered.length === 0 && (
            <div style={{ padding: "20px", textAlign: "center", color: "var(--color-fg-muted)", fontSize: "13px" }}>
              No commands matching "{query}"
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
