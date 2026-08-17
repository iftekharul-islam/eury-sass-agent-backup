import * as React from "react";
import { createPortal } from "react-dom";
import { Icon } from "./Icons";
import type { Theme } from "../lib/theme";

export interface UserMenuProps {
  name: string;
  plan: string;
  avatar: string;
  theme: Theme;
  onOpenSettings?: () => void;
  onSetTheme: (theme: Theme) => void;
  onLogout?: () => void;
}

export function UserMenu({ name, plan, avatar, theme, onOpenSettings, onSetTheme, onLogout }: UserMenuProps) {
  const [open, setOpen] = React.useState(false);
  const anchorRef = React.useRef<HTMLDivElement>(null);
  const menuRef = React.useRef<HTMLDivElement>(null);
  const [layout, setLayout] = React.useState<{ top: number; left: number } | null>(null);

  const updateLayout = React.useCallback(() => {
    const anchor = anchorRef.current?.getBoundingClientRect();
    const menuHeight = menuRef.current?.offsetHeight ?? 280;
    if (!anchor) return;
    const panelWidth = 232;
    const gap = 8;
    const margin = 12;
    const left = Math.max(margin, Math.min(anchor.left, window.innerWidth - panelWidth - margin));
    let top = anchor.top - menuHeight - gap;
    if (top < margin) {
      top = Math.min(anchor.bottom + gap, window.innerHeight - menuHeight - margin);
    }
    setLayout({ top, left });
  }, []);

  React.useLayoutEffect(() => {
    if (!open) {
      setLayout(null);
      return;
    }
    updateLayout();
    const menuEl = menuRef.current;
    if (!menuEl) return;
    const observer = new ResizeObserver(() => updateLayout());
    observer.observe(menuEl);
    return () => observer.disconnect();
  }, [open, updateLayout]);

  React.useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (anchorRef.current?.contains(target) || menuRef.current?.contains(target)) return;
      setOpen(false);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", updateLayout);
    window.addEventListener("scroll", updateLayout, true);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", updateLayout);
      window.removeEventListener("scroll", updateLayout, true);
    };
  }, [open, updateLayout]);

  const initial = avatar || name[0] || "U";
  const isDark = theme === "dark";

  const menu =
    open && typeof document !== "undefined" ? (
      <div
        ref={menuRef}
        className="user-menu-menu"
        style={{
          position: "fixed",
          top: layout?.top ?? -9999,
          left: layout?.left ?? -9999,
          zIndex: 300,
          visibility: layout ? "visible" : "hidden",
        }}
      >
        <div className="user-menu-panel">
          <div className="user-menu-header">
            <span className="avatar user">{initial}</span>
            <div className="user-menu-header-text">
              <span className="name">{name}</span>
              <span className="plan">{plan}</span>
            </div>
          </div>

          <div className="user-menu-divider" />

          <button
            type="button"
            className="user-menu-row"
            onClick={() => {
              setOpen(false);
              onOpenSettings?.();
            }}
          >
            <Icon name="sliders" size={15} />
            <span>Personalization</span>
          </button>
          <button
            type="button"
            className="user-menu-row"
            onClick={() => {
              setOpen(false);
              onOpenSettings?.();
            }}
          >
            <Icon name="user" size={15} />
            <span>Profile</span>
          </button>
          <button
            type="button"
            className="user-menu-row"
            onClick={() => {
              setOpen(false);
              onOpenSettings?.();
            }}
          >
            <Icon name="settings" size={15} />
            <span>Settings</span>
          </button>

          <div className="user-menu-divider" />

          <button
            type="button"
            className="user-menu-row"
            onClick={() => onSetTheme(isDark ? "light" : "dark")}
          >
            <Icon name={isDark ? "moon" : "sun"} size={15} />
            <span>{isDark ? "Dark mode" : "Light mode"}</span>
          </button>

          <div className="user-menu-divider" />

          <button
            type="button"
            className="user-menu-row danger"
            onClick={() => {
              setOpen(false);
              onLogout?.();
            }}
          >
            <Icon name="logout" size={15} />
            <span>Log out</span>
          </button>
        </div>
      </div>
    ) : null;

  return (
    <>
      <div
        ref={anchorRef}
        className="user-menu-trigger"
        onClick={() => setOpen((p) => !p)}
        role="button"
        tabIndex={0}
        aria-label="Account menu"
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setOpen((p) => !p);
          }
        }}
      >
        <span className="avatar user">{initial}</span>
        <span className="name">{name}</span>
        <Icon name="chev-d" size={14} />
      </div>
      {menu ? createPortal(menu, document.body) : null}
    </>
  );
}
