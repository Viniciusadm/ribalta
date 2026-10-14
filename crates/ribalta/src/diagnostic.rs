use serde::Serialize;

use crate::text::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub end_column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Pending {
    severity: Severity,
    code: &'static str,
    message: String,
    span: Span,
    hint: Option<String>,
}

impl Pending {
    pub fn hint(&mut self, hint: impl Into<String>) -> &mut Self {
        self.hint = Some(hint.into());
        self
    }
}

#[derive(Debug, Default)]
pub struct Diagnostics {
    items: Vec<Pending>,
    origin: Option<(Span, String)>,
}

impl Diagnostics {
    pub fn error(&mut self, code: &'static str, span: Span, message: impl Into<String>) -> &mut Pending {
        self.push(Severity::Error, code, span, message.into())
    }

    pub fn warning(&mut self, code: &'static str, span: Span, message: impl Into<String>) -> &mut Pending {
        self.push(Severity::Warning, code, span, message.into())
    }

    pub fn set_origin(&mut self, origin: Option<(Span, String)>) -> Option<(Span, String)> {
        std::mem::replace(&mut self.origin, origin)
    }

    pub fn has_origin(&self) -> bool {
        self.origin.is_some()
    }

    pub fn absorb(&mut self, other: Diagnostics) {
        for item in other.items {
            let Pending { severity, code, message, span, hint } = item;
            let pending = self.push(severity, code, span, message);
            pending.hint = hint;
        }
    }

    fn push(&mut self, severity: Severity, code: &'static str, span: Span, message: String) -> &mut Pending {
        let (span, message) = match &self.origin {
            Some((origin, component)) => (*origin, crate::text::tr(format!("{message} (no componente `{component}`)"), format!("{message} (in component `{component}`)"))),
            None => (span, message),
        };
        self.items.push(Pending { severity, code, message, span, hint: None });
        self.items.last_mut().expect("just pushed")
    }

    pub fn finish(self, source: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = source.lines().collect();
        let mut result: Vec<Diagnostic> = self
            .items
            .into_iter()
            .map(|item| {
                let text = lines.get(item.span.line.saturating_sub(1)).copied().unwrap_or("");
                Diagnostic {
                    severity: item.severity,
                    code: item.code,
                    message: item.message,
                    line: item.span.line.max(1),
                    column: utf16_column(text, item.span.from) + 1,
                    end_column: utf16_column(text, item.span.to.max(item.span.from)) + 1,
                    hint: item.hint,
                }
            })
            .collect();
        result.sort_by_key(|item| (item.line, item.column));
        result.dedup();
        result
    }
}

fn utf16_column(text: &str, byte: usize) -> usize {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    text[..byte].encode_utf16().count()
}
