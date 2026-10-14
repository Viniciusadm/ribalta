mod attrs;
mod block;
mod chart;
mod diagnostic;
mod document;
mod eval;
mod expr;
mod format;
pub mod icons;
pub mod highlight;
mod inline;
pub mod ir;
pub mod library;
mod lower;
mod style;
mod text;
mod theme;
mod value;
#[cfg(target_arch = "wasm32")]
mod wasm;

use serde::{Deserialize, Serialize};

pub use diagnostic::{Diagnostic, Severity};
pub use format::{format_source, same_result};
pub use ir::{Document, PageSize};
pub use text::Locale;

use diagnostic::Diagnostics;
use ir::Meta;
use lower::Lowerer;
use text::{tr, Line};

pub const LANGUAGE_VERSION: u32 = 2;

pub fn block_names() -> &'static [&'static str] {
    lower::BUILTIN_BLOCKS
}

pub fn directive_names() -> &'static [&'static str] {
    block::PREAMBLE_DIRECTIVES
}

pub fn attribute_names() -> Vec<&'static str> {
    lower::attribute_names()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Compilation {
    pub document: Document,
    pub diagnostics: Vec<Diagnostic>,
}

impl Compilation {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

pub fn compile(source: &str) -> Compilation {
    compile_with_prelude("", source)
}

pub fn compile_with_prelude(prelude: &str, source: &str) -> Compilation {
    compile_localized(prelude, source, Locale::Pt)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CompileOptions {
    pub page: PageSize,
}

pub fn parse_page_size(text: &str) -> Option<PageSize> {
    lower::page_size(text, PageSize::DEFAULT)
}

pub fn compile_localized(prelude: &str, source: &str, locale: Locale) -> Compilation {
    compile_with_options(prelude, source, locale, CompileOptions::default())
}

pub fn compile_with_options(prelude: &str, source: &str, locale: Locale, options: CompileOptions) -> Compilation {
    text::with_locale(locale, || compile_inner(prelude, source, options))
}

fn compile_inner(prelude: &str, source: &str, options: CompileOptions) -> Compilation {
    let mut prelude_diags = Diagnostics::default();
    let prelude_lines = lines_of(prelude);
    let (prelude_preamble_lines, _) = document::split(&prelude_lines);
    let prelude_preamble = document::parse_preamble(&prelude_preamble_lines, &mut prelude_diags);

    let mut diags = Diagnostics::default();
    let lines = lines_of(source);
    let (preamble_lines, page_sources) = document::split(&lines);
    let preamble = document::parse_preamble(&preamble_lines, &mut diags);
    let prelude = (!prelude.trim().is_empty()).then_some(&prelude_preamble);
    let mut lowerer = Lowerer::new(options.page, prelude, &preamble, &mut diags);
    lowerer.total_pages = page_sources.len();
    let mut pages = Vec::with_capacity(page_sources.len());
    for (index, source) in page_sources.iter().enumerate() {
        pages.push(lowerer.page(index, source));
    }
    lowerer.check_regions();
    lowerer.finish(&pages);
    let theme = lowerer.theme_ir();
    let state = lowerer.document_state();
    let (page, mode, random) = (lowerer.page, lowerer.mode, lowerer.uses_random);
    let document = Document {
        version: LANGUAGE_VERSION,
        meta: Meta { title: preamble.title.clone(), author: preamble.author.clone(), page, mode, random },
        theme,
        state,
        externs: std::mem::take(&mut lowerer.externs),
        icons: icons::used_by(&pages),
        persist: std::mem::take(&mut lowerer.persist),
        scorm: lowerer.scorm.take(),
        pages,
    };
    Compilation { document, diagnostics: diags.finish(source) }
}

pub fn check_prelude(prelude: &str) -> Vec<Diagnostic> {
    check_prelude_localized(prelude, Locale::Pt)
}

pub fn check_prelude_localized(prelude: &str, locale: Locale) -> Vec<Diagnostic> {
    text::with_locale(locale, || check_prelude_inner(prelude))
}

fn check_prelude_inner(prelude: &str) -> Vec<Diagnostic> {
    let mut diags = Diagnostics::default();
    let lines = lines_of(prelude);
    let (preamble_lines, page_sources) = document::split(&lines);
    let preamble = document::parse_preamble(&preamble_lines, &mut diags);
    for (name, value) in [("title", &preamble.title), ("author", &preamble.author)] {
        if value.is_some()
            && let Some(line) = preamble_lines.iter().find(|line| line.trimmed().starts_with(&format!("@{name}"))) {
                diags.warning("R520", line.full_span(), tr(format!("`@{name}` no tema do projeto é ignorado; defina-o em cada documento"), format!("`@{name}` in the project theme is ignored; set it in each document")));
            }
    }
    if let Some(line) = page_sources.iter().flat_map(|page| page.separator.iter().chain(page.lines.iter())).find(|line| !line.is_blank() && !line.trimmed().starts_with("//")) {
        diags
            .error("R519", line.full_span(), tr("o tema do projeto só pode ter definições, não páginas", "the project theme can only contain definitions, not pages"))
            .hint(tr("Use apenas definições: @theme, @color, @font, @style, @component, @let, @data, @state, @extern, @transition, @mode, @header, @footer e @overlay", "Use only definitions: @theme, @color, @font, @style, @component, @let, @data, @state, @extern, @transition, @mode, @header, @footer and @overlay"));
    }
    Lowerer::new(PageSize::DEFAULT, None, &preamble, &mut diags).check_regions();
    diags.finish(prelude)
}

fn lines_of(source: &str) -> Vec<Line<'_>> {
    source.lines().enumerate().map(|(index, text)| Line { number: index + 1, offset: 0, text }).collect()
}

pub fn compile_to_json(source: &str) -> String {
    serde_json::to_string(&compile(source)).expect("compilation output is always serializable")
}

pub fn compile_localized_to_json(prelude: &str, source: &str, locale: Locale) -> String {
    serde_json::to_string(&compile_localized(prelude, source, locale)).expect("compilation output is always serializable")
}

pub fn compile_with_options_to_json(prelude: &str, source: &str, locale: Locale, options_json: &str) -> String {
    let options = serde_json::from_str(options_json).unwrap_or_default();
    serde_json::to_string(&compile_with_options(prelude, source, locale, options)).expect("compilation output is always serializable")
}

pub fn check_prelude_to_json(prelude: &str, locale: Locale) -> String {
    serde_json::to_string(&check_prelude_localized(prelude, locale)).expect("diagnostics are always serializable")
}

pub fn definitions_to_json(source: &str) -> String {
    serde_json::to_string(&library::definitions(source)).expect("definitions are always serializable")
}

pub fn extract_to_json(source: &str, roots_json: &str) -> String {
    let roots: Vec<library::DefRef> = serde_json::from_str(roots_json).unwrap_or_default();
    serde_json::to_string(&library::extract(source, &roots)).expect("extraction is always serializable")
}

pub fn conflicts_to_json(theme: &str, snippet: &str) -> String {
    serde_json::to_string(&library::conflicts(theme, snippet)).expect("conflicts are always serializable")
}

pub fn merge_to_json(theme: &str, snippet: &str, resolutions_json: &str, label: &str) -> String {
    let Ok(resolutions) = serde_json::from_str::<Vec<library::Resolution>>(resolutions_json) else {
        return serde_json::json!({ "ok": false, "conflicts": [], "error": "invalid resolutions" }).to_string();
    };
    let label = (!label.trim().is_empty()).then_some(label);
    match library::merge(theme, snippet, &resolutions, label) {
        Ok(merged) => serde_json::json!({ "ok": true, "source": merged.source, "renamed": merged.renamed, "skipped": merged.skipped }).to_string(),
        Err(conflicts) => serde_json::json!({ "ok": false, "conflicts": conflicts }).to_string(),
    }
}

pub fn validate_item_to_json(kind: &str, source: &str, entry: &str, locale: Locale) -> String {
    let diagnostics = match serde_json::from_value::<library::ItemKind>(serde_json::Value::String(kind.to_string())) {
        Ok(kind) => library::validate_item(kind, source, (!entry.trim().is_empty()).then_some(entry), locale),
        Err(_) => vec![Diagnostic {
            severity: Severity::Error,
            code: "R533",
            message: text::with_locale(locale, || tr(format!("tipo de item desconhecido: `{kind}`"), format!("unknown item kind: `{kind}`"))),
            line: 1,
            column: 1,
            end_column: 1,
            hint: None,
        }],
    };
    serde_json::to_string(&diagnostics).expect("diagnostics are always serializable")
}
