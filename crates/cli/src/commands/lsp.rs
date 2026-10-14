use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ribalta::library::{DefKind, Definition};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument, Notification as _, PublishDiagnostics,
};
use lsp_types::request::{Completion, DocumentSymbolRequest, Formatting, GotoDefinition, Request as _};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionTextEdit, Diagnostic as LspDiagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentFormattingParams,
    DocumentSymbolParams, GotoDefinitionParams, InitializeParams, Location, NumberOrString, OneOf, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url,
};
use serde_json::{Value, json};

use crate::args::Lang;
use crate::Outcome;
use crate::error::{CliError, CliResult};

const THEME_FILE: &str = "theme.ribalta";
const FIELDS: [&str; 7] = ["step", "elapsed", "page", "pages", "title", "author", "date"];
const ENUM_VALUES: [(&str, &[&str]); 18] = [
    ("align", &["left", "center", "right"]),
    ("valign", &["top", "center", "bottom"]),
    ("size", &["xs", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl", "6xl", "7xl"]),
    ("pad", &["none", "xs", "sm", "md", "lg", "xl", "2xl"]),
    ("gap", &["none", "xs", "sm", "md", "lg", "xl", "2xl"]),
    ("radius", &["none", "sm", "md", "lg", "xl", "full"]),
    ("shadow", &["sm", "md", "lg", "none"]),
    ("weight", &["light", "regular", "medium", "semibold", "bold", "black"]),
    ("fit", &["cover", "contain"]),
    ("type", &["bar", "line", "area", "pie", "donut"]),
    ("header", &["on", "off"]),
    ("footer", &["on", "off"]),
    ("overlay", &["on", "off"]),
    ("cover", &["true", "false"]),
    ("hidden", &["true", "false"]),
    ("enter", &["fade", "up", "down", "left", "right", "zoom", "none"]),
    ("transition", &["fade", "slide", "zoom", "none"]),
    ("nav", &["on", "off"]),
];

pub fn run(lang: Lang) -> CliResult<Outcome> {
    let (connection, threads) = Connection::stdio();
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(["@", "$", "#", ":", "{", ".", "=", " "].map(String::from).to_vec()),
            ..CompletionOptions::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    };
    let failure = |error: &dyn std::fmt::Display| CliError::new(format!("o servidor de linguagem parou: {error}"));
    let capabilities = serde_json::to_value(capabilities).map_err(|error| failure(&error))?;
    let initialize = connection.initialize(capabilities).map_err(|error| failure(&error))?;
    let params: InitializeParams = serde_json::from_value(initialize).map_err(|error| failure(&error))?;
    let theme = params
        .initialization_options
        .as_ref()
        .and_then(|options| options.get("theme"))
        .and_then(Value::as_str)
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from);
    let mut server = Server { connection: &connection, lang, theme, documents: HashMap::new() };
    server.serve().map_err(|error| failure(&error))?;
    drop(connection);
    threads.join().map_err(|error| failure(&error))?;
    Ok(Outcome::Success)
}

struct Server<'a> {
    connection: &'a Connection,
    lang: Lang,
    theme: Option<PathBuf>,
    documents: HashMap<Url, String>,
}

