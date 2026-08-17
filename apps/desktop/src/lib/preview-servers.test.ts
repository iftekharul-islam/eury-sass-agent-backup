import { beforeEach, describe, expect, it } from "vitest";
import {
  detectPreviewUrls,
  previewServers,
  selectActivePreview,
} from "./preview-servers";

/** A real escape byte, written this way so none appears in this file. */
const ESC = String.fromCharCode(27);

beforeEach(() => {
  previewServers.reset();
});

describe("detectPreviewUrls", () => {
  it("reads the URL out of a Vite banner, colour codes and all", () => {
    const banner = `  ${ESC}[32m➜${ESC}[39m  ${ESC}[1mLocal${ESC}[22m:   ${ESC}[36mhttp://localhost:5173/${ESC}[39m\n`;

    expect(detectPreviewUrls(banner)).toEqual([
      { id: "http://localhost:5173", url: "http://localhost:5173/", host: "localhost", port: "5173" },
    ]);
  });

  it("points a bind address at something the browser can open", () => {
    // 0.0.0.0 is where the server listens, not an address to browse to.
    const [hit] = detectPreviewUrls("Listening on http://0.0.0.0:3000");
    expect(hit.url).toBe("http://localhost:3000/");
  });

  it("keeps the path but not the sentence it sits in", () => {
    const [hit] = detectPreviewUrls("Open http://127.0.0.1:8080/admin/login.");
    expect(hit.url).toBe("http://127.0.0.1:8080/admin/login");
  });

  it("ignores remote URLs — a preview is a local server", () => {
    expect(detectPreviewUrls("see https://example.com:443/docs")).toEqual([]);
  });

  it("finds each distinct server once, however often it is printed", () => {
    const hits = detectPreviewUrls(
      "Local: http://localhost:5173/\nNetwork: http://localhost:5173/\nAPI: http://localhost:3001/",
    );
    expect(hits.map((hit) => hit.id)).toEqual([
      "http://localhost:5173",
      "http://localhost:3001",
    ]);
  });
});

describe("previewServers", () => {
  it("records a server from terminal output and shows it as live", () => {
    previewServers.observeOutput("term-1", "  ➜  Local:   http://localhost:5173/\n");

    const active = selectActivePreview(previewServers.getState());
    expect(active?.url).toBe("http://localhost:5173/");
    expect(active?.status).toBe("live");
    expect(active?.terminalId).toBe("term-1");
  });

  it("marks a server stopped when the session hosting it ends", () => {
    previewServers.observeOutput("term-1", "http://localhost:5173/");
    previewServers.markTerminalStopped("term-1");

    expect(selectActivePreview(previewServers.getState())?.status).toBe("stopped");
  });

  it("brings a restarted server back on the same card", () => {
    previewServers.observeOutput("term-1", "http://localhost:5173/");
    previewServers.markTerminalStopped("term-1");
    previewServers.observeOutput("term-2", "http://localhost:5173/");

    const state = previewServers.getState();
    expect(state.servers).toHaveLength(1);
    expect(state.servers[0].status).toBe("live");
    expect(state.servers[0].terminalId).toBe("term-2");
  });

  it("prefers a live server over a stopped one that was seen later", () => {
    previewServers.observeOutput("term-1", "http://localhost:5173/");
    previewServers.observeOutput("term-2", "http://localhost:3001/");
    previewServers.markTerminalStopped("term-2");

    expect(selectActivePreview(previewServers.getState())?.port).toBe("5173");
  });

  it("drops a dismissed server", () => {
    previewServers.observeOutput("term-1", "http://localhost:5173/");
    previewServers.dismiss("http://localhost:5173");

    expect(selectActivePreview(previewServers.getState())).toBeUndefined();
  });
});
