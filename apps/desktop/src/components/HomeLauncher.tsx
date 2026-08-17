import * as React from "react";
import { Icon } from "./Icons";
import { EuryMark } from "./EuryMark";
import { useAppSettings } from "../lib/settings";

export interface HomeLauncherProps {
  onSubmitPrompt?: (prompt: string) => void;
}

export function HomeLauncher({ onSubmitPrompt }: HomeLauncherProps) {
  const [settings] = useAppSettings();
  const [prompt, setPrompt] = React.useState("");

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (prompt.trim()) {
        onSubmitPrompt?.(prompt);
        setPrompt("");
      }
    }
  };

  const submit = () => {
    if (!prompt.trim()) return;
    onSubmitPrompt?.(prompt);
    setPrompt("");
  };

  return (
    <div className="launcher">
      <EuryMark size={52} className="launcher-mark" />

      <h1>
        {settings.profile.preferredName
          ? `Welcome back, ${settings.profile.preferredName}`
          : "What can I help you with?"}
      </h1>
      <p className="lead">Ask Eury anything to start a conversation.</p>

      <div className="composer focused" style={{ width: "min(680px, 100%)", marginBottom: "28px" }}>
        <textarea
          className="input"
          placeholder="Ask Eury anything…"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={handleKeyDown}
          rows={1}
          style={{ minHeight: "44px" }}
        />
        <div className="composer-bar">
          <button type="button" className="cb-icon" title="Attach context">
            <Icon name="clip" size={16} />
          </button>
          <span className="spacer" />
          <span className="select">
            {settings.model.activeModelLabel}
            <Icon name="chev-d" />
          </span>
          <button type="button" className="send" onClick={submit} aria-label="Send">
            <Icon name="send" size={16} />
          </button>
        </div>
      </div>
    </div>
  );
}