impl Server<'_> {
    fn serve(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        for message in &self.connection.receiver {
            match message {
                Message::Request(request) => {
                    if self.connection.handle_shutdown(&request)? {
                        return Ok(());
                    }
                    self.request(request)?;
                }
                Message::Notification(notification) => self.notification(notification)?,
                Message::Response(_) => {}
            }
        }
        Ok(())
    }

    fn request(&self, request: Request) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let id = request.id.clone();
        let result = match request.method.as_str() {
            Completion::METHOD => serde_json::from_value::<CompletionParams>(request.params).map(|params| json!(self.complete(&params))),
            GotoDefinition::METHOD => serde_json::from_value::<GotoDefinitionParams>(request.params).map(|params| json!(self.definition(&params))),
            DocumentSymbolRequest::METHOD => serde_json::from_value::<DocumentSymbolParams>(request.params).map(|params| self.symbols(&params.text_document.uri)),
            Formatting::METHOD => serde_json::from_value::<DocumentFormattingParams>(request.params).map(|params| json!(self.format(&params.text_document.uri))),
            _ => return self.reply_error(id, ErrorCode::MethodNotFound, format!("método não suportado: {}", request.method)),
        };
        match result {
            Ok(value) => self.connection.sender.send(Message::Response(Response::new_ok(id, value)))?,
            Err(error) => return self.reply_error(id, ErrorCode::InvalidParams, error.to_string()),
        }
        Ok(())
    }

    fn reply_error(&self, id: RequestId, code: ErrorCode, message: String) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.connection.sender.send(Message::Response(Response::new_err(id, code as i32, message)))?;
        Ok(())
    }

    fn notification(&mut self, notification: Notification) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(notification.params)?;
                self.documents.insert(params.text_document.uri.clone(), params.text_document.text);
                self.refresh(&params.text_document.uri)?;
            }
            DidChangeTextDocument::METHOD => {
                let mut params: DidChangeTextDocumentParams = serde_json::from_value(notification.params)?;
                if let Some(change) = params.content_changes.pop() {
                    self.documents.insert(params.text_document.uri.clone(), change.text);
                }
                self.refresh(&params.text_document.uri)?;
            }
            DidSaveTextDocument::METHOD => {
                let params: DidSaveTextDocumentParams = serde_json::from_value(notification.params)?;
                self.refresh(&params.text_document.uri)?;
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams = serde_json::from_value(notification.params)?;
                self.documents.remove(&params.text_document.uri);
                self.publish(params.text_document.uri, Vec::new())?;
            }
            _ => {}
        }
        Ok(())
    }

    fn refresh(&self, changed: &Url) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let targets: Vec<&Url> = if self.is_theme(changed) { self.documents.keys().collect() } else { vec![changed] };
        for uri in targets {
            let diagnostics = self.diagnostics(uri);
            self.publish(uri.clone(), diagnostics)?;
        }
        Ok(())
    }

    fn publish(&self, uri: Url, diagnostics: Vec<LspDiagnostic>) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let params = PublishDiagnosticsParams { uri, diagnostics, version: None };
        self.connection.sender.send(Message::Notification(Notification::new(PublishDiagnostics::METHOD.to_owned(), params)))?;
        Ok(())
    }

    fn theme_path(&self, uri: &Url) -> Option<PathBuf> {
        let path = uri.to_file_path().ok()?;
        match &self.theme {
            Some(theme) if theme.is_absolute() => Some(theme.clone()),
            Some(theme) => Some(path.parent()?.join(theme)),
            None => Some(path.parent()?.join(THEME_FILE)),
        }
    }

    fn is_theme(&self, uri: &Url) -> bool {
        let Ok(path) = uri.to_file_path() else { return false };
        self.theme_path(uri).is_some_and(|theme| same_file(&theme, &path))
    }

    fn theme_source(&self, uri: &Url) -> Option<(Url, String)> {
        if self.is_theme(uri) {
            return None;
        }
        let path = self.theme_path(uri)?;
        let theme_uri = Url::from_file_path(&path).ok()?;
        let text = self.documents.get(&theme_uri).cloned().or_else(|| std::fs::read_to_string(&path).ok())?;
        Some((theme_uri, text))
    }

    fn compile(&self, uri: &Url) -> Option<Analysis> {
        let source = self.documents.get(uri)?.clone();
        let theme = self.theme_source(uri);
        let prelude = theme.as_ref().map(|(_, text)| text.as_str()).unwrap_or_default();
        let compilation = ribalta::compile_localized(prelude, &source, self.lang.locale());
        let document = serde_json::to_value(&compilation.document).unwrap_or(Value::Null);
        Some(Analysis { definitions: ribalta::library::definitions(&source), theme_definitions: theme.as_ref().map(|(_, text)| ribalta::library::definitions(text)).unwrap_or_default(), theme_uri: theme.map(|(uri, _)| uri), document, source })
    }

    fn diagnostics(&self, uri: &Url) -> Vec<LspDiagnostic> {
        let Some(source) = self.documents.get(uri) else { return Vec::new() };
        let found = if self.is_theme(uri) {
            ribalta::check_prelude_localized(source, self.lang.locale())
        } else {
            let prelude = self.theme_source(uri).map(|(_, text)| text).unwrap_or_default();
            ribalta::compile_localized(&prelude, source, self.lang.locale()).diagnostics
        };
        found.iter().map(|diagnostic| convert(diagnostic, self.lang)).collect()
    }

    fn complete(&self, params: &CompletionParams) -> Vec<CompletionItem> {
        let position = params.text_document_position.position;
        let Some(analysis) = self.compile(&params.text_document_position.text_document.uri) else { return Vec::new() };
        let line = analysis.source.lines().nth(position.line as usize).unwrap_or_default();
        let before = prefix_utf16(line, position.character as usize);
        let Some((partial, options)) = completion_options(before, &analysis) else { return Vec::new() };
        let start = Position { line: position.line, character: position.character.saturating_sub(partial.encode_utf16().count() as u32) };
        options
            .into_iter()
            .map(|(label, kind, detail)| CompletionItem {
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range: Range { start, end: position }, new_text: label.clone() })),
                label,
                kind: Some(kind),
                detail,
                ..CompletionItem::default()
            })
            .collect()
    }

    fn definition(&self, params: &GotoDefinitionParams) -> Option<Location> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let analysis = self.compile(uri)?;
        let line = analysis.source.lines().nth(position.line as usize)?;
        let (sigil, word) = word_at(line, position.character as usize)?;
        let kind = match sigil {
            Some('#') => {
                let page = analysis.pages().into_iter().find(|page| page.id == word)?;
                return Some(Location { uri: uri.clone(), range: line_range(page.line) });
            }
            Some('$') => DefKind::Variable,
            Some('.') => DefKind::Style,
            _ if line.trim_start().starts_with("::") => DefKind::Component,
            _ => DefKind::Style,
        };
        let found = |definitions: &[Definition]| definitions.iter().find(|definition| definition.kind == kind && definition.name == word).map(|definition| definition.start_line);
        if let Some(line) = found(&analysis.definitions) {
            return Some(Location { uri: uri.clone(), range: line_range(line as u64) });
        }
        let line = found(&analysis.theme_definitions)?;
        Some(Location { uri: analysis.theme_uri?, range: line_range(line as u64) })
    }

    fn symbols(&self, uri: &Url) -> Value {
        let Some(analysis) = self.compile(uri) else { return json!([]) };
        let mut symbols: Vec<Value> = analysis
            .definitions
            .iter()
            .filter(|definition| !matches!(definition.kind, DefKind::Setting))
            .map(|definition| {
                let kind = match definition.kind {
                    DefKind::Variable => 13,
                    DefKind::Color => 14,
                    DefKind::Style => 23,
                    DefKind::Component => 12,
                    _ => 3,
                };
                symbol(&definition.name, kind, definition.start_line as u64, definition.end_line.max(definition.start_line) as u64)
            })
            .collect();
        if !self.is_theme(uri) {
            symbols.extend(analysis.pages().into_iter().map(|page| {
                let name = if page.title.trim().is_empty() { format!("#{}", page.id) } else { format!("#{} · {}", page.id, page.title) };
                symbol(&name, 5, page.line, page.end_line.max(page.line))
            }));
        }
        Value::Array(symbols)
    }

    fn format(&self, uri: &Url) -> Vec<TextEdit> {
        let Some(source) = self.documents.get(uri) else { return Vec::new() };
        let formatted = ribalta::format_source(source);
        let prelude = self.theme_source(uri).map(|(_, text)| text).unwrap_or_default();
        if formatted == *source || (!self.is_theme(uri) && !ribalta::same_result(&prelude, source, &formatted)) {
            return Vec::new();
        }
        let lines = source.lines().count() as u32 + 1;
        vec![TextEdit { range: Range { start: Position { line: 0, character: 0 }, end: Position { line: lines, character: 0 } }, new_text: formatted }]
    }
}

