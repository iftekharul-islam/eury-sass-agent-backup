import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const [designSystem, mockup] = await Promise.all([
  readFile(new URL("docs/05-ui/01-design-system.md", root), "utf8"),
  readFile(new URL("mockups/index.html", root), "utf8"),
]);

const tokens = [
  "--color-bg",
  "--color-bg-sunken",
  "--color-bg-elevated",
  "--color-bg-inset",
  "--color-bg-hover",
  "--color-bg-active",
  "--color-fg",
  "--color-fg-muted",
  "--color-fg-subtle",
  "--color-fg-faint",
  "--color-border",
  "--color-border-strong",
  "--color-accent",
  "--color-accent-hover",
  "--color-accent-fg",
  "--color-accent-subtle",
  "--color-success",
  "--color-warning",
  "--color-danger",
  "--color-info",
  "--color-diff-add-bg",
  "--color-diff-del-bg",
  "--color-diff-add-fg",
  "--color-diff-del-fg",
  "--color-diff-word-add",
  "--color-diff-word-del",
];

const missing = tokens.filter(
  (token) => !designSystem.includes(`\`${token}\``) || !mockup.includes(`${token}:`),
);

if (missing.length > 0) {
  console.error(`Token parity failed for: ${missing.join(", ")}`);
  process.exit(1);
}

console.log(`Token parity passed for ${tokens.length} semantic tokens`);
