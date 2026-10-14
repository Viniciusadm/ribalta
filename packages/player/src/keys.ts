import { useEffect, useRef } from "react";

export interface KeyHandlers {
  move(direction: -1 | 1): void;
  shortcut?(event: KeyboardEvent): boolean;
  first?(): void;
  last?(): void;
  toggleNotes?(): void;
  escape?(): void;
}

const forward = ["ArrowRight", "ArrowDown", " ", "PageDown", "Enter"];
const backward = ["ArrowLeft", "ArrowUp", "PageUp", "Backspace"];

export function submitsToPage(event: KeyboardEvent) {
  return event.key === "Enter" && event.target instanceof HTMLInputElement && event.target.closest(".ribalta-page") !== null && !event.target.hasAttribute("data-submit");
}

export function useDocumentKeys(handlers: KeyHandlers) {
  const latest = useRef(handlers);
  latest.current = handlers;
  useEffect(() => {
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    const onKey = (event: KeyboardEvent) => {
      const current = latest.current;
      if (event.target instanceof HTMLElement && event.target.closest("input, textarea, [contenteditable='true']")) {
        if (submitsToPage(event)) current.shortcut?.(event);
        return;
      }
      if (event.target instanceof HTMLElement && event.target.closest("[data-interactive]") && (event.key === "Enter" || event.key === " ")) return;
      if (current.shortcut?.(event)) return;
      if (forward.includes(event.key)) {
        event.preventDefault();
        current.move(1);
      } else if (backward.includes(event.key)) {
        event.preventDefault();
        current.move(-1);
      } else if (event.key === "Home") {
        current.first?.();
      } else if (event.key === "End") {
        current.last?.();
      } else if (event.key.toLowerCase() === "n") {
        current.toggleNotes?.();
      } else if (event.key === "Escape") {
        current.escape?.();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