struct Analysis {
    source: String,
    document: Value,
    definitions: Vec<Definition>,
    theme_definitions: Vec<Definition>,
    theme_uri: Option<Url>,
}

struct PageInfo {
    id: String,
    title: String,
    line: u64,
    end_line: u64,
}

impl Analysis {
    fn pages(&self) -> Vec<PageInfo> {
        self.document["pages"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|page| PageInfo {
                id: page["id"].as_str().unwrap_or_default().to_owned(),
                title: page["title"].as_str().unwrap_or_default().to_owned(),
                line: page["line"].as_u64().unwrap_or(1),
                end_line: page["endLine"].as_u64().unwrap_or(1),
            })
            .collect()
    }

    fn named(&self, kind: DefKind) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for definition in self.definitions.iter().chain(&self.theme_definitions) {
            if definition.kind == kind && !names.contains(&definition.name) {
                names.push(definition.name.clone());
            }
        }
        names
    }

    fn theme_keys(&self, field: &str) -> Vec<String> {
        self.document["theme"][field].as_object().map(|map| map.keys().cloned().collect()).unwrap_or_default()
    }
}

type Options = Vec<(String, CompletionItemKind, Option<String>)>;

fn completion_options<'a>(before: &'a str, analysis: &Analysis) -> Option<(&'a str, Options)> {
    let plain = |names: Vec<String>, kind: CompletionItemKind| names.into_iter().map(|name| (name, kind, None)).collect::<Options>();
    let statics = |names: &[&str], kind: CompletionItemKind| names.iter().map(|name| (name.to_string(), kind, None)).collect::<Options>();
    let partial = trailing_word(before);
    let head = &before[..before.len() - partial.len()];
    let trimmed = head.trim_start();

    if let Some(rest) = trimmed.strip_prefix("::") {
        let rest = rest.trim_start_matches(':').trim_start();
        if rest == "icon " || rest.starts_with("icon ") && rest.trim_end() == "icon" {
            return Some((partial, statics(&ribalta::icons::names().collect::<Vec<_>>(), CompletionItemKind::CONSTANT)));
        }
        if rest.is_empty() {
            let mut options = statics(ribalta::block_names(), CompletionItemKind::KEYWORD);
            options.extend(analysis.named(DefKind::Component).into_iter().map(|name| (name, CompletionItemKind::FUNCTION, Some("componente".to_owned()))));
            return Some((partial, options));
        }
    }
    if head.ends_with('$') {
        let mut options = plain(analysis.named(DefKind::Variable), CompletionItemKind::VARIABLE);
        options.extend(statics(&FIELDS, CompletionItemKind::FIELD));
        return Some((partial, options));
    }
    if trimmed == "@" && head.ends_with('@') {
        return Some((partial, statics(ribalta::directive_names(), CompletionItemKind::KEYWORD)));
    }
    if head.ends_with('#') && head.trim_start() != "#" && head.len() > 1 {
        return Some((partial, plain(analysis.pages().into_iter().map(|page| page.id).collect(), CompletionItemKind::REFERENCE)));
    }
    let open = before.rfind('{')?;
    if before.rfind('}').is_some_and(|close| close > open) {
        return None;
    }
    let inside = &head[open + 1..];
    if inside.ends_with('.') {
        let mut options = plain(analysis.named(DefKind::Style), CompletionItemKind::CLASS);
        options.extend(plain(analysis.theme_keys("colors"), CompletionItemKind::COLOR));
        return Some((partial, options));
    }
    if let Some(key) = inside.strip_suffix('=') {
        let key = trailing_word(key);
        let options = match key {
            "color" | "bg" => plain(analysis.theme_keys("colors"), CompletionItemKind::COLOR),
            "font" => plain(analysis.theme_keys("fonts"), CompletionItemKind::ENUM_MEMBER),
            "next" => plain(analysis.pages().into_iter().map(|page| page.id).collect(), CompletionItemKind::REFERENCE),
            _ => ENUM_VALUES.iter().find(|(name, _)| *name == key).map(|(_, values)| statics(values, CompletionItemKind::ENUM_MEMBER)).unwrap_or_default(),
        };
        return Some((partial, options));
    }
    if inside.is_empty() || inside.ends_with(' ') {
        return Some((partial, statics(&ribalta::attribute_names(), CompletionItemKind::PROPERTY)));
    }
    None
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '-' || character == '_'
}

