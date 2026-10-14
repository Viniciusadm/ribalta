import { useRef, type PointerEvent } from "react";

const minDistance = 50;

export function useSwipe(onSwipe: (direction: -1 | 1) => void) {
  const start = useRef<{ x: number; y: number } | null>(null);
  const swiped = useRef(false);

  return {
    handlers: {
      onPointerDown(event: PointerEvent) {
        if (event.pointerType === "mouse") return;
        start.current = { x: event.clientX, y: event.clientY };
        swiped.current = false;
      },
      onPointerUp(event: PointerEvent) {
        const origin = start.current;
        start.current = null;
        if (!origin) return;
        const dx = event.clientX - origin.x;
        const dy = event.clientY - origin.y;
        if (Math.abs(dx) < minDistance || Math.abs(dx) < Math.abs(dy) * 1.5) return;
        swiped.current = true;
        onSwipe(dx < 0 ? 1 : -1);
      },
      onPointerCancel() {
        start.current = null;
      },
    },
    consumeSwipe() {
      const value = swiped.current;
      swiped.current = false;
      return value;
    },
  };
}
