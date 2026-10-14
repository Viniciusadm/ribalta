import { readFile } from "node:fs/promises";
import { renderToStaticMarkup } from "react-dom/server";
import { instantiateCompiler, type Compiler, type Diagnostic, type Locale } from "@ribaltalang/compiler";
import { icons } from "@ribaltalang/renderer/icons";
import { LocaleContext } from "@ribaltalang/renderer/locale";
import { StaticPage } from "@ribaltalang/renderer/StaticPage";

const pageSize = { page: { width: 1920, height: 1080 } };

let compiler: Promise<Compiler> | null = null;

function loadCompiler(wasmPath: string) {
  compiler ??= readFile(wasmPath).then((bytes) => instantiateCompiler(bytes));
  return compiler;
}

export interface Preview {
  html: string;
  diagnostics: Diagnostic[];
}

export async function renderPreview(wasmPath: string, source: string, locale: Locale): Promise<Preview> {
  const compile = await loadCompiler(wasmPath);
  const { document, diagnostics } = compile(source, "", locale, pageSize);
  const pages = document.pages.map((page, index) => (
    <figure key={index} className="ribalta-preview-page">
      <StaticPage doc={document} page={page} />
    </figure>
  ));
  const markup = renderToStaticMarkup(
    <LocaleContext.Provider value={locale}>
      <div className={`ribalta-preview ribalta-preview-${Math.min(document.pages.length, 3)} not-content`}>{pages}</div>
    </LocaleContext.Provider>,
  );
  return { html: markup.replace(/ href="[^"]*"/g, ""), diagnostics };
}

export function iconNames() {
  return Object.keys(icons);
}

export function renderIconGrid() {
  return renderToStaticMarkup(
    <ul className="ribalta-icon-grid not-content">
      {Object.entries(icons).map(([name, Icon]) => (
        <li key={name}>
          <Icon size={28} strokeWidth={1.75} aria-hidden />
          <code>{name}</code>
        </li>
      ))}
    </ul>,
  );
}
