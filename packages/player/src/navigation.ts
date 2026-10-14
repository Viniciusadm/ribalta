import type { Action, Document, Scalar } from "@ribaltalang/compiler";
import { evaluate, formatScalar, nextRandom } from "@ribaltalang/renderer/expr";

export type Position = { index: number; step: number };

export function normalizePosition(position: Position, steps: number[]): Position {
  const last = Math.max(steps.length - 1, 0);
  const index = clamp(Math.trunc(Number.isFinite(position.index) ? position.index : 0), 0, last);
  const step = clamp(Math.trunc(Number.isFinite(position.step) ? position.step : 0), 0, steps[index] ?? 0);
  return { index, step };
}

export function advancePosition(position: Position, direction: -1 | 1, steps: number[]): Position {
  const current = normalizePosition(position, steps);
  if (direction === 1) {
    if (current.step < (steps[current.index] ?? 0)) return { ...current, step: current.step + 1 };
    if (current.index === steps.length - 1) return current;
    return { index: current.index + 1, step: 0 };
  }
  if (current.step > 0) return { ...current, step: current.step - 1 };
  if (current.index === 0) return current;
  return { index: current.index - 1, step: steps[current.index - 1] ?? 0 };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

export type State = Record<string, Scalar>;

export interface HistoryEntry {
  index: number;
  step: number;
  locals: State;
}

export interface Session {
  index: number;
  step: number;
  state: State;
  history: HistoryEntry[];
  rng: number;
  elapsed: number;
  cues: Cue[];
}

export interface Cue {
  id: number;
  type: "play" | "stop";
  name: string;
}

const keptCues = 32;

export function freshSeed() {
  const values = new Uint32Array(1);
  globalThis.crypto?.getRandomValues?.(values);
  return values[0] || Math.floor(Math.random() * 4294967296);
}

function locals(doc: Document, index: number, state: State): State {
  const own = doc.pages[index]?.state ?? {};
  return Object.fromEntries(Object.keys(own).map((name) => [name, state[name] ?? own[name]!]));
}

function withoutLocals(doc: Document, index: number, state: State): State {
  const own = doc.pages[index]?.state ?? {};
  return Object.fromEntries(Object.entries(state).filter(([name]) => !(name in own)));
}

function place(doc: Document, session: Session, index: number, step: number, restored?: State): Session {
  const last = Math.max(doc.pages.length - 1, 0);
  const target = clamp(index, 0, last);
  const steps = doc.pages[target]?.steps ?? 0;
  const globals = withoutLocals(doc, session.index, session.state);
  const state = { ...globals, ...(doc.pages[target]?.state ?? {}), ...(restored ?? {}) };
  return { ...session, index: target, step: clamp(step, 0, steps), state, elapsed: 0 };
}

function changeState(session: Session, action: Action): Session {
  let rng = session.rng;
  const draw = () => {
    const result = nextRandom(rng);
    rng = result.seed;
    return result.value;
  };
  const changed = applyChange(session, action, { state: session.state, step: session.step, draw });
  return rng === session.rng ? changed : { ...changed, rng };
}

function applyChange(session: Session, action: Action, scope: { state: State; step: number; draw: () => number }): Session {
  const list = (name: string) => {
    const value = session.state[name];
    return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
  };
  const write = (name: string, value: Scalar) => ({ ...session, state: { ...session.state, [name]: value } });
  switch (action.type) {
    case "set":
      return write(action.name, evaluate(action.value, scope));
    case "toggle":
      return write(action.name, session.state[action.name] !== true);
    case "add": {
      const item = formatScalar(evaluate(action.value, scope));
      const items = list(action.name);
      return items.includes(item) ? session : write(action.name, [...items, item]);
    }
    case "remove": {
      const item = formatScalar(evaluate(action.value, scope));
      return write(action.name, list(action.name).filter((current) => current !== item));
    }
    case "clear":
      return write(action.name, []);
    case "play":
    case "stop": {
      const id = (session.cues[session.cues.length - 1]?.id ?? 0) + 1;
      return { ...session, cues: [...session.cues, { id, type: action.type, name: action.name }].slice(-keptCues) };
    }
    default:
      return session;
  }
}

function arrive(doc: Document, session: Session): Session {
  return (doc.pages[session.index]?.onEnter ?? []).reduce(changeState, session);
}

export function startSession(doc: Document, index = 0, step = 0, state: State = {}, restoring = false, rng = freshSeed()): Session {
  const base: Session = { index: 0, step: 0, state: { ...doc.state }, history: [], rng, elapsed: 0, cues: [] };
  const known = Object.fromEntries(Object.entries(state).filter(([name, value]) => sameKind(doc.state?.[name], value)));
  const placed = place(doc, base, index, step);
  const session = { ...placed, state: { ...placed.state, ...known } };
  return restoring ? session : arrive(doc, session);
}

export function moveTo(doc: Document, session: Session, index: number): Session {
  return { ...arrive(doc, place(doc, session, index, 0)), history: [] };
}

export function indexOf(doc: Document, id: string) {
  return doc.pages.findIndex((page) => page.id === id);
}

function linear(doc: Document, from: number, direction: -1 | 1) {
  for (let index = from + direction; index >= 0 && index < doc.pages.length; index += direction) {
    if (!doc.pages[index]?.hidden) return index;
  }
  return -1;
}

export function goTo(doc: Document, session: Session, target: string | number, step = 0): Session {
  const index = typeof target === "number" ? target : indexOf(doc, target);
  if (index < 0 || index >= doc.pages.length) return session;
  const entry: HistoryEntry = { index: session.index, step: session.step, locals: locals(doc, session.index, session.state) };
  const moved = arrive(doc, place(doc, session, index, step));
  return { ...moved, history: [...session.history, entry] };
}

export function back(doc: Document, session: Session): Session {
  const entry = session.history[session.history.length - 1];
  if (!entry) return advance(doc, session, -1);
  const moved = place(doc, session, entry.index, entry.step, entry.locals);
  return { ...moved, history: session.history.slice(0, -1) };
}

export function advance(doc: Document, session: Session, direction: -1 | 1): Session {
  const page = doc.pages[session.index];
  if (!page?.locked) return travel(doc, session, direction);
  if (direction === 1 && session.step < page.steps) return { ...session, step: session.step + 1 };
  if (direction === -1 && session.step > 0) return { ...session, step: session.step - 1 };
  return session;
}

export function isGame(doc: Document) {
  return doc.meta.mode === "game";
}

function travel(doc: Document, session: Session, direction: -1 | 1): Session {
  const page = doc.pages[session.index];
  if (!page) return session;
  if (direction === 1 && session.step < page.steps) return { ...session, step: session.step + 1 };
  if (direction === -1 && session.step > 0) return { ...session, step: session.step - 1 };
  if (page.hidden && session.history.length) return back(doc, session);
  if (direction === 1 && page.next) {
    const index = indexOf(doc, page.next);
    if (index >= 0) return arrive(doc, place(doc, session, index, 0));
  }
  const index = linear(doc, session.index, direction);
  if (index < 0) return session;
  return arrive(doc, place(doc, session, index, direction === 1 ? 0 : (doc.pages[index]?.steps ?? 0)));
}

export function runActions(doc: Document, session: Session, actions: Action[]): Session {
  let current = session;
  for (const action of actions) {
    switch (action.type) {
      case "set":
      case "toggle":
      case "add":
      case "remove":
      case "clear":
      case "play":
      case "stop":
        current = changeState(current, action);
        break;
      case "go":
        current = goTo(doc, current, action.target);
        break;
      case "back":
        current = back(doc, current);
        break;
      case "next":
        current = travel(doc, current, 1);
        break;
      case "prev":
        current = travel(doc, current, -1);
        break;
    }
  }
  return current;
}

export function tick(doc: Document, session: Session): Session {
  const page = doc.pages[session.index];
  if (!page?.clock) return session;
  const ticked = { ...session, elapsed: session.elapsed + 1 };
  const reached = page.timeout !== undefined && session.elapsed < page.timeout && ticked.elapsed >= page.timeout;
  return reached && page.onTimeout?.length ? runActions(doc, ticked, page.onTimeout) : ticked;
}

export function linearPages(doc: Document) {
  return doc.pages.map((page, index) => ({ page, index })).filter(({ page }) => !page.hidden);
}

const randomParam = "_rng";

export function sameKind(initial: Scalar | undefined, value: Scalar) {
  if (initial === undefined) return false;
  if (Array.isArray(initial) || Array.isArray(value)) return Array.isArray(initial) && Array.isArray(value);
  return typeof initial === typeof value;
}

function sameValue(a: Scalar, b: Scalar) {
  if (Array.isArray(a) && Array.isArray(b)) return a.length === b.length && a.every((item, index) => item === b[index]);
  return a === b;
}

function decodeList(raw: string): string[] | null {
  try {
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed) && parsed.every((item) => typeof item === "string") ? parsed : null;
  } catch {
    return null;
  }
}

