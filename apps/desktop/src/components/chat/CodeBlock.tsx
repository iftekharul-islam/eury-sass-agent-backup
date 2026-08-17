import * as React from "react";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import css from "highlight.js/lib/languages/css";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import python from "highlight.js/lib/languages/python";
import sql from "highlight.js/lib/languages/sql";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import { Icon } from "../Icons";
import { isShellLanguage, toRunnableCommand, useRunCommand } from "../../lib/run-command";

hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("js", javascript);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("tsx", typescript);
hljs.registerLanguage("jsx", javascript);
hljs.registerLanguage("python", python);
hljs.registerLanguage("py", python);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("sh", bash);
hljs.registerLanguage("shell", bash);
hljs.registerLanguage("json", json);
hljs.registerLanguage("sql", sql);
hljs.registerLanguage("css", css);
hljs.registerLanguage("html", xml);
hljs.registerLanguage("xml", xml);

function highlight(code: string, language: string) {
  if (language && hljs.getLanguage(language)) {
    return hljs.highlight(code, { language }).value;
  }
  return hljs.highlightAuto(code).value;
}

export interface CodeBlockProps {
  code: string;
  language?: string;
}

export function CodeBlock({ code, language }: CodeBlockProps) {
  const [copied, setCopied] = React.useState(false);
  const lang = (language ?? "text").toLowerCase();
  const html = React.useMemo(() => highlight(code, lang), [code, lang]);

  // A shell block is an instruction the user would otherwise have to retype in
  // another app — the dev server the agent just told them to start, most of
  // all. With a workspace terminal available, it runs from here.
  const runCommand = useRunCommand();
  const runnable = React.useMemo(
    () => (isShellLanguage(lang) ? toRunnableCommand(code) : ""),
    [code, lang],
  );
  const canRun = Boolean(runCommand && runnable);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      /* ignore */
    }
  };

  return (
    <div className="code-block-shell group/code">
      <div className="code-block-toolbar">
        <span className="code-block-lang">{lang}</span>
        {canRun && (
          <button
            type="button"
            className="code-block-run"
            onClick={() => runCommand?.(runnable)}
            title="Run in the workspace terminal"
          >
            <Icon name="play" size={12} />
            Run
          </button>
        )}
        <button
          type="button"
          className="code-block-copy"
          onClick={copy}
          aria-label="Copy code"
        >
          <Icon name={copied ? "check" : "copy"} size={12} />
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="code-block-pre">
        <code className={`hljs language-${lang}`} dangerouslySetInnerHTML={{ __html: html }} />
      </pre>
    </div>
  );
}