fn trailing_word(text: &str) -> &str {
    let start = text.char_indices().rev().take_while(|(_, character)| is_word(*character)).last().map_or(text.len(), |(index, _)| index);
    &text[start..]
}

fn prefix_utf16(line: &str, units: usize) -> &str {
    let mut counted = 0;
    for (index, character) in line.char_indices() {
        if counted >= units {
            return &line[..index];
        }
        counted += character.len_utf16();
    }
    line
}

fn word_at(line: &str, character: usize) -> Option<(Option<char>, String)> {
    let cursor = prefix_utf16(line, character).len();
    let start = line[..cursor].char_indices().rev().take_while(|(_, item)| is_word(*item)).last().map_or(cursor, |(index, _)| index);
    let end = line[cursor..].char_indices().find(|(_, item)| !is_word(*item)).map_or(line.len(), |(index, _)| cursor + index);
    if start == end {
        return None;
    }
    let sigil = line[..start].chars().next_back().filter(|item| matches!(item, '#' | '$' | '.'));
    Some((sigil, line[start..end].to_owned()))
}

fn line_range(line: u64) -> Range {
    let line = line.saturating_sub(1) as u32;
    Range { start: Position { line, character: 0 }, end: Position { line, character: 0 } }
}

fn symbol(name: &str, kind: u32, start: u64, end: u64) -> Value {
    let range = json!({ "start": { "line": start.saturating_sub(1), "character": 0 }, "end": { "line": end.saturating_sub(1), "character": 0 } });
    json!({ "name": name, "kind": kind, "range": range, "selectionRange": range })
}

