use serde::{Deserialize, Serialize};

use crate::args::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    #[serde(default)]
    pub end_column: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl From<&ribalta::Diagnostic> for Diagnostic {
    fn from(value: &ribalta::Diagnostic) -> Self {
        Self {
            severity: match value.severity {
                ribalta::Severity::Error => Severity::Error,
                ribalta::Severity::Warning => Severity::Warning,
            },
            code: value.code.to_owned(),
            message: value.message.clone(),
            line: value.line,
            column: value.column,
            end_column: value.end_column,
            hint: value.hint.clone(),
        }
    }
}

pub fn count(diagnostics: &[Diagnostic], severity: Severity) -> usize {
    diagnostics.iter().filter(|diagnostic| diagnostic.severity == severity).count()
}

pub fn render(diagnostics: &[Diagnostic], source: Option<&str>, file: &str, lang: Lang) -> String {
    let lines: Vec<&str> = source.map(|source| source.lines().collect()).unwrap_or_default();
    let mut output = String::new();
    for diagnostic in diagnostics {
        let label = match (diagnostic.severity, lang) {
            (Severity::Error, Lang::Pt) => "erro",
            (Severity::Error, Lang::En) => "error",
            (Severity::Warning, Lang::Pt) => "aviso",
            (Severity::Warning, Lang::En) => "warning",
        };
        output.push_str(&format!("{label} {} · {file}:{}:{}\n", diagnostic.code, diagnostic.line, diagnostic.column));
        output.push_str(&format!("  {}\n", diagnostic.message));
        if let Some(text) = lines.get(diagnostic.line.saturating_sub(1)) {
            output.push_str(&snippet(text, diagnostic));
        }
        if let Some(hint) = &diagnostic.hint {
            let hint_label = if lang == Lang::Pt { "dica" } else { "hint" };
            output.push_str(&format!("  {hint_label}: {hint}\n"));
        }
        output.push('\n');
    }
    output
}

pub fn summary(diagnostics: &[Diagnostic], lang: Lang) -> String {
    let (errors, warnings) = (count(diagnostics, Severity::Error), count(diagnostics, Severity::Warning));
    match lang {
        Lang::Pt if errors + warnings == 0 => "nenhum problema encontrado".to_owned(),
        Lang::En if errors + warnings == 0 => "no problems found".to_owned(),
        Lang::Pt => format!("{} {}, {} {}", errors, plural(errors, "erro", "erros"), warnings, plural(warnings, "aviso", "avisos")),
        Lang::En => format!("{} {}, {} {}", errors, plural(errors, "error", "errors"), warnings, plural(warnings, "warning", "warnings")),
    }
}

pub fn plural<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}

fn snippet(text: &str, diagnostic: &Diagnostic) -> String {
    let number = diagnostic.line.to_string();
    let gutter = " ".repeat(number.len());
    let start = char_index(text, diagnostic.column.saturating_sub(1));
    let end = char_index(text, diagnostic.end_column.saturating_sub(1)).max(start + 1);
    let padding: String = text.chars().take(start).map(|char| if char == '\t' { '\t' } else { ' ' }).collect();
    let carets = "^".repeat(end - start);
    format!("  {number} | {text}\n  {gutter} | {padding}{carets}\n")
}

fn char_index(text: &str, utf16_column: usize) -> usize {
    let mut units = 0;
    for (index, char) in text.chars().enumerate() {
        if units >= utf16_column {
            return index;
        }
        units += char.len_utf16();
    }
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostic(severity: Severity, column: usize, end_column: usize) -> Diagnostic {
        Diagnostic {
            severity,
            code: "R209".into(),
            message: "atributo desconhecido `colr`".into(),
            line: 1,
            column,
            end_column,
            hint: Some("Você quis dizer `color`?".into()),
        }
    }

    #[test]
    fn renders_location_snippet_and_hint() {
        let output = render(&[diagnostic(Severity::Error, 13, 17)], Some("# Título {colr=coral}"), "a.ribalta", Lang::Pt);
        assert_eq!(
            output,
            "erro R209 · a.ribalta:1:13\n  atributo desconhecido `colr`\n  1 | # Título {colr=coral}\n    |             ^^^^\n  dica: Você quis dizer `color`?\n\n"
        );
    }

    #[test]
    fn converts_utf16_columns_to_chars() {
        assert_eq!(char_index("😀ab", 2), 1);
        assert_eq!(char_index("abc", 10), 3);
    }

    #[test]
    fn summarizes_in_both_languages() {
        let items = [diagnostic(Severity::Error, 1, 2), diagnostic(Severity::Warning, 1, 2), diagnostic(Severity::Warning, 1, 2)];
        assert_eq!(summary(&items, Lang::Pt), "1 erro, 2 avisos");
        assert_eq!(summary(&items, Lang::En), "1 error, 2 warnings");
        assert_eq!(summary(&[], Lang::Pt), "nenhum problema encontrado");
    }
}
