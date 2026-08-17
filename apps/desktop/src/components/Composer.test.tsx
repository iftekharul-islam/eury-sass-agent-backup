/**
 * @vitest-environment jsdom
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { Composer } from "./Composer";
import { COMPOSER_MODES } from "./ModePicker";

let container: HTMLDivElement;
let root: Root;
const noop = () => {};

function render(props: Partial<React.ComponentProps<typeof Composer>> = {}) {
  act(() => {
    root.render(
      <Composer
        mode="Agent"
        model={{ provider: "OpenAI", modelId: "gpt-5.6", label: "GPT 5.6" }}
        onModeChange={props.onModeChange ?? noop}
        onModelChange={noop}
        onSubmit={props.onSubmit ?? noop}
        useManagedModels={false}
        {...props}
      />,
    );
  });
}

/** React tracks its own value, so a bare `el.value =` never reaches onChange. */
function type(textarea: HTMLTextAreaElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(
    HTMLTextAreaElement.prototype,
    "value",
  )?.set;
  act(() => {
    setter?.call(textarea, value);
    textarea.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

function click(el: Element | null) {
  act(() => {
    el?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

beforeEach(() => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  // jsdom ships no ResizeObserver; the picker uses one to keep its position
  // pinned to the composer.
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

describe("Composer mode picker", () => {
  it("opens with every run mode and its description", () => {
    render();
    click(container.querySelector(".select"));

    const items = document.querySelectorAll(".mode-picker-item");
    expect(items).toHaveLength(COMPOSER_MODES.length);
    const text = document.body.textContent ?? "";
    for (const option of COMPOSER_MODES) {
      expect(text).toContain(option.label);
      expect(text).toContain(option.description);
    }
  });

  it("escapes the composer's clipping by rendering in a portal", () => {
    render();
    click(container.querySelector(".select"));

    const menu = document.querySelector(".mode-picker-menu");
    expect(menu).not.toBeNull();
    // Outside the composer subtree, so `overflow: hidden` cannot cut it off.
    expect(container.contains(menu)).toBe(false);
    expect(menu?.parentElement).toBe(document.body);
    expect((menu as HTMLElement).style.position).toBe("fixed");
  });

  it("reports the picked mode and closes", () => {
    const onModeChange = vi.fn();
    render({ onModeChange });
    click(container.querySelector(".select"));

    const plan = [...document.querySelectorAll(".mode-picker-item")].find((el) =>
      el.textContent?.startsWith("Plan"),
    );
    click(plan ?? null);

    expect(onModeChange).toHaveBeenCalledWith("Plan");
    expect(document.querySelector(".mode-picker-menu")).toBeNull();
  });
});

describe("Composer send button", () => {
  it("becomes a stop button while a run streams", () => {
    const onStop = vi.fn();
    render({ isRunning: true, onStop });

    const stop = container.querySelector(".send.is-stop");
    expect(stop).not.toBeNull();
    expect(stop?.getAttribute("aria-label")).toBe("Stop run");
    expect(container.querySelector('[aria-label="Send prompt"]')).toBeNull();

    click(stop);
    expect(onStop).toHaveBeenCalledTimes(1);
  });

  it("does not submit on Enter while a run streams", () => {
    const onSubmit = vi.fn();
    const onStop = vi.fn();
    render({ isRunning: true, onStop, onSubmit });

    const textarea = container.querySelector("textarea") as HTMLTextAreaElement;
    type(textarea, "queued message");
    act(() => {
      textarea.dispatchEvent(
        new KeyboardEvent("keydown", { key: "Enter", bubbles: true }),
      );
    });

    expect(onSubmit).not.toHaveBeenCalled();
    // The text stays put rather than being silently swallowed.
    expect(textarea.value).toBe("queued message");
  });

  it("sends normally when no run is active", () => {
    const onSubmit = vi.fn();
    render({ onSubmit });

    const textarea = container.querySelector("textarea") as HTMLTextAreaElement;
    type(textarea, "hello");
    click(container.querySelector('[aria-label="Send prompt"]'));

    expect(onSubmit).toHaveBeenCalledWith("hello", undefined);
  });
});
