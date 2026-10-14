use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::attrs::Attrs;
use crate::block::{Directive, List, Node, PREAMBLE_DIRECTIVES};
use crate::diagnostic::{Diagnostic, Diagnostics, Severity};
use crate::document::{self, ComponentDef, RegionDef};
use crate::inline::{RawInline, parse_inline};
use crate::lower::{BUILTIN_BLOCKS, valid_token};
use crate::text::{self, tr, Locale, Span, Text};
use crate::theme::{BUILTIN_STYLES, THEMES, builtin};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DefKind {
    Setting,
    Variable,
    Color,
    Font,
    Style,
    Component,
    Header,
    Footer,
    Overlay,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct DefRef {
    pub kind: DefKind,
    pub name: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Param {
    pub name: String,
    pub default: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    pub kind: DefKind,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub deps: Vec<DefRef>,
    pub params: Vec<Param>,
    pub slots: Vec<String>,
    pub uses_content: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Extraction {
    pub source: String,
    pub included: Vec<DefRef>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub kind: DefKind,
    pub name: String,
    pub identical: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Resolution {
    Replace { kind: DefKind, name: String },
    Rename { kind: DefKind, name: String, to: String },
    Keep { kind: DefKind, name: String },
}

impl Resolution {
    fn target(&self) -> DefRef {
        match self {
            Resolution::Replace { kind, name } | Resolution::Rename { kind, name, .. } | Resolution::Keep { kind, name } => DefRef { kind: *kind, name: name.clone() },
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Renamed {
    pub kind: DefKind,
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Merged {
    pub source: String,
    pub renamed: Vec<Renamed>,
    pub skipped: Vec<DefRef>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Theme,
    Component,
    Style,
    Palette,
}

const KIND_ORDER: &[DefKind] = &[DefKind::Setting, DefKind::Variable, DefKind::Color, DefKind::Font, DefKind::Style, DefKind::Component, DefKind::Header, DefKind::Footer, DefKind::Overlay];

#[derive(Clone, Debug, PartialEq)]
enum Usage {
    Key(String),
    Class,
    Arg(String, String),
}

type ParamUsages = HashMap<String, Vec<Usage>>;

#[derive(Clone, Debug)]
struct Reference {
    target: DefRef,
    span: Span,
}

struct Entry {
    definition: Definition,
    name_span: Option<Span>,
    block_start: usize,
    references: Vec<Reference>,
}

impl Entry {
    fn key(&self) -> DefRef {
        DefRef { kind: self.definition.kind, name: self.definition.name.clone() }
    }

    fn is(&self, target: &DefRef) -> bool {
        self.definition.kind == target.kind && self.definition.name == target.name
    }
}

enum Body<'p> {
    Plain,
    Variable(Option<&'p Text>),
    Data,
    Color(&'p Text),
    Style(&'p Attrs),
    Component(&'p ComponentDef),
    Region(&'p RegionDef),
}

struct Item<'p> {
    kind: DefKind,
    name: String,
    line: usize,
    name_span: Option<Span>,
    body: Body<'p>,
}

#[derive(Default)]
struct Scope {
    styles: HashSet<String>,
    colors: HashSet<String>,
    fonts: HashSet<String>,
    components: HashSet<String>,
    variables: HashSet<String>,
    usages: HashMap<String, ParamUsages>,
}

impl Scope {
    fn class_kind(&self, name: &str) -> Option<DefKind> {
        if self.styles.contains(name) {
            Some(DefKind::Style)
        } else if BUILTIN_STYLES.iter().any(|(builtin, _)| *builtin == name) {
            None
        } else if self.colors.contains(name) {
            Some(DefKind::Color)
        } else {
            None
        }
    }
}

struct Analysis {
    lines: Vec<String>,
    preamble_len: usize,
    entries: Vec<Entry>,
    usages: HashMap<String, ParamUsages>,
}

impl Analysis {
    fn new(source: &str) -> Self {
        let lines = crate::lines_of(source);
        let (preamble_lines, _) = document::split(&lines);
        let mut diags = Diagnostics::default();
        let preamble = document::parse_preamble(&preamble_lines, &mut diags);
        let texts: Vec<String> = source.lines().map(str::to_string).collect();
        let preamble_len = preamble_lines.len();

        let mut items: Vec<Item<'_>> = Vec::new();
        for (name, setting) in [("theme", &preamble.theme), ("version", &preamble.version), ("page", &preamble.page)] {
            if let Some(value) = setting {
                items.push(Item { kind: DefKind::Setting, name: name.to_string(), line: value.span.line, name_span: None, body: Body::Plain });
            }
        }
        for assignment in preamble.lets.iter().chain(&preamble.states).chain(&preamble.externs) {
            items.push(Item { kind: DefKind::Variable, name: assignment.name.value.clone(), line: assignment.name.span.line, name_span: Some(assignment.name.span), body: Body::Variable(Some(&assignment.value)) });
        }
        for data in &preamble.data {
            items.push(Item { kind: DefKind::Variable, name: data.name.value.clone(), line: data.name.span.line, name_span: Some(data.name.span), body: Body::Data });
        }
        for (token, value) in &preamble.colors {
            items.push(Item { kind: DefKind::Color, name: token.value.clone(), line: token.span.line, name_span: Some(token.span), body: Body::Color(value) });
        }
        for font in &preamble.fonts {
            items.push(Item { kind: DefKind::Font, name: font.name.value.clone(), line: font.name.span.line, name_span: Some(font.name.span), body: Body::Plain });
        }
        for (name, attrs) in &preamble.styles {
            items.push(Item { kind: DefKind::Style, name: name.value.clone(), line: name.span.line, name_span: Some(name.span), body: Body::Style(attrs) });
        }
        for component in &preamble.components {
            items.push(Item { kind: DefKind::Component, name: component.name.value.clone(), line: component.name.span.line, name_span: Some(component.name.span), body: Body::Component(component) });
        }
        for (kind, name, region) in [(DefKind::Header, "header", &preamble.header), (DefKind::Footer, "footer", &preamble.footer), (DefKind::Overlay, "overlay", &preamble.overlay)] {
            if let Some(region) = region {
                items.push(Item { kind, name: name.to_string(), line: region.keyword.line, name_span: None, body: Body::Region(region) });
            }
        }
        items.sort_by_key(|item| item.line);

        let builtin_colors: HashSet<String> = THEMES.iter().filter_map(|name| builtin(name)).flat_map(|theme| theme.colors.into_keys()).collect();
        let builtin_fonts: HashSet<String> = THEMES.iter().filter_map(|name| builtin(name)).flat_map(|theme| theme.fonts.into_keys()).collect();
        let mut scope = Scope::default();
        for item in &items {
            let name = item.name.clone();
            match item.kind {
                DefKind::Color if !builtin_colors.contains(&name) => {
                    scope.colors.insert(name);
                }
                DefKind::Font if !builtin_fonts.contains(&name) => {
                    scope.fonts.insert(name);
                }
                DefKind::Style if !BUILTIN_STYLES.iter().any(|(builtin, _)| *builtin == name) => {
                    scope.styles.insert(name);
                }
                DefKind::Component if !BUILTIN_BLOCKS.contains(&name.as_str()) => {
                    scope.components.insert(name);
                }
                DefKind::Variable => {
                    scope.variables.insert(name);
                }
                _ => {}
            }
        }

        let mut usages = HashMap::new();
        for item in &items {
            if let Body::Component(component) = item.body {
                let mut walker = Walker::new(&scope, &texts, component);
                walker.nodes(&component.body);
                usages.insert(item.name.clone(), walker.usages);
            }
        }
        scope.usages = usages;

        let entries = items
            .iter()
            .map(|item| {
                let mut params = Vec::new();
                let mut slots = Vec::new();
                let mut uses_content = false;
                let end_line = match item.body {
                    Body::Component(_) | Body::Data => block_end(&texts, item.line, preamble_len),
                    Body::Region(region) if region.nodes.is_some() => block_end(&texts, item.line, preamble_len),
                    _ => item.line,
                };
                let references = match item.body {
                    Body::Plain | Body::Data => Vec::new(),
                    Body::Variable(value) => {
                        let mut walker = Walker::plain(&scope, &texts);
                        if let Some(value) = value {
                            walker.variables(value);
                        }
                        walker.references
                    }
                    Body::Color(value) => {
                        let mut walker = Walker::plain(&scope, &texts);
                        walker.variables(value);
                        walker.colors("color", value);
                        walker.references
                    }
                    Body::Style(attrs) => {
                        let mut walker = Walker::plain(&scope, &texts);
                        walker.attrs(attrs);
                        walker.references
                    }
                    Body::Component(component) => {
                        let mut walker = Walker::new(&scope, &texts, component);
                        walker.nodes(&component.body);
                        let own = scope.usages.get(&item.name).cloned().unwrap_or_default();
                        for (param, default) in &component.params {
                            params.push(Param { name: param.value.clone(), default: default.as_ref().map(|value| value.value.clone()) });
                            if let Some(default) = default {
                                let param_usages = own.get(&param.value).cloned().unwrap_or_default();
                                walker.value(&param_usages, default, 0);
                            }
                        }
                        let names: HashSet<&str> = component.params.iter().map(|(param, _)| param.value.as_str()).collect();
                        let mut placeholders = Vec::new();
                        collect_placeholders(&component.body, &mut placeholders);
                        for placeholder in placeholders {
                            if placeholder == "content" {
                                uses_content = true;
                            } else if !names.contains(placeholder.as_str()) && !slots.contains(&placeholder) {
                                slots.push(placeholder);
                            }
                        }
                        walker.references
                    }
                    Body::Region(region) => {
                        let mut walker = Walker::plain(&scope, &texts);
                        if let Some(attrs) = &region.attrs {
                            walker.attrs(attrs);
                        }
                        if let Some(nodes) = &region.nodes {
                            walker.nodes(nodes);
                        }
                        walker.references
                    }
                };
                let mut deps: Vec<DefRef> = Vec::new();
                for reference in &references {
                    let target = &reference.target;
                    if (target.kind == item.kind && target.name == item.name) || deps.contains(target) {
                        continue;
                    }
                    deps.push(target.clone());
                }
                let block_start = comment_start(&texts, item.line);
                Entry {
                    definition: Definition { kind: item.kind, name: item.name.clone(), start_line: item.line, end_line, deps, params, slots, uses_content },
                    name_span: item.name_span,
                    block_start,
                    references,
                }
            })
            .collect();
        let usages = scope.usages;
        Analysis { lines: texts, preamble_len, entries, usages }
    }

    fn has(&self, kind: DefKind, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.definition.kind == kind && entry.definition.name == name)
    }

    fn last(&self, target: &DefRef) -> Option<&Entry> {
        self.entries.iter().rev().find(|entry| entry.is(target))
    }

    fn block(&self, entry: &Entry) -> Vec<String> {
        self.lines[entry.block_start - 1..entry.definition.end_line.min(self.lines.len())].to_vec()
    }

    fn body(&self, entry: &Entry) -> String {
        self.lines[entry.definition.start_line - 1..entry.definition.end_line.min(self.lines.len())]
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render(&self, entries: &[&Entry]) -> Vec<String> {
        let mut out = Vec::new();
        let mut previous: Option<(DefKind, bool)> = None;
        for entry in entries {
            let multiline = entry.definition.end_line > entry.definition.start_line;
            if let Some((kind, was_multiline)) = previous
                && (kind != entry.definition.kind || multiline || was_multiline) {
                    out.push(String::new());
                }
            out.extend(self.block(entry));
            previous = Some((entry.definition.kind, multiline));
        }
        out
    }

    fn rewrite(&mut self, mut edits: Vec<(Span, String, String)>) {
        edits.sort_by_key(|edit| std::cmp::Reverse((edit.0.line, edit.0.from)));
        edits.dedup_by(|a, b| a.0 == b.0);
        for (span, from, to) in edits {
            let Some(line) = span.line.checked_sub(1).and_then(|index| self.lines.get_mut(index)) else {
                continue;
            };
            if line.get(span.from..span.to) == Some(from.as_str()) {
                line.replace_range(span.from..span.to, &to);
            }
        }
    }
}

fn block_end(lines: &[String], start: usize, last: usize) -> usize {
    (start + 1..=last.max(start)).find(|number| lines.get(number - 1).is_some_and(|line| line.trim() == "@end")).unwrap_or(last.max(start))
}

fn comment_start(lines: &[String], line: usize) -> usize {
    let mut start = line;
    while start > 1 && lines[start - 2].trim().starts_with("//") {
        start -= 1;
    }
    start
}

fn collect_placeholders(nodes: &[Node], out: &mut Vec<String>) {
    for node in nodes {
        match node {
            Node::Placeholder { name, .. } => out.push(name.clone()),
            Node::Directive(directive) => collect_placeholders(&directive.children, out),
            Node::Quote { children, .. } => collect_placeholders(children, out),
            _ => {}
        }
    }
}

fn color_tokens(value: &str) -> Vec<(usize, &str)> {
    let bytes = value.as_bytes();
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i <= bytes.len() {
        let boundary = if i == bytes.len() || bytes[i].is_ascii_whitespace() {
            Some(1)
        } else if bytes[i..].starts_with(b"..") {
            Some(2)
        } else {
            None
        };
        match boundary {
            Some(width) => {
                if i > start {
                    tokens.push((start, &value[start..i]));
                }
                i += width;
                start = i;
            }
            None => i += 1,
        }
    }
    tokens
}

struct Walker<'a> {
    scope: &'a Scope,
    lines: &'a [String],
    params: HashSet<String>,
    usages: ParamUsages,
    references: Vec<Reference>,
}

impl<'a> Walker<'a> {
    fn plain(scope: &'a Scope, lines: &'a [String]) -> Self {
        Self { scope, lines, params: HashSet::new(), usages: HashMap::new(), references: Vec::new() }
    }

    fn new(scope: &'a Scope, lines: &'a [String], component: &ComponentDef) -> Self {
        let mut walker = Self::plain(scope, lines);
        walker.params = component.params.iter().map(|(param, _)| param.value.clone()).collect();
        walker
    }

    fn origin(&self, span: Span) -> usize {
        let quoted = span.line.checked_sub(1).and_then(|index| self.lines.get(index)).and_then(|line| line.as_bytes().get(span.from)).is_some_and(|byte| matches!(byte, b'"' | b'\''));
        span.from + usize::from(quoted)
    }

    fn refer(&mut self, kind: DefKind, name: &str, line: usize, from: usize) {
        self.references.push(Reference { target: DefRef { kind, name: name.to_string() }, span: Span::new(line, from, from + name.len()) });
    }

    fn param<'v>(&self, value: &'v str) -> Option<&'v str> {
        let name = value.strip_prefix('$')?;
        self.params.contains(name).then_some(name)
    }

    fn note(&mut self, param: &str, usage: Usage) {
        let list = self.usages.entry(param.to_string()).or_default();
        if !list.contains(&usage) {
            list.push(usage);
        }
    }

    fn variables(&mut self, text: &Text) {
        let origin = self.origin(text.span);
        let bytes = text.value.as_bytes();
        for (index, _) in text.value.match_indices('$') {
            if index > 0 && bytes[index - 1] == b'\\' {
                continue;
            }
            let Some((path, _)) = crate::expr::path_at(&text.value, index + 1) else {
                continue;
            };
            let name = &path[0];
            if !self.params.contains(name) && self.scope.variables.contains(name) {
                self.refer(DefKind::Variable, name, text.span.line, origin + index + 1);
            }
        }
    }

    fn class(&mut self, text: &Text) {
        self.variables(text);
        if let Some(param) = self.param(&text.value) {
            self.note(param, Usage::Class);
        } else if let Some(kind) = self.scope.class_kind(&text.value) {
            let from = self.origin(text.span);
            self.refer(kind, &text.value, text.span.line, from);
        }
    }

    fn pair(&mut self, key: &str, value: &Text) {
        self.variables(value);
        match key {
            "color" | "bg" | "border" | "colors" => self.colors(key, value),
            _ => {
                if let Some(param) = self.param(&value.value) {
                    self.note(param, Usage::Key(key.to_string()));
                } else if key == "font" && self.scope.fonts.contains(&value.value) {
                    let from = self.origin(value.span);
                    self.refer(DefKind::Font, &value.value, value.span.line, from);
                }
            }
        }
    }

    fn colors(&mut self, key: &str, value: &Text) {
        let origin = self.origin(value.span);
        for (offset, token) in color_tokens(&value.value) {
            let base = token.split('/').next().unwrap_or("");
            if let Some(param) = self.param(base) {
                self.note(param, Usage::Key(key.to_string()));
            } else if self.scope.colors.contains(base) {
                self.refer(DefKind::Color, base, value.span.line, origin + offset);
            }
        }
    }

    fn attrs(&mut self, attrs: &Attrs) {
        for class in &attrs.classes {
            self.class(class);
        }
        for (class, condition) in &attrs.conditional {
            self.class(class);
            self.variables(condition);
        }
        for pair in &attrs.pairs {
            if let Some(value) = &pair.value {
                self.pair(&pair.key.value, value);
            }
        }
    }

    fn optional(&mut self, attrs: &Option<Attrs>) {
        if let Some(attrs) = attrs {
            self.attrs(attrs);
        }
    }

    fn arguments(&mut self, component: &str, attrs: &Attrs) {
        for class in &attrs.classes {
            self.class(class);
        }
        for pair in &attrs.pairs {
            let Some(value) = &pair.value else {
                continue;
            };
            self.variables(value);
            if let Some(param) = self.param(&value.value) {
                self.note(param, Usage::Arg(component.to_string(), pair.key.value.clone()));
                continue;
            }
            match self.scope.usages.get(component) {
                Some(params) => {
                    let usages = params.get(&pair.key.value).cloned().unwrap_or_default();
                    self.value(&usages, value, 0);
                }
                None if !self.scope.components.contains(component) => self.pair(&pair.key.value, value),
                None => {}
            }
        }
    }

    fn value(&mut self, usages: &[Usage], value: &Text, depth: usize) {
        if depth > 8 {
            return;
        }
        for usage in usages {
            match usage {
                Usage::Key(key) => self.pair(key, value),
                Usage::Class => self.class(value),
                Usage::Arg(component, param) => {
                    let nested = self.scope.usages.get(component).and_then(|params| params.get(param)).cloned().unwrap_or_default();
                    self.value(&nested, value, depth + 1);
                }
            }
        }
    }

    fn text(&mut self, text: &Text) {
        self.variables(text);
        let mut diags = Diagnostics::default();
        let inlines = parse_inline(&text.value, text.span.line, text.span.from, &mut diags);
        self.inlines(&inlines);
    }

    fn inlines(&mut self, nodes: &[RawInline]) {
        for node in nodes {
            match node {
                RawInline::Strong(children) | RawInline::Emphasis(children) | RawInline::Strike(children) | RawInline::Link { children, .. } => self.inlines(children),
                RawInline::Span { attrs, children } => {
                    self.attrs(attrs);
                    self.inlines(children);
                }
                RawInline::Image { attrs, .. } => self.optional(attrs),
                RawInline::Text(_) | RawInline::Code(_) => {}
            }
        }
    }

    fn nodes(&mut self, nodes: &[Node]) {
        for node in nodes {
            match node {
                Node::Heading { text, attrs, .. } => {
                    self.text(text);
                    self.optional(attrs);
                }
                Node::Paragraph { lines, attrs, .. } => {
                    for line in lines {
                        self.text(line);
                    }
                    self.optional(attrs);
                }
                Node::List(list) => self.list(list),
                Node::Quote { children, .. } => self.nodes(children),
                Node::Code { attrs, .. } | Node::Image { attrs, .. } => self.optional(attrs),
                Node::Table(table) => {
                    for cell in table.header.iter().chain(table.rows.iter().flatten()) {
                        self.text(cell);
                    }
                }
                Node::Directive(directive) => self.directive(directive),
                Node::Placeholder { .. } => {}
            }
        }
    }

    fn list(&mut self, list: &List) {
        for item in &list.items {
            for line in &item.lines {
                self.text(line);
            }
            self.optional(&item.attrs);
            for nested in &item.children {
                self.list(nested);
            }
        }
    }

    fn directive(&mut self, directive: &Directive) {
        let name = directive.name.value.as_str();
        if let Some(value) = &directive.value {
            self.variables(value);
        }
        if name.is_empty() || BUILTIN_BLOCKS.contains(&name) {
            self.optional(&directive.attrs);
        } else {
            if self.scope.components.contains(name) {
                self.refer(DefKind::Component, name, directive.name.span.line, directive.name.span.from);
            }
            if let Some(attrs) = &directive.attrs {
                self.arguments(name, attrs);
            }
        }
        self.nodes(&directive.children);
    }
}

fn order(entries: Vec<&Entry>) -> Vec<&Entry> {
    let mut out = Vec::with_capacity(entries.len());
    for kind in KIND_ORDER {
        let group: Vec<&Entry> = entries.iter().copied().filter(|entry| entry.definition.kind == *kind).collect();
        if matches!(kind, DefKind::Variable | DefKind::Color | DefKind::Style) {
            out.extend(topological(group));
        } else {
            out.extend(group);
        }
    }
    out
}

fn topological(group: Vec<&Entry>) -> Vec<&Entry> {
    let names: HashSet<&str> = group.iter().map(|entry| entry.definition.name.as_str()).collect();
    let mut done = vec![false; group.len()];
    let mut emitted: HashSet<&str> = HashSet::new();
    let mut out = Vec::with_capacity(group.len());
    loop {
        let ready = (0..group.len()).find(|&index| {
            let entry = group[index];
            !done[index]
                && entry
                    .definition
                    .deps
                    .iter()
                    .filter(|dep| dep.kind == entry.definition.kind && dep.name != entry.definition.name && names.contains(dep.name.as_str()))
                    .all(|dep| emitted.contains(dep.name.as_str()))
        });
        let Some(index) = ready.or_else(|| (0..group.len()).find(|&index| !done[index])) else {
            break;
        };
        done[index] = true;
        emitted.insert(group[index].definition.name.as_str());
        out.push(group[index]);
    }
    out
}

fn line_diagnostic(code: &'static str, message: String, hint: Option<String>) -> Diagnostic {
    Diagnostic { severity: Severity::Error, code, message, line: 1, column: 1, end_column: 1, hint }
}

fn kind_label(kind: DefKind) -> (&'static str, &'static str) {
    match kind {
        DefKind::Setting => ("configuração", "setting"),
        DefKind::Variable => ("variável", "variable"),
        DefKind::Color => ("cor", "color"),
        DefKind::Font => ("fonte", "font"),
        DefKind::Style => ("estilo", "style"),
        DefKind::Component => ("componente", "component"),
        DefKind::Header => ("cabeçalho", "header"),
        DefKind::Footer => ("rodapé", "footer"),
        DefKind::Overlay => ("camada livre", "overlay"),
    }
}

pub fn definitions(source: &str) -> Vec<Definition> {
    Analysis::new(source).entries.into_iter().map(|entry| entry.definition).collect()
}

pub fn extract(source: &str, roots: &[DefRef]) -> Extraction {
    let analysis = Analysis::new(source);
    let mut missing = Vec::new();
    let mut queue: Vec<DefRef> = Vec::new();
    for root in roots {
        if analysis.has(root.kind, &root.name) {
            queue.push(root.clone());
        } else {
            missing.push(root.clone());
        }
    }
    let mut wanted: HashSet<DefRef> = HashSet::new();
    while let Some(target) = queue.pop() {
        if !wanted.insert(target.clone()) {
            continue;
        }
        for entry in analysis.entries.iter().filter(|entry| entry.is(&target)) {
            queue.extend(entry.definition.deps.iter().cloned());
        }
    }
    let selected = order(analysis.entries.iter().filter(|entry| wanted.contains(&entry.key())).collect());
    let mut included: Vec<DefRef> = Vec::new();
    for entry in &selected {
        let key = entry.key();
        if !included.contains(&key) {
            included.push(key);
        }
    }
    let lines = analysis.render(&selected);
    let source = if lines.is_empty() { String::new() } else { format!("{}\n", lines.join("\n")) };
    let mut diagnostics = crate::check_prelude(&source);
    for root in missing {
        let (pt, en) = kind_label(root.kind);
        diagnostics.push(line_diagnostic("R532", tr(format!("{pt} não encontrado(a): `{}`", root.name), format!("{en} not found: `{}`", root.name)), None));
    }
    Extraction { source, included, diagnostics }
}

fn find_conflicts(theme: &Analysis, snippet: &Analysis) -> Vec<Conflict> {
    let mut seen: HashSet<DefRef> = HashSet::new();
    let mut out = Vec::new();
    for entry in &snippet.entries {
        let key = entry.key();
        if !seen.insert(key.clone()) {
            continue;
        }
        if let Some(existing) = theme.last(&key) {
            let mine = snippet.last(&key).unwrap_or(entry);
            out.push(Conflict { kind: key.kind, name: key.name, identical: theme.body(existing) == snippet.body(mine) });
        }
    }
    out
}

pub fn conflicts(theme: &str, snippet: &str) -> Vec<Conflict> {
    find_conflicts(&Analysis::new(theme), &Analysis::new(snippet))
}

enum Decision {
    Skip,
    Replace,
    Rename(String),
}

fn rename_allowed(theme: &Analysis, snippet: &Analysis, kind: DefKind, to: &str, taken: &HashSet<DefRef>) -> bool {
    let builtin_names: Vec<String> = match kind {
        DefKind::Color => THEMES.iter().filter_map(|name| builtin(name)).flat_map(|theme| theme.colors.into_keys()).collect(),
        DefKind::Font => THEMES.iter().filter_map(|name| builtin(name)).flat_map(|theme| theme.fonts.into_keys()).collect(),
        DefKind::Style => BUILTIN_STYLES.iter().map(|(name, _)| name.to_string()).collect(),
        DefKind::Component => BUILTIN_BLOCKS.iter().map(|name| name.to_string()).collect(),
        DefKind::Variable => {
            if to == "step" || !to.chars().next().is_some_and(|c| c.is_ascii_lowercase()) || !to.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
                return false;
            }
            Vec::new()
        }
        DefKind::Setting | DefKind::Header | DefKind::Footer | DefKind::Overlay => return false,
    };
    let clashes = |kind: DefKind| theme.has(kind, to) || snippet.has(kind, to);
    valid_token(to)
        && !builtin_names.iter().any(|name| name == to)
        && !clashes(kind)
        && !(kind == DefKind::Color && (clashes(DefKind::Style) || BUILTIN_STYLES.iter().any(|(name, _)| *name == to)))
        && !taken.contains(&DefRef { kind, name: to.to_string() })
}

pub fn merge(theme: &str, snippet: &str, resolutions: &[Resolution], label: Option<&str>) -> Result<Merged, Vec<Conflict>> {
    let theme_analysis = Analysis::new(theme);
    let mut snippet_analysis = Analysis::new(snippet);
    let mut decisions: HashMap<DefRef, Decision> = HashMap::new();
    let mut skipped = Vec::new();
    let mut renamed = Vec::new();
    let mut unresolved = Vec::new();
    let mut taken: HashSet<DefRef> = HashSet::new();
    for conflict in find_conflicts(&theme_analysis, &snippet_analysis) {
        let key = DefRef { kind: conflict.kind, name: conflict.name.clone() };
        if conflict.identical {
            skipped.push(key.clone());
            decisions.insert(key, Decision::Skip);
            continue;
        }
        match resolutions.iter().rev().find(|resolution| resolution.target() == key) {
            Some(Resolution::Replace { .. }) => {
                decisions.insert(key, Decision::Replace);
            }
            Some(Resolution::Keep { .. }) => {
                skipped.push(key.clone());
                decisions.insert(key, Decision::Skip);
            }
            Some(Resolution::Rename { to, .. }) if rename_allowed(&theme_analysis, &snippet_analysis, key.kind, to, &taken) => {
                taken.insert(DefRef { kind: key.kind, name: to.clone() });
                renamed.push(Renamed { kind: key.kind, from: key.name.clone(), to: to.clone() });
                decisions.insert(key, Decision::Rename(to.clone()));
            }
            _ => unresolved.push(conflict),
        }
    }
    if !unresolved.is_empty() {
        return Err(unresolved);
    }

    let mut edits: Vec<(Span, String, String)> = Vec::new();
    for entry in &snippet_analysis.entries {
        if let (Some(Decision::Rename(to)), Some(span)) = (decisions.get(&entry.key()), entry.name_span) {
            edits.push((span, entry.definition.name.clone(), to.clone()));
        }
        for reference in &entry.references {
            if let Some(Decision::Rename(to)) = decisions.get(&reference.target) {
                edits.push((reference.span, reference.target.name.clone(), to.clone()));
            }
        }
    }
    snippet_analysis.rewrite(edits);

    let snippet_entries = &snippet_analysis.entries;
    let appended: Vec<usize> = (0..snippet_entries.len()).filter(|&index| matches!(decisions.get(&snippet_entries[index].key()), None | Some(Decision::Rename(_)))).collect();
    let mut replaced: Vec<(DefRef, &Entry)> = Vec::new();
    for entry in snippet_entries {
        let key = entry.key();
        if matches!(decisions.get(&key), Some(Decision::Replace))
            && !replaced.iter().any(|(existing, _)| *existing == key)
            && let Some(target) = theme_analysis.last(&key) {
                replaced.push((key, target));
            }
    }
    replaced.sort_by_key(|(_, target)| target.block_start);

    let mut moved: HashSet<usize> = HashSet::new();
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    for (key, target) in &replaced {
        let own: Vec<&Entry> = snippet_entries.iter().filter(|entry| entry.is(key)).collect();
        let mut hoisted: Vec<&Entry> = Vec::new();
        if matches!(key.kind, DefKind::Variable | DefKind::Color | DefKind::Style) {
            let mut stack: Vec<String> = own.iter().flat_map(|entry| entry.definition.deps.iter()).filter(|dep| dep.kind == key.kind).map(|dep| dep.name.clone()).collect();
            while let Some(name) = stack.pop() {
                for &index in &appended {
                    let entry = &snippet_entries[index];
                    if entry.definition.kind == key.kind && entry.definition.name == name && moved.insert(index) {
                        hoisted.push(entry);
                        stack.extend(entry.definition.deps.iter().filter(|dep| dep.kind == key.kind).map(|dep| dep.name.clone()));
                    }
                }
            }
        }
        let mut block = Vec::new();
        for entry in topological(hoisted).into_iter().chain(own) {
            block.extend(snippet_analysis.block(entry));
        }
        replacements.push((target.block_start, target.definition.end_line, block));
    }

    let rest: Vec<&Entry> = appended.iter().filter(|index| !moved.contains(index)).map(|&index| &snippet_entries[index]).collect();
    let rest = order(rest);
    if replacements.is_empty() && rest.is_empty() {
        return Ok(Merged { source: theme.to_string(), renamed, skipped });
    }

    let mut out: Vec<String> = Vec::new();
    let mut number = 1;
    while number <= theme_analysis.lines.len() {
        match replacements.iter().find(|(start, _, _)| *start == number) {
            Some((_, end, block)) => {
                out.extend(block.iter().cloned());
                number = (*end).max(number) + 1;
            }
            None => {
                out.push(theme_analysis.lines[number - 1].clone());
                number += 1;
            }
        }
    }
    if !rest.is_empty() {
        while out.last().is_some_and(|line| line.trim().is_empty()) {
            out.pop();
        }
        if !out.is_empty() {
            out.push(String::new());
        }
        if let Some(label) = label.map(str::trim).filter(|label| !label.is_empty()) {
            out.push(format!("// Biblioteca: {label}"));
        }
        out.extend(snippet_analysis.render(&rest));
    }
    Ok(Merged { source: format!("{}\n", out.join("\n")), renamed, skipped })
}

pub fn validate_item(kind: ItemKind, source: &str, entry: Option<&str>, locale: Locale) -> Vec<Diagnostic> {
    text::with_locale(locale, || validate_inner(kind, source, entry, locale))
}

fn item_label(kind: ItemKind) -> (&'static str, &'static str) {
    match kind {
        ItemKind::Theme => ("um tema", "a theme"),
        ItemKind::Component => ("um componente", "a component"),
        ItemKind::Style => ("um estilo", "a style"),
        ItemKind::Palette => ("uma paleta", "a palette"),
    }
}

fn validate_inner(kind: ItemKind, source: &str, entry: Option<&str>, locale: Locale) -> Vec<Diagnostic> {
    let mut found = crate::check_prelude_inner(source);
    let analysis = Analysis::new(source);
    let mut diags = Diagnostics::default();
    let allowed: &[&str] = match kind {
        ItemKind::Theme => PREAMBLE_DIRECTIVES,
        ItemKind::Palette => &["color", "font", "let"],
        ItemKind::Style => &["color", "font", "style", "let", "data"],
        ItemKind::Component => &["component", "style", "color", "font", "let", "data", "state"],
    };
    let (label_pt, label_en) = item_label(kind);
    let inside: HashSet<usize> = analysis.entries.iter().flat_map(|entry| entry.definition.start_line + 1..=entry.definition.end_line).collect();
    for (index, line) in analysis.lines.iter().take(analysis.preamble_len).enumerate() {
        let number = index + 1;
        let Some(rest) = line.trim().strip_prefix('@') else {
            continue;
        };
        if inside.contains(&number) {
            continue;
        }
        let name = rest.split(|c: char| c.is_whitespace() || c == '{').next().unwrap_or("");
        if name == "end" || allowed.contains(&name) || !PREAMBLE_DIRECTIVES.contains(&name) {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let list = allowed.iter().map(|name| format!("@{name}")).collect::<Vec<_>>().join(", ");
        diags
            .error("R530", Span::new(number, indent, indent + 1 + name.len()), tr(format!("`@{name}` não pode ser usado em {label_pt}"), format!("`@{name}` cannot be used in {label_en}")))
            .hint(tr(format!("Aqui só valem: {list}"), format!("Only these are allowed here: {list}")));
    }
    let wanted = match kind {
        ItemKind::Component => Some(DefKind::Component),
        ItemKind::Style => Some(DefKind::Style),
        ItemKind::Theme | ItemKind::Palette => None,
    };
    if let Some(wanted) = wanted {
        let (pt, en) = kind_label(wanted);
        let available: Vec<&str> = analysis.entries.iter().filter(|entry| entry.definition.kind == wanted).map(|entry| entry.definition.name.as_str()).collect();
        let hint = if available.is_empty() {
            tr(format!("Defina pelo menos um {pt} no código"), format!("Define at least one {en} in the source"))
        } else {
            tr(format!("Disponíveis: {}", available.join(", ")), format!("Available: {}", available.join(", ")))
        };
        match entry.map(str::trim).filter(|name| !name.is_empty()) {
            None => {
                diags.error("R531", Span::new(1, 0, 0), tr(format!("escolha qual {pt} é o principal"), format!("choose which {en} is the main one"))).hint(hint);
            }
            Some(name) => match analysis.entries.iter().rev().find(|entry| entry.definition.kind == wanted && entry.definition.name == name) {
                None => {
                    diags.error("R532", Span::new(1, 0, 0), tr(format!("{pt} não encontrado(a): `{name}`"), format!("{en} not found: `{name}`"))).hint(hint);
                }
                Some(main) if wanted == DefKind::Component => {
                    let usage = sample_usage(&analysis, main);
                    let span = main.name_span.unwrap_or_else(|| Span::new(main.definition.start_line, 0, 0));
                    for diagnostic in crate::compile_localized(source, &usage, locale).diagnostics.into_iter().filter(|diagnostic| diagnostic.severity == Severity::Error) {
                        let pending = diags.error(diagnostic.code, span, diagnostic.message);
                        if let Some(hint) = diagnostic.hint {
                            pending.hint(hint);
                        }
                    }
                }
                Some(_) => {}
            },
        }
    }
    found.extend(diags.finish(source));
    found.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.column));
    found.dedup();
    found
}

fn sample_usage(analysis: &Analysis, entry: &Entry) -> String {
    let name = &entry.definition.name;
    let usages = analysis.usages.get(name).cloned().unwrap_or_default();
    let args: Vec<String> = entry
        .definition
        .params
        .iter()
        .filter(|param| param.default.is_none())
        .map(|param| format!("{}=\"{}\"", param.name, sample_value(analysis, usages.get(&param.name).map(Vec::as_slice).unwrap_or(&[]), 0)))
        .collect();
    let attrs = if args.is_empty() { String::new() } else { format!(" {{{}}}", args.join(" ")) };
    if entry.definition.uses_content || !entry.definition.slots.is_empty() { format!("::: {name}{attrs}\n:::\n") } else { format!(":: {name}{attrs}\n") }
}

fn sample_value(analysis: &Analysis, usages: &[Usage], depth: usize) -> &'static str {
    for usage in usages {
        let value = match usage {
            Usage::Class => Some("center"),
            Usage::Key(key) => Some(match key.as_str() {
                "color" | "bg" | "border" | "colors" => "accent",
                "font" => "body",
                "image" => "https://example.com/image.png",
                "size" | "pad" | "gap" | "radius" | "shadow" => "md",
                "weight" => "bold",
                "align" | "valign" => "center",
                "w" | "h" => "auto",
                "x" | "y" => "0",
                "opacity" | "step" => "1",
                "leading" | "tracking" => "normal",
                "italic" | "uppercase" | "legend" | "stacked" | "values" | "wrap" => "true",
                "type" => "bar",
                "cols" => "2",
                "fit" => "cover",
                _ => "Texto",
            }),
            Usage::Arg(component, param) if depth < 8 => {
                let nested = analysis.usages.get(component).and_then(|params| params.get(param)).map(Vec::as_slice).unwrap_or(&[]);
                (!nested.is_empty()).then(|| sample_value(analysis, nested, depth + 1))
            }
            Usage::Arg(..) => None,
        };
        if let Some(value) = value {
            return value;
        }
    }
    "Texto"
}
