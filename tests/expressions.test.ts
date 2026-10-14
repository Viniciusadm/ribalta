import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import type { Inline, Scalar } from "@ribaltalang/compiler";
import { evaluate, formatScalar } from "@ribaltalang/renderer/expr";
import { testCompiler } from "./compiler";

interface Fixture {
  expr: string;
  values: Record<string, Scalar>;
  expected: string;
}

function literal(value: Scalar): string {
  if (Array.isArray(value)) return `[${value.map((item) => JSON.stringify(item)).join(", ")}]`;
  return typeof value === "string" ? JSON.stringify(value) : String(value);
}

const fixtures: Fixture[] = JSON.parse(readFileSync(resolve(__dirname, "../crates/ribalta/tests/fixtures/expressions.json"), "utf8"));

function rendered(content: Inline[], state: Record<string, Scalar>) {
  return content.map((inline) => (inline.type === "text" ? inline.value : inline.type === "expr" ? formatScalar(evaluate(inline.expr, { state, step: 0 })) : "")).join("");
}

describe("runtime expressions", () => {
  it("agree with the compiler on every shared fixture", async () => {
    const compile = await testCompiler();
    for (const fixture of fixtures) {
      const definitions = Object.entries(fixture.values)
        .map(([name, value]) => `@state ${name} = ${literal(value)}`)
        .join("\n");
      const { document, diagnostics } = compile(`${definitions}\n\n# $(${fixture.expr})`);
      expect(diagnostics.filter((diagnostic) => diagnostic.code !== "R608"), fixture.expr).toEqual([]);
      const heading = document.pages[0]!.children[0]!;
      expect(heading.type).toBe("heading");
      if (heading.type === "heading") expect(rendered(heading.content, document.state ?? {}), fixture.expr).toBe(fixture.expected);
    }
  });
});
