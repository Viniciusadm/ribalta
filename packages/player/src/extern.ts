import type { Document, Field, Row, Scalar } from "@ribaltalang/compiler";
import { sameKind, type Session, type State } from "./navigation";

export interface ExternValues {
  state: State;
  ignored: string[];
}

function isField(value: unknown): value is Field {
  return typeof value === "boolean" || typeof value === "number" || typeof value === "string";
}

function isRow(value: unknown): value is Row {
  return typeof value === "object" && value !== null && !Array.isArray(value) && Object.values(value).every(isField);
}

function isScalar(value: unknown): value is Scalar {
  if (Array.isArray(value)) return value.every((item) => typeof item === "string") || value.every(isRow);
  return isField(value);
}

function matchesExample(example: Scalar | undefined, value: Scalar) {
  if (!sameKind(example, value)) return false;
  if (!Array.isArray(example) || !Array.isArray(value)) return true;
  const shape = example[0];
  if (shape === undefined || typeof shape !== "object") return value.every((item) => typeof item === "string");
  return value.every((row) => isRow(row) && Object.entries(shape).every(([key, field]) => typeof row[key] === typeof field));
}

export function externState(doc: Document, values: Record<string, unknown>): ExternValues {
  const declared = new Set(doc.externs ?? []);
  const state: State = {};
  const ignored: string[] = [];
  for (const [name, value] of Object.entries(values)) {
    if (declared.has(name) && isScalar(value) && matchesExample(doc.state?.[name], value)) state[name] = value;
    else ignored.push(name);
  }
  return { state, ignored };
}

export function withExterns(session: Session, state: State): Session {
  return { ...session, state: { ...session.state, ...state } };
}
