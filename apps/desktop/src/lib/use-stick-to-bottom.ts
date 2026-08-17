import * as React from "react";

/** How close to the bottom counts as "at the bottom" when re-attaching. */
const AT_BOTTOM_PX = 8;

export interface StickToBottom<T extends HTMLElement> {
  ref: React.RefObject<T | null>;
  /** True while the view follows new content. */
  pinned: boolean;
  onScroll: (event: React.UIEvent<T>) => void;
  onWheel: (event: React.WheelEvent<T>) => void;
  /** Returns to the bottom and resumes following. */
  jumpToBottom: () => void;
}

/**
 * Keeps a scroll container at the bottom while content streams in, and gets
 * out of the way the instant the user scrolls up.
 *
 * Two rules earn their keep here:
 *
 * - **Direction, not distance.** Detaching on a threshold ("more than 64px
 *   from the bottom") means a small scroll up leaves the view still pinned,
 *   so the next delta yanks it back down — the shake. Any upward movement
 *   hands control over, however small.
 * - **Instant, never smooth.** A smooth scroll started on one delta is still
 *   animating when the next arrives, and the two animations fight. Following
 *   is a direct write; only the explicit jump animates.
 *
 * Following only ever scrolls *down*, so no "was that me or the user?" flag is
 * needed: a programmatic scroll can never look like a scroll up.
 */
export function useStickToBottom<T extends HTMLElement>(): StickToBottom<T> {
  const ref = React.useRef<T | null>(null);
  const pinnedRef = React.useRef(true);
  const [pinned, setPinned] = React.useState(true);
  const lastTopRef = React.useRef(0);

  const setPin = React.useCallback((next: boolean) => {
    pinnedRef.current = next;
    setPinned(next);
  }, []);

  // After every commit and before paint, so growing content is never painted
  // at the old offset first.
  React.useLayoutEffect(() => {
    const el = ref.current;
    if (!el || !pinnedRef.current) return;
    el.scrollTop = el.scrollHeight;
    lastTopRef.current = el.scrollTop;
  });

  const onScroll = React.useCallback(
    (event: React.UIEvent<T>) => {
      const el = event.currentTarget;
      const top = el.scrollTop;
      const distance = el.scrollHeight - el.clientHeight - top;

      if (top < lastTopRef.current - 1) {
        setPin(false);
      } else if (distance <= AT_BOTTOM_PX) {
        // Back at the bottom under their own steam: follow along again.
        setPin(true);
      }
      lastTopRef.current = top;
    },
    [setPin],
  );

  // The wheel says what the user wants before the scroll lands, so an upward
  // flick releases the pin without the frame in between where it snaps back.
  const onWheel = React.useCallback(
    (event: React.WheelEvent<T>) => {
      if (event.deltaY < 0) setPin(false);
    },
    [setPin],
  );

  const jumpToBottom = React.useCallback(() => {
    setPin(true);
    const el = ref.current;
    if (!el) return;
    if (typeof el.scrollTo === "function") {
      el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
    } else {
      el.scrollTop = el.scrollHeight;
    }
    lastTopRef.current = el.scrollHeight;
  }, [setPin]);

  return { ref, pinned, onScroll, onWheel, jumpToBottom };
}
