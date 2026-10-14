import { describe, expect, it } from "vitest";
import expressions from "../crates/ribalta/conformance/expressions.json";
import navigation from "../crates/ribalta/conformance/navigation.json";
import type { Action, Document, Expr, Scalar } from "@ribaltalang/compiler";
import { advance, back, goTo, indexOf, runActions, startSession, tick, type Session } from "@ribaltalang/player/navigation";
import { evaluate, formatScalar, nextRandom } from "@ribaltalang/renderer/expr";

interface ExpressionCase {
  name: string;
  expr: unknown;
  state?: Record<string, Scalar>;
  step?: number;
  seed?: number;
  elapsed?: number;
  value: Scalar;
  printed: string;
}

type Operation = { op: "next" } | { op: "prev" } | { op: "back" } | { op: "tick" } | { op: "go"; target: string } | { op: "actions"; actions: unknown[] };

interface NavigationCase {
  name: string;
  document: keyof typeof navigation.documents;
  start: { page: string; step?: number; seed?: number };
  ops: Operation[];
  expect: { page: string; step: number; elapsed?: number; state?: Record<string, Scalar> }[];
}

function apply(doc: Document, session: Session, operation: Operation) {
  switch (operation.op) {
    case "next":
      return advance(doc, session, 1);
    case "prev":
      return advance(doc, session, -1);
    case "back":
      return back(doc, session);
    case "tick":
      return tick(doc, session);
    case "go":
      return goTo(doc, session, operation.target);
    case "actions":
      return runActions(doc, session, operation.actions as Action[]);
  }
}

describe("conformance: expressions", () => {
  it.each((expressions.cases as ExpressionCase[]).map((item) => [item.name, item] as const))("%s", (_, item) => {
    let seed = item.seed ?? 0;
    const draw = () => {
      const result = nextRandom(seed);
      seed = result.seed;
      return result.value;
    };
    const value = evaluate(item.expr as Expr, { state: item.state ?? {}, step: item.step ?? 0, draw, elapsed: item.elapsed ?? 0 });
    expect(value).toEqual(item.value);
    expect(formatScalar(value)).toBe(item.printed);
  });
});

describe("conformance: navigation", () => {
  it.each((navigation.cases as NavigationCase[]).map((item) => [item.name, item] as const))("%s", (_, item) => {
    const doc = navigation.documents[item.document] as unknown as Document;
    let session = startSession(doc, indexOf(doc, item.start.page), item.start.step ?? 0, {}, false, item.start.seed);
    item.ops.forEach((operation, index) => {
      session = apply(doc, session, operation);
      const expected = item.expect[index]!;
      expect({ page: doc.pages[session.index]?.id, step: session.step }, `after operation ${index + 1}`).toEqual({ page: expected.page, step: expected.step });
      if (expected.state) expect(session.state, `state after operation ${index + 1}`).toEqual(expected.state);
      if (expected.elapsed !== undefined) expect(session.elapsed, `elapsed after operation ${index + 1}`).toBe(expected.elapsed);
    });
  });
});
