// @vitest-environment node
import { beforeAll, describe, expect, it } from "vitest";
import type { Compiler, PageSize } from "@ribaltalang/compiler";
import { documentFonts, renderToHtml } from "@ribaltalang/renderer/html";
import { testCompiler } from "./compiler";

let compile: Compiler;

beforeAll(async () => {
  compile = await testCompiler();
});

function documentOf(source: string, page: PageSize = { width: 720, height: null }) {
  const { document, diagnostics } = compile(source, "", "pt", { page });
  expect(diagnostics).toEqual([]);
  return document;
}

describe("renderToHtml", () => {
  it("renders without a DOM", () => {
    expect(typeof globalThis.document).toBe("undefined");
    const html = renderToHtml(documentOf("@title Post\n\n# Olá\n\nTexto com **negrito**."));
    expect(html).toContain('<article class="ribalta-html">');
    expect(html).toContain("Olá");
    expect(html).toContain("<strong>negrito</strong>");
  });

  it("renders one section per page and the motion styles once", () => {
    const html = renderToHtml(documentOf("# Um\n---\n# Dois\n---\n# Três"));
    expect(html.match(/<section class="ribalta-html-page">/g)).toHaveLength(3);
    expect(html.match(/<style /g)).toHaveLength(1);
  });

  it("keeps the page width fixed unless fluid", () => {
    const doc = documentOf("# Olá");
    expect(renderToHtml(doc)).toContain("width:720px");
    const fluid = renderToHtml(doc, { fluid: true });
    expect(fluid).toContain("width:100%");
    expect(fluid).toContain("max-width:720px");
  });

  it("ignores fluid on fixed-height pages", () => {
    const html = renderToHtml(documentOf("# Olá", { width: 1920, height: 1080 }), { fluid: true });
    expect(html).toContain("width:1920px");
    expect(html).not.toContain("max-width");
  });

  it("resolves stored images through the host", () => {
    const id = "3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c";
    const html = renderToHtml(documentOf(`![gráfico](asset:${id})`), { host: { asset: (asset) => `/media/${asset}.png` } });
    expect(html).toContain(`src="/media/${id}.png"`);
  });

  it("asks the host for the theme fonts", () => {
    const doc = documentOf('@font titulo "Bebas Neue" 400\n\n# Oi {font=titulo}');
    expect(documentFonts(doc)).toBeNull();
    expect(documentFonts(doc, { asset: (id) => id, fontStylesheet: (families) => families.map(({ family }) => family).join(",") })).toBe("Bebas Neue");
  });
});
