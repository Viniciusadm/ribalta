import type { Document } from "@ribaltalang/compiler";
import { sessionHash, type Session } from "./navigation";

export interface ScormApi {
  LMSInitialize(argument: ""): string;
  LMSFinish(argument: ""): string;
  LMSGetValue(element: string): string;
  LMSSetValue(element: string, value: string): string;
  LMSCommit(argument: ""): string;
}

const searchDepth = 10;
const suspendLimit = 4096;

function apiIn(target: Window | null): ScormApi | null {
  let current = target;
  for (let depth = 0; current && depth < searchDepth; depth++) {
    try {
      const api = (current as Window & { API?: ScormApi }).API;
      if (api && typeof api.LMSInitialize === "function") return api;
    } catch {
      return null;
    }
    if (current.parent === current) break;
    current = current.parent;
  }
  return null;
}

export function findScormApi(start: Window = window): ScormApi | null {
  return apiIn(start) ?? apiIn(start.opener as Window | null);
}

export class ScormSession {
  private sent = new Map<string, string>();
  private completed = false;
  private finished = false;

  private constructor(
    private readonly api: ScormApi,
    readonly resumeHash: string,
  ) {}

  static connect(start: Window = window): ScormSession | null {
    const api = findScormApi(start);
    if (!api || api.LMSInitialize("") !== "true") return null;
    const status = api.LMSGetValue("cmi.core.lesson_status");
    const session = new ScormSession(api, api.LMSGetValue("cmi.suspend_data").startsWith("#") ? api.LMSGetValue("cmi.suspend_data") : "");
    session.completed = ["completed", "passed", "failed"].includes(status);
    if (!status || status === "not attempted") session.set("cmi.core.lesson_status", "incomplete");
    api.LMSCommit("");
    return session;
  }

  report(doc: Document, session: Session) {
    const scorm = doc.scorm;
    const page = doc.pages[session.index];
    if (!page) return;
    const score = scorm?.score ? Number(session.state[scorm.score] ?? doc.state?.[scorm.score] ?? 0) : null;
    const max = scorm?.max ?? 100;
    let changed = false;
    if (score !== null && Number.isFinite(score)) {
      changed = this.set("cmi.core.score.min", "0") || changed;
      changed = this.set("cmi.core.score.max", String(max)) || changed;
      changed = this.set("cmi.core.score.raw", String(Math.min(Math.max(score, 0), max))) || changed;
    }
    const reached = scorm?.complete?.length ? scorm.complete.includes(page.id) : session.index === doc.pages.length - 1;
    if (reached || this.completed) {
      this.completed = true;
      const passed = scorm?.pass === undefined || score === null ? null : score >= scorm.pass;
      changed = this.set("cmi.core.lesson_status", passed === null ? "completed" : passed ? "passed" : "failed") || changed;
    }
    changed = this.set("cmi.core.lesson_location", page.id.slice(0, 255)) || changed;
    const hash = sessionHash(doc, session);
    if (hash.length <= suspendLimit) changed = this.set("cmi.suspend_data", hash) || changed;
    if (changed) this.api.LMSCommit("");
  }

  finish() {
    if (this.finished) return;
    this.finished = true;
    this.set("cmi.core.exit", this.completed ? "" : "suspend");
    this.api.LMSCommit("");
    this.api.LMSFinish("");
  }

  private set(element: string, value: string) {
    if (this.sent.get(element) === value) return false;
    this.sent.set(element, value);
    this.api.LMSSetValue(element, value);
    return true;
  }
}
