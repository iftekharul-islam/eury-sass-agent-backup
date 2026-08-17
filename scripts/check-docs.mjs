import { readdir, readFile, stat } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const docsRoot = path.join(root, "docs");

try {
  execFileSync("python3", [path.join(docsRoot, "09-roadmap", "generate_phases.py"), "--check"], {
    cwd: root,
    encoding: "utf8",
  });
} catch (err) {
  console.error("Roadmap generation check failed:", err.stdout || err.message);
  process.exit(1);
}

async function markdownFiles(directory) {
  const entries = await readdir(directory);
  const files = await Promise.all(
    entries.map(async (entry) => {
      const target = path.join(directory, entry);
      return (await stat(target)).isDirectory() ? markdownFiles(target) : [target];
    }),
  );
  return files.flat().filter((file) => file.endsWith(".md"));
}

const files = await markdownFiles(docsRoot);
const failures = [];

for (const file of files) {
  const contents = await readFile(file, "utf8");
  if (!/^Spec-Version:\s+\d+\.\d+\.\d+$/m.test(contents)) {
    failures.push(`${path.relative(root, file)} is missing a valid Spec-Version`);
  }

  for (const match of contents.matchAll(/\[[^\]]+\]\(([^)]+)\)/g)) {
    const href = match[1];
    if (href.startsWith("http") || href.startsWith("#") || href.startsWith("mailto:")) {
      continue;
    }
    const [relativePath] = href.split("#");
    if (!relativePath.endsWith(".md")) {
      continue;
    }
    const target = path.resolve(path.dirname(file), relativePath);
    try {
      await stat(target);
    } catch {
      failures.push(
        `${path.relative(root, file)} links to missing ${relativePath}`,
      );
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`Documentation checks passed for ${files.length} markdown files`);
