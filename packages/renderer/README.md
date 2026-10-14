# @ribaltalang/renderer

React renderer for [Ribalta](https://viniciusadm.github.io/ribalta/) documents. It draws the JSON IR produced by [`@ribaltalang/compiler`](https://www.npmjs.com/package/@ribaltalang/compiler) or the [`ribalta`](https://crates.io/crates/ribalta) crate: themes, layouts, charts, icons, reveal steps and state.

```sh
npm install @ribaltalang/renderer react
```

## Usage

```tsx
import "@ribaltalang/renderer/styles.css";
import { RendererHostContext, type RendererHost } from "@ribaltalang/renderer/host";
import { PageView, ScaledPage } from "@ribaltalang/renderer/PageView";

const host: RendererHost = {
  asset: (id) => `/files/${id}`,
};

export function Screen({ doc, state }) {
  const page = doc.pages[0];
  return (
    <RendererHostContext.Provider value={host}>
      <div style={{ width: "100vw", height: "100vh" }}>
        <ScaledPage width={page.width} height={page.height}>
          <PageView doc={doc} page={page} state={state} />
        </ScaledPage>
      </div>
    </RendererHostContext.Provider>
  );
}
```

- Import `@ribaltalang/renderer/styles.css` once. It brings the base styles and the bundled fonts.
- The host tells the renderer where `asset:id` images and sounds live (`asset`) and, optionally, where to load fonts the renderer does not bundle (`fontStylesheet`). Without a host, assets stay empty and only bundled fonts are used.
- `state` overrides the document's state, which is how a page feeds values declared with `@extern`. Validate them with `externState` from [`@ribaltalang/player`](https://www.npmjs.com/package/@ribaltalang/player).
- `PageView` draws one page and takes `step` (reveal step), `elapsed` (seconds on the page) and `onAction` (click actions) for interactive documents.
- Every page has its own `width` and `height`. A `null` height means the page grows with its content: `PageView` drops the fixed height and `ScaledPage` scales it to the available width (`fit="width"`) instead of fitting it whole (`fit="contain"`, the default for fixed pages).
- `DocumentView` (from `@ribaltalang/renderer/DocumentView`) stacks every visible page scaled to the width, for reading a whole document as one scrolling column.
- `StaticPage` draws a page as a self-contained SVG, for thumbnails and static pages; `PrintPages` prints every page at its own size through `window.print()`.

## HTML on the server

`@ribaltalang/renderer/html` turns a compiled document into an HTML string in Node, with no DOM, for static sites and prerendering:

```ts
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { instantiateCompiler } from "@ribaltalang/compiler";
import { googleFontsQuery } from "@ribaltalang/renderer/fonts";
import { documentFonts, renderToHtml } from "@ribaltalang/renderer/html";
import type { RendererHost } from "@ribaltalang/renderer/host";

const wasm = createRequire(import.meta.url).resolve("@ribaltalang/compiler/ribalta.wasm");
const compile = await instantiateCompiler(await readFile(wasm));
const { document, diagnostics } = compile(await readFile("post.ribalta", "utf8"));

const host: RendererHost = {
  asset: (id) => `/media/${id}`,
  fontStylesheet: (families) => `https://fonts.googleapis.com/css2?${googleFontsQuery(families)}&display=swap`,
};
const html = renderToHtml(document, { locale: "pt", host, fluid: true });
const fonts = documentFonts(document, host);
```

- The result is an `<article class="ribalta-html">` with one `<section class="ribalta-html-page">` per page, plus the theme's animation styles. Serve it together with `@ribaltalang/renderer/styles.css`, and add a `<link rel="stylesheet">` for `fonts` when it is not `null`.
- `fluid` lets pages with an automatic height (`@page 720xauto`) shrink to the available width, up to their own width, so the text reflows on small screens. Fixed-height pages keep their size.
- `meta.date` and `meta.description` come from `@date` and `@description`, ready for listings and `<meta>` tags.
- The output is static: `@state`, clicks and reveal steps show their initial state with every step visible. Use `PageView` on the client for interactive documents.

## License

MIT or Apache-2.0, at your option.