export function sessionHash(doc: Document, session: Session) {
  const page = doc.pages[session.index];
  if (!page) return "";
  const params = new URLSearchParams();
  for (const [name, value] of Object.entries(session.state)) {
    const initial = doc.state?.[name];
    if (initial === undefined || sameValue(initial, value) || doc.externs?.includes(name)) continue;
    params.set(name, Array.isArray(value) ? JSON.stringify(value) : String(value));
  }
  if (doc.meta?.random) params.set(randomParam, String(session.rng));
  const query = params.toString();
  return `#/${encodeURIComponent(page.id)}${session.step ? `/${session.step}` : ""}${query ? `?${query}` : ""}`;
}

export function parseSessionHash(doc: Document, hash: string): Session | null {
  const match = /^#\/([^/?]+)(?:\/(\d+))?(?:\?(.*))?$/.exec(hash);
  if (!match) return null;
  const index = indexOf(doc, decodeURIComponent(match[1]!));
  if (index < 0) return null;
  const state: State = {};
  const params = new URLSearchParams(match[3] ?? "");
  const seed = Number(params.get(randomParam));
  for (const [name, raw] of params) {
    const initial = doc.state?.[name];
    if (doc.externs?.includes(name)) continue;
    if (Array.isArray(initial)) {
      const items = decodeList(raw);
      if (items) state[name] = items;
    } else if (typeof initial === "boolean") state[name] = raw === "true";
    else if (typeof initial === "number" && Number.isFinite(Number(raw))) state[name] = Number(raw);
    else if (typeof initial === "string") state[name] = raw;
  }
  const restoredSeed = Number.isInteger(seed) && seed >= 0 && seed < 4294967296 && params.has(randomParam) ? seed : undefined;
  return startSession(doc, index, Number(match[2] ?? 0), state, true, restoredSeed);
}
