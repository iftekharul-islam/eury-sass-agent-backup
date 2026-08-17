import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  // Load .env from the agent/ workspace root so a single .env file
  // configures both the Rust backend (via dotenvy) and this frontend.
  envDir: path.resolve(dirname, "../.."),
  server: {
    port: 1420,
    strictPort: true,
  },
});
