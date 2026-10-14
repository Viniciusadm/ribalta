import { useEffect } from "react";
import type { Document } from "@ribaltalang/compiler";
import type { Session, State } from "./navigation";

const prefix = "ribalta.persist:";

function storageKey(doc: Document) {
  return `${prefix}${window.location.pathname}|${doc.meta.title ?? ""}`;
}

function kept(doc: Document, state: State): State {
  const names = new Set(doc.persist ?? []);
  return Object.fromEntries(Object.entries(state).filter(([name]) => names.has(name)));
}

export function loadPersisted(doc: Document): State {
  if (!doc.persist?.length) return {};
  try {
    const parsed: unknown = JSON.parse(window.localStorage.getItem(storageKey(doc)) ?? "null");
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? kept(doc, parsed as State) : {};
  } catch {
    return {};
  }
}

export function usePersist(doc: Document, session: Session | null) {
  const state = session?.state;
  useEffect(() => {
    if (!doc.persist?.length || !state) return;
    try {
      window.localStorage.setItem(storageKey(doc), JSON.stringify(kept(doc, state)));
    } catch {
      return;
    }
  }, [doc, state]);
}
