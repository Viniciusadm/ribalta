import { renderToStaticMarkup } from "react-dom/server";
import type { Document, Locale } from "@ribaltalang/compiler";
import { fontStylesheetUrl } from "./fonts";
import { RendererHostContext, type RendererHost } from "./host";
import { LocaleContext, defaultLocale } from "./locale";
import { PageView } from "./PageView";

export interface HtmlOptions {
  locale?: Locale;
  host?: RendererHost;
  fluid?: boolean;
  scale?: number;
}

export function renderToHtml(doc: Document, { locale = defaultLocale, host, fluid = false, scale = 1 }: HtmlOptions = {}) {
  return renderToStaticMarkup(
    <LocaleContext.Provider value={locale}>
      <RendererHostContext.Provider value={host ?? null}>
        <article className="ribalta-html" style={scale === 1 ? undefined : { zoom: scale }}>
          {doc.pages.map((page, index) => (
            <section key={index} className="ribalta-html-page">
              <PageView doc={doc} page={page} fluid={fluid} />
            </section>
          ))}
        </article>
      </RendererHostContext.Provider>
    </LocaleContext.Provider>,
  );
}

export function documentFonts(doc: Document, host?: RendererHost | null) {
  return fontStylesheetUrl(doc.theme.fontFamilies, host ?? null);
}