fn convert(diagnostic: &ribalta::Diagnostic, lang: Lang) -> LspDiagnostic {
    let line = diagnostic.line.saturating_sub(1) as u32;
    let start = diagnostic.column.saturating_sub(1) as u32;
    let end = (diagnostic.end_column.saturating_sub(1) as u32).max(start + 1);
    let message = match &diagnostic.hint {
        Some(hint) => format!("{}\n{}: {hint}", diagnostic.message, if lang == Lang::Pt { "dica" } else { "hint" }),
        None => diagnostic.message.clone(),
    };
    LspDiagnostic {
        range: Range { start: Position { line, character: start }, end: Position { line, character: end } },
        severity: Some(match diagnostic.severity {
            ribalta::Severity::Error => DiagnosticSeverity::ERROR,
            ribalta::Severity::Warning => DiagnosticSeverity::WARNING,
        }),
        code: Some(NumberOrString::String(diagnostic.code.to_owned())),
        source: Some("ribalta".to_owned()),
        message,
        ..LspDiagnostic::default()
    }
}

fn same_file(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analysis(source: &str) -> Analysis {
        let compilation = ribalta::compile_localized("", source, ribalta::Locale::from_code("pt"));
        Analysis {
            document: serde_json::to_value(&compilation.document).unwrap(),
            definitions: ribalta::library::definitions(source),
            theme_definitions: Vec::new(),
            theme_uri: None,
            source: source.to_owned(),
        }
    }

    fn labels(before: &str, source: &str) -> Option<(String, Vec<String>)> {
        completion_options(before, &analysis(source)).map(|(partial, options)| (partial.to_owned(), options.into_iter().map(|(label, ..)| label).collect()))
    }

    const SOURCE: &str = "@state score = 0\n@style card {pad=lg}\n@component badge {text}\n:: box {.card}\n@end\n\n--- {#start}\n# Olá\n\n--- {#end}\n# Fim\n";

    #[test]
    fn completes_by_context() {
        let (partial, options) = labels("[Ir]{click=\"go #st", SOURCE).unwrap();
        assert_eq!(partial, "st");
        assert_eq!(options, vec!["start", "end"]);
        assert!(labels("Pontos: $sc", SOURCE).unwrap().1.contains(&"score".to_owned()));
        let (_, blocks) = labels("::: ", SOURCE).unwrap();
        assert!(blocks.contains(&"grid".to_owned()) && blocks.contains(&"badge".to_owned()));
        assert!(labels(":: icon sta", SOURCE).unwrap().1.contains(&"star".to_owned()));
        assert!(labels("@sta", SOURCE).unwrap().1.contains(&"state".to_owned()));
        assert!(labels("Texto {.ca", SOURCE).unwrap().1.contains(&"card".to_owned()));
        assert_eq!(labels("Texto {align=", SOURCE).unwrap().1, vec!["left", "center", "right"]);
        assert!(labels("Texto {color=", SOURCE).unwrap().1.contains(&"accent".to_owned()));
        assert!(labels("Texto {click=\"go #start\" ", SOURCE).unwrap().1.contains(&"on-enter".to_owned()));
        assert!(labels("# Tít", SOURCE).is_none());
        assert!(labels("Texto {a=b} fim", SOURCE).is_none());
    }

    #[test]
    fn finds_words_with_their_sigil() {
        assert_eq!(word_at("go #start now", 5), Some((Some('#'), "start".to_owned())));
        assert_eq!(word_at("Olá $score", 7), Some((Some('$'), "score".to_owned())));
        assert_eq!(word_at(":: badge {x=1}", 4), Some((None, "badge".to_owned())));
        assert_eq!(word_at("   ", 1), None);
    }

    #[test]
    fn converts_utf16_columns_and_hints() {
        let compilation = ribalta::compile_localized("", "# Olá {colr=red}\n", ribalta::Locale::from_code("pt"));
        let diagnostic = convert(&compilation.diagnostics[0], Lang::Pt);
        assert_eq!(diagnostic.range.start, Position { line: 0, character: 7 });
        assert_eq!(diagnostic.code, Some(NumberOrString::String(compilation.diagnostics[0].code.to_owned())));
        assert!(diagnostic.message.contains("dica:"));
    }
}
