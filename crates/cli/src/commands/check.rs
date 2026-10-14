use reqwest::Url;
use serde::Serialize;

use crate::Outcome;
use crate::args::{CheckArgs, Lang};
use crate::commands::assets;
use crate::source::{display_name, read_source};
use crate::diagnostics::{self, Diagnostic, Severity};
use crate::error::CliResult;
use crate::output::print_json;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report<'a> {
    file: &'a str,
    pages: usize,
    diagnostics: &'a [Diagnostic],
    #[serde(skip_serializing_if = "<[Diagnostic]>::is_empty")]
    theme_diagnostics: &'a [Diagnostic],
}

pub fn run(args: &CheckArgs, lang: Lang, asset_base: Option<&Url>) -> CliResult<Outcome> {
    let source = read_source(&args.file)?;
    let theme = args.theme.as_deref().map(read_source).transpose()?.unwrap_or_default();
    let theme_diagnostics: Vec<Diagnostic> = if theme.trim().is_empty() {
        Vec::new()
    } else {
        ribalta::check_prelude_localized(&theme, lang.locale()).iter().map(Into::into).collect()
    };
    let options = ribalta::CompileOptions { page: args.page.unwrap_or_default() };
    let compilation = ribalta::compile_with_options(&theme, &source, lang.locale(), options);
    let mut file_diagnostics: Vec<Diagnostic> = compilation.diagnostics.iter().map(Into::into).collect();
    if args.check_assets {
        let document = serde_json::to_value(&compilation.document).unwrap_or_default();
        file_diagnostics.extend(assets::check(&document, &source, asset_base, lang));
        file_diagnostics.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.column));
    }
    let file = display_name(&args.file);
    let pages = compilation.document.pages.len();

    if args.json {
        print_json(&Report { file: &file, pages, diagnostics: &file_diagnostics, theme_diagnostics: &theme_diagnostics });
    } else {
        if let Some(theme_path) = &args.theme {
            print!("{}", diagnostics::render(&theme_diagnostics, Some(&theme), &display_name(theme_path), lang));
        }
        print!("{}", diagnostics::render(&file_diagnostics, Some(&source), &file, lang));
        let all: Vec<Diagnostic> = theme_diagnostics.iter().chain(&file_diagnostics).cloned().collect();
        let unit = match (lang, pages) {
            (Lang::Pt, 1) => "página",
            (Lang::Pt, _) => "páginas",
            (Lang::En, 1) => "page",
            (Lang::En, _) => "pages",
        };
        println!("{file}: {pages} {unit} · {}", diagnostics::summary(&all, lang));
    }

    let all = || theme_diagnostics.iter().chain(&file_diagnostics);
    let errors = all().any(|diagnostic| diagnostic.severity == Severity::Error);
    let warnings = all().any(|diagnostic| diagnostic.severity == Severity::Warning);
    Ok(if errors || (args.strict && warnings) { Outcome::CompileErrors } else { Outcome::Success })
}
