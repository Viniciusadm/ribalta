import { useEffect, type Dispatch, type SetStateAction } from "react";
import type { Document } from "@ribaltalang/compiler";
import { tick, type Session } from "./navigation";

export function useClock<T extends Session | null>(doc: Document, session: T, setSession: Dispatch<SetStateAction<T>>, enabled = true) {
  const running = enabled && session !== null && Boolean(doc.pages[session.index]?.clock);
  const index = session?.index;
  useEffect(() => {
    if (!running) return;
    const timer = window.setInterval(() => setSession((current) => (current ? (tick(doc, current) as T) : current)), 1000);
    return () => window.clearInterval(timer);
  }, [doc, running, index, setSession]);
}
