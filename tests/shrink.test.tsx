import { beforeAll, describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import type { Compiler } from "@ribaltalang/compiler";
import { PageView } from "@ribaltalang/renderer/PageView";
import { renderToHtml } from "@ribaltalang/renderer/html";
import { fitShrink, searchShrink, shrinkFontSize } from "@ribaltalang/renderer/shrink";
import { testCompiler } from "./compiler";

let compile: Compiler;

beforeAll(async () => {
  compile = await testCompiler();
});

function documentOf(source: string) {
  const { document, diagnostics } = compile(source, "", "pt", { page: { width: 1920, height: 1080 } });
  expect(diagnostics).toEqual([]);
  return document;
}

function fakeText(height: number, limit: number) {
  const applied: number[] = [];
  return {
    applied,
    apply: (factor: number) => applied.push(factor),
    overflows: () => height * applied[applied.length - 1]! > limit,
  };
}

describe("searchShrink", () => {
  it("keeps the full size when the text already fits", () => {
    const text = fakeText(500, 1000);
    expect(searchShrink(text.apply, text.overflows)).toBe(1);
    expect(text.applied).toEqual([1]);
  });

  it("finds the largest factor that fits and leaves it applied", () => {
    const text = fakeText(1000, 600);
    const factor = searchShrink(text.apply, text.overflows);
    expect(factor).toBeLessThanOrEqual(0.6);
    expect(factor).toBeGreaterThan(0.59);
    expect(text.applied.at(-1)).toBe(factor);
  });

  it("stops at the minimum when nothing fits", () => {
    const text = { applied: [] as number[] };
    const factor = searchShrink(
      (value) => text.applied.push(value),
      () => true,
    );
    expect(factor).toBe(0);
    expect(text.applied).toEqual([1, 0]);
  });
});

describe("shrinkFontSize", () => {
  it("scales the declared size down to the minimum", () => {
    expect(shrinkFontSize(150, 64)).toBe("max(64px, calc(150px * var(--ribalta-shrink, 1)))");
  });

  it("never grows past the declared size", () => {
    expect(shrinkFontSize(40, 64)).toBe("max(40px, calc(40px * var(--ribalta-shrink, 1)))");
  });

  it("follows the inherited size when none is declared", () => {
    expect(shrinkFontSize(undefined, 20)).toBe("max(20px, calc(1em * var(--ribalta-shrink, 1)))");
  });
});

describe("shrink in the renderer", () => {
  it("uses the theme heading size as the starting point", () => {
    const doc = documentOf("@theme ink\n\n# Título {shrink=xl}");
    const html = renderToHtml(doc);
    const heading = doc.theme.headings[0]?.size;
    expect(heading).toBeTypeOf("number");
    expect(html).toContain(`font-size:max(52px, calc(${heading}px * var(--ribalta-shrink, 1)))`);
  });

  it("uses the declared size on paragraphs and spans", () => {
    const html = renderToHtml(documentOf("Texto com [destaque]{size=5xl shrink=2xl} {size=3xl shrink}"));
    expect(html).toContain("font-size:max(20px, calc(88px * var(--ribalta-shrink, 1)))");
    expect(html).toContain("font-size:max(64px, calc(150px * var(--ribalta-shrink, 1)))");
  });

  it("leaves pages without shrink untouched", () => {
    const doc = documentOf("# Título");
    const { container } = render(<PageView doc={doc} page={doc.pages[0]!} />);
    const page = container.querySelector<HTMLElement>(".ribalta-page")!;
    expect(page.style.getPropertyValue("--ribalta-shrink")).toBe("");
    expect(fitShrink(page)).toBe(1);
  });

  it("sets the factor on pages with shrink", () => {
    const doc = documentOf("# Título {shrink}");
    const { container } = render(<PageView doc={doc} page={doc.pages[0]!} />);
    const page = container.querySelector<HTMLElement>(".ribalta-page")!;
    expect(page.style.getPropertyValue("--ribalta-shrink")).toBe("1");
  });
});
