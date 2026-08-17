import * as React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import DOMPurify from "dompurify";
import { CodeBlock } from "./chat/CodeBlock";
import { isOpenableUrl, openExternalUrl } from "../lib/open";

function extractCode(children: React.ReactNode): string {
  if (typeof children === "string") return children.replace(/\n$/, "");
  if (Array.isArray(children)) {
    return children.map(extractCode).join("").replace(/\n$/, "");
  }
  if (React.isValidElement<{ children?: React.ReactNode }>(children)) {
    return extractCode(children.props.children);
  }
  return String(children ?? "").replace(/\n$/, "");
}

export interface MarkdownRendererProps {
  content: string;
}

// Ensure external links require confirmation (could be done via onClick or sanitization)
DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if ("target" in node) {
    node.setAttribute("target", "_blank");
    node.setAttribute("rel", "noopener noreferrer");
  }
});

const MemoizedMarkdown = React.memo(
  ({ content }: { content: string }) => {
    // Sanitize before rendering (rehypeRaw can be unsafe if not sanitized)
    const sanitized = DOMPurify.sanitize(content, { USE_PROFILES: { html: true } });

    return (
      <div className="markdown-body">
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          rehypePlugins={[rehypeRaw]}
          components={{
            pre({ children }) {
              return <>{children}</>;
            },
            code(props) {
              const { children, className, ...rest } = props;
              const match = /language-([\w+-]+)/.exec(className || "");
              const codeText = extractCode(children);

              if (match || (className && codeText.includes("\n"))) {
                return <CodeBlock code={codeText} language={match?.[1]} />;
              }

              return (
                <code className="code-inline" {...rest}>
                  {children}
                </code>
              );
            },
            a(props) {
              const { href, children, ...rest } = props;
              // The webview has nowhere to navigate to and `window.open` is
              // inert inside it, so a link has to be handed to the OS. Only
              // schemes a browser or mail client can take: `javascript:`,
              // `file:` and `data:` are dropped rather than opened.
              const handleClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
                e.preventDefault();
                if (!href || !isOpenableUrl(href)) return;
                void openExternalUrl(href).catch((err) => {
                  console.error("Failed to open link:", err);
                });
              };
              return (
                <a href={href} title={href} onClick={handleClick} {...rest}>
                  {children}
                </a>
              );
            }
          }}
        >
          {sanitized}
        </ReactMarkdown>
      </div>
    );
  },
  (prev, next) => prev.content === next.content
);

export function MarkdownRenderer({ content }: MarkdownRendererProps) {
  // For extremely large streaming messages, splitting by block might be better,
  // but for now we rely on React.memo and react-markdown's internal efficiency.
  // We can chunk by paragraphs/headers to only re-render the last active block.
  
  // Basic chunking: split by double newline.
  const blocks = React.useMemo(() => {
    // If the stream is active, the last chunk is updating rapidly.
    const chunks = content.split(/\n\n(?=.)/); // split by \n\n but keep empty lines together
    return chunks;
  }, [content]);

  return (
    <>
      {blocks.map((block, i) => (
        <MemoizedMarkdown key={i} content={block + (i < blocks.length - 1 ? "\n\n" : "")} />
      ))}
    </>
  );
}
