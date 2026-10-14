use crate::attrs::{Attrs, attached_attrs, parse_attrs};
use crate::block::{Node, PREAMBLE_DIRECTIVES, Table, parse_blocks};
use crate::diagnostic::Diagnostics;
use crate::text::{tr, Line, Span, Text, suggest};

pub struct PageSource<'a> {
    pub separator: Option<Line<'a>>,
    pub lines: Vec<Line<'a>>,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub name: Text,
    pub params: Vec<(Text, Option<Text>)>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct FontDef {
    pub name: Text,
    pub family: Text,
    pub weights: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct RegionDef {
    pub keyword: Span,
    pub attrs: Option<Attrs>,
    pub nodes: Option<Vec<Node>>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: Text,
    pub value: Text,
}

#[derive(Debug, Clone)]
pub struct DataDef {
    pub name: Text,
    pub table: Option<Table>,
}

#[derive(Debug, Default)]
pub struct Preamble {
    pub title: Option<String>,
    pub author: Option<String>,
    pub theme: Option<Text>,
    pub version: Option<Text>,
    pub page: Option<Text>,
    pub colors: Vec<(Text, Text)>,
    pub fonts: Vec<FontDef>,
    pub styles: Vec<(Text, Attrs)>,
    pub components: Vec<ComponentDef>,
    pub header: Option<RegionDef>,
    pub footer: Option<RegionDef>,
    pub overlay: Option<RegionDef>,
    pub lets: Vec<Assignment>,
    pub states: Vec<Assignment>,
    pub externs: Vec<Assignment>,
    pub data: Vec<DataDef>,
    pub transition: Option<Text>,
    pub mode: Option<Text>,
    pub persist: Vec<Text>,
    pub scorm: Option<Text>,
}

pub fn is_separator(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("---") else {
        return false;
    };
    let rest = rest.trim_start_matches('-').trim();
    rest.is_empty() || (rest.starts_with('{') && rest.ends_with('}'))
}

fn opens_block(trimmed: &str) -> bool {
    let word = trimmed.split(|c: char| c.is_whitespace() || c == '{').next().unwrap_or("");
    match word {
        "@component" | "@data" => true,
        "@header" | "@footer" | "@overlay" => trimmed[word.len()..].trim() != "none",
        _ => false,
    }
}

fn body_until_end<'l, 'a>(lines: &'l [Line<'a>], start: usize) -> (&'l [Line<'a>], usize, bool) {
    let mut end = start;
    while end < lines.len() && lines[end].trimmed() != "@end" {
        end += 1;
    }
    let closed = end < lines.len();
    (&lines[start..end], if closed { end + 1 } else { end }, closed)
}

pub fn split<'a>(lines: &[Line<'a>]) -> (Vec<Line<'a>>, Vec<PageSource<'a>>) {
    let mut preamble = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trimmed();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            preamble.push(lines[i]);
            i += 1;
        } else if opens_block(trimmed) {
            while i < lines.len() {
                preamble.push(lines[i]);
                i += 1;
                if lines[i - 1].trimmed() == "@end" {
                    break;
                }
            }
        } else if trimmed.starts_with('@') {
            preamble.push(lines[i]);
            i += 1;
        } else {
            break;
        }
    }
    let mut pages: Vec<PageSource<'a>> = Vec::new();
    let mut in_fence = false;
    for line in &lines[i..] {
        let trimmed = line.trimmed();
        if !in_fence && is_separator(trimmed) {
            pages.push(PageSource { separator: Some(*line), lines: Vec::new(), start_line: line.number, end_line: line.number });
            continue;
        }
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
        }
        if pages.is_empty() {
            pages.push(PageSource { separator: None, lines: Vec::new(), start_line: line.number, end_line: line.number });
        }
        let current = pages.last_mut().expect("ensured above");
        current.lines.push(*line);
        current.end_line = line.number;
    }
    (preamble, pages)
}

pub fn separator_attrs(line: Line<'_>, diags: &mut Diagnostics) -> Option<Attrs> {
    let (open, close) = attached_attrs(line.text)?;
    Some(parse_attrs(&line.text[open + 1..close], line.number, line.offset + open + 1, diags))
}

pub fn parse_preamble(lines: &[Line<'_>], diags: &mut Diagnostics) -> Preamble {
    let mut preamble = Preamble::default();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        i += 1;
        let trimmed = line.trimmed();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let indent = line.indent();
        let name_end = trimmed[1..].find(|c: char| c.is_whitespace() || c == '{').map_or(trimmed.len(), |index| index + 1);
        let name = &trimmed[1..name_end];
        let rest_start = indent + name_end;
        let rest = line.slice(rest_start);
        let value_start = rest.indent();
        let value = rest.text_at(value_start, rest.text.trim_end().len().max(value_start));
        let name_span = line.span(indent, indent + name_end);
        let require_value = |diags: &mut Diagnostics, value: &Text| {
            if value.value.is_empty() {
                diags.error("R501", name_span, tr(format!("`@{name}` precisa de um valor"), format!("`@{name}` needs a value")));
                false
            } else {
                true
            }
        };
        match name {
            "title" => {
                if require_value(diags, &value) {
                    preamble.title = Some(value.value);
                }
            }
            "author" => {
                if require_value(diags, &value) {
                    preamble.author = Some(value.value);
                }
            }
            "theme" => {
                if require_value(diags, &value) {
                    preamble.theme = Some(value);
                }
            }
            "version" => {
                if require_value(diags, &value) {
                    preamble.version = Some(value);
                }
            }
            "page" => {
                if require_value(diags, &value) {
                    preamble.page = Some(value);
                }
            }
            "color" => match two_words(&value) {
                Some((token, color)) => preamble.colors.push((token, color)),
                None => {
                    diags.error("R502", value.span, tr("`@color` espera um nome e um valor", "`@color` expects a name and a value")).hint(tr("Exemplo: @color marca #ff6b4a", "Example: @color brand #ff6b4a"));
                }
            },
            "font" => {
                if let Some(font) = font_def(&value, diags) {
                    preamble.fonts.push(font);
                }
            }
            "style" => match (two_words(&value), attached_attrs(rest.text)) {
                (Some((style_name, _)), Some((open, close))) => {
                    let attrs = parse_attrs(&rest.text[open + 1..close], rest.number, rest.offset + open + 1, diags);
                    preamble.styles.push((style_name, attrs));
                }
                _ => {
                    diags.error("R503", value.span, tr("`@style` espera um nome e atributos", "`@style` expects a name and attributes")).hint(tr("Exemplo: @style destaque {color=coral weight=bold}", "Example: @style highlight {color=coral weight=bold}"));
                }
            },
            "component" => {
                let (body_lines, next, closed) = body_until_end(lines, i);
                i = next;
                if !closed {
                    unclosed(diags, name_span, name);
                }
                if let Some(component) = component_def(rest, body_lines, diags) {
                    preamble.components.push(component);
                }
            }
            "header" | "footer" | "overlay" => {
                let region = if value.value == "none" {
                    RegionDef { keyword: name_span, attrs: None, nodes: None }
                } else {
                    let attrs = attached_attrs(rest.text).map(|(open, close)| parse_attrs(&rest.text[open + 1..close], rest.number, rest.offset + open + 1, diags));
                    let stray = attached_attrs(rest.text).map_or(value.value.as_str(), |(open, _)| rest.text[..open].trim());
                    if !stray.is_empty() {
                        diags
                            .warning("R525", value.span, tr(format!("o texto `{stray}` depois de `@{name}` será ignorado"), format!("the text `{stray}` after `@{name}` will be ignored")))
                            .hint(tr(format!("Escreva o conteúdo nas linhas seguintes, até `@end`. Para desligar, use `@{name} none`"), format!("Write the content on the following lines, up to `@end`. To turn it off, use `@{name} none`")));
                    }
                    let (body_lines, next, closed) = body_until_end(lines, i);
                    i = next;
                    if !closed {
                        unclosed(diags, name_span, name);
                    }
                    RegionDef { keyword: name_span, attrs, nodes: Some(parse_blocks(body_lines, diags)) }
                };
                let slot = match name {
                    "header" => &mut preamble.header,
                    "footer" => &mut preamble.footer,
                    _ => &mut preamble.overlay,
                };
                if slot.is_some() {
                    diags.error("R521", name_span, tr(format!("`@{name}` foi definido mais de uma vez"), format!("`@{name}` was defined more than once"))).hint(tr("Junte todo o conteúdo em um único bloco", "Put all the content in a single block"));
                } else {
                    *slot = Some(region);
                }
            }
            "let" | "state" | "extern" => {
                if let Some(assignment) = assignment(name, &value, diags) {
                    match name {
                        "let" => preamble.lets.push(assignment),
                        "state" => preamble.states.push(assignment),
                        _ => preamble.externs.push(assignment),
                    }
                }
            }
            "transition" => {
                if require_value(diags, &value) {
                    preamble.transition = Some(value);
                }
            }
            "mode" => {
                if require_value(diags, &value) {
                    preamble.mode = Some(value);
                }
            }
            "scorm" => {
                if require_value(diags, &value) {
                    preamble.scorm = Some(value);
                }
            }
            "persist" => {
                if require_value(diags, &value) {
                    let mut offset = 0;
                    for word in value.value.split(|c: char| c.is_whitespace() || c == ',') {
                        let start = value.value[offset..].find(word).map_or(offset, |found| offset + found);
                        offset = start + word.len();
                        if !word.is_empty() {
                            preamble.persist.push(Text::new(word.trim_start_matches('$'), value.span.line, value.span.from + start));
                        }
                    }
                }
            }
            "data" => {
                let (body_lines, next, closed) = body_until_end(lines, i);
                i = next;
                if !closed {
                    unclosed(diags, name_span, name);
                }
                if value.value.is_empty() {
                    diags.error("R609", name_span, tr("`@data` precisa de um nome", "`@data` needs a name")).hint(tr("Exemplo: @data equipe, seguido de uma tabela e @end", "Example: @data team, followed by a table and @end"));
                    continue;
                }
                let nodes = parse_blocks(body_lines, diags);
                let mut tables = nodes.into_iter().filter_map(|node| match node {
                    Node::Table(table) => Some(table),
                    _ => None,
                });
                let table = tables.next();
                if table.is_none() {
                    diags
                        .error("R609", value.span, tr(format!("`@data {}` precisa de uma tabela", value.value), format!("`@data {}` needs a table", value.value)))
                        .hint(tr("Escreva uma tabela Markdown: a primeira linha tem os nomes dos campos", "Write a Markdown table: the first row holds the field names"));
                }
                preamble.data.push(DataDef { name: value, table });
            }
            "end" => {
                diags
                    .error("R505", name_span, tr("`@end` sem um bloco aberto", "`@end` without an open block"))
                    .hint(tr("`@end` fecha um `@component`, `@data`, `@header`, `@footer` ou `@overlay`", "`@end` closes a `@component`, `@data`, `@header`, `@footer` or `@overlay`"));
            }
            _ => {
                let pending = diags.error("R506", name_span, tr(format!("definição desconhecida: `@{name}`"), format!("unknown definition: `@{name}`")));
                match suggest(name, PREAMBLE_DIRECTIVES.iter().copied()) {
                    Some(found) => pending.hint(tr(format!("Você quis dizer `@{found}`?"), format!("Did you mean `@{found}`?"))),
                    None => pending.hint(tr(format!("Definições disponíveis: {}", PREAMBLE_DIRECTIVES.iter().map(|n| format!("@{n}")).collect::<Vec<_>>().join(", ")), format!("Available definitions: {}", PREAMBLE_DIRECTIVES.iter().map(|n| format!("@{n}")).collect::<Vec<_>>().join(", ")))),
                };
            }
        }
    }
    preamble
}

fn unclosed(diags: &mut Diagnostics, span: Span, name: &str) {
    diags
        .error("R504", span, tr(format!("`@{name}` sem `@end`"), format!("`@{name}` without `@end`")))
        .hint(tr(format!("Termine o bloco `@{name}` com uma linha `@end`"), format!("End the `@{name}` block with an `@end` line")));
}

pub fn assignment(keyword: &str, value: &Text, diags: &mut Diagnostics) -> Option<Assignment> {
    let text = value.value.as_str();
    let Some(equals) = text.find('=').filter(|index| !text[index + 1..].starts_with('=')) else {
        diags
            .error("R609", value.span, tr(format!("`@{keyword}` espera um nome, `=` e um valor"), format!("`@{keyword}` expects a name, `=` and a value")))
            .hint(tr(format!("Exemplo: @{keyword} destaque = coral"), format!("Example: @{keyword} highlight = coral")));
        return None;
    };
    let raw_name = &text[..equals];
    let name_start = raw_name.len() - raw_name.trim_start().len();
    let name = Text::new(raw_name.trim(), value.span.line, value.span.from + name_start);
    let raw_value = &text[equals + 1..];
    let value_start = equals + 1 + (raw_value.len() - raw_value.trim_start().len());
    let expression = Text::new(raw_value.trim(), value.span.line, value.span.from + value_start);
    if name.value.is_empty() || expression.value.is_empty() {
        diags
            .error("R609", value.span, tr(format!("`@{keyword}` espera um nome, `=` e um valor"), format!("`@{keyword}` expects a name, `=` and a value")))
            .hint(tr(format!("Exemplo: @{keyword} destaque = coral"), format!("Example: @{keyword} highlight = coral")));
        return None;
    }
    Some(Assignment { name, value: expression })
}

fn two_words(value: &Text) -> Option<(Text, Text)> {
    let text = value.value.as_str();
    let first_end = text.find(|c: char| c.is_whitespace() || c == '{').unwrap_or(text.len());
    let first = Text::new(&text[..first_end], value.span.line, value.span.from);
    let rest = text[first_end..].trim_start();
    let rest_start = text.len() - rest.len();
    let second = Text::new(rest.trim_end(), value.span.line, value.span.from + rest_start);
    (!first.value.is_empty()).then_some((first, second))
}

fn font_def(value: &Text, diags: &mut Diagnostics) -> Option<FontDef> {
    let (name, rest) = two_words(value)?;
    let text = rest.value.as_str();
    let (family, weights_text) = match text.strip_prefix('"') {
        Some(after) => match after.find('"') {
            Some(close) => (&after[..close], &after[close + 1..]),
            None => {
                diags.error("R507", rest.span, tr("nome da fonte sem aspas de fechamento", "font name without closing quotes"));
                return None;
            }
        },
        None => (text, ""),
    };
    let family_text = Text::new(family, rest.span.line, rest.span.from + 1);
    if family.is_empty() || !family.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ') {
        diags
            .error("R508", rest.span, tr(format!("nome de fonte inválido: `{family}`"), format!("invalid font name: `{family}`")))
            .hint(tr("Use o nome de uma família do Google Fonts entre aspas, como @font titulo \"Bebas Neue\" 400", "Use a Google Fonts family name in quotes, such as @font title \"Bebas Neue\" 400"));
        return None;
    }
    let mut weights = Vec::new();
    for part in weights_text.split_whitespace() {
        match part.parse::<u16>() {
            Ok(weight) if (100..=900).contains(&weight) && weight % 100 == 0 => weights.push(weight),
            _ => {
                diags.error("R509", rest.span, tr(format!("peso de fonte inválido: `{part}`"), format!("invalid font weight: `{part}`"))).hint(tr("Pesos vão de 100 a 900, de 100 em 100", "Weights go from 100 to 900, in steps of 100"));
            }
        }
    }
    weights.sort_unstable();
    weights.dedup();
    Some(FontDef { name, family: family_text, weights })
}

fn component_def(header: Line<'_>, body: &[Line<'_>], diags: &mut Diagnostics) -> Option<ComponentDef> {
    let text = header.text;
    let start = header.indent();
    let name_end = text[start..].find(|c: char| c.is_whitespace() || c == '{').map_or(text.len(), |index| start + index);
    if name_end == start {
        diags.error("R510", header.full_span(), tr("`@component` precisa de um nome", "`@component` needs a name")).hint(tr("Exemplo: @component destaque {titulo cor=accent}", "Example: @component highlight {title color=accent}"));
        return None;
    }
    let name = header.text_at(start, name_end);
    if !name.value.chars().next().is_some_and(|c| c.is_ascii_lowercase()) || !name.value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        diags.error("R511", name.span, tr(format!("nome de componente inválido: `{}`", name.value), format!("invalid component name: `{}`", name.value))).hint(tr("Use letras minúsculas, números e hífens", "Use lowercase letters, numbers and hyphens"));
        return None;
    }
    let mut params = Vec::new();
    if let Some((open, close)) = attached_attrs(text) {
        let attrs = parse_attrs(&text[open + 1..close], header.number, header.offset + open + 1, diags);
        if !attrs.classes.is_empty() || attrs.id.is_some() {
            diags.error("R512", attrs.span, tr("parâmetros de componente não aceitam .estilo nem #id", "component parameters do not accept .style or #id"));
        }
        for pair in attrs.pairs {
            params.push((pair.key, pair.value));
        }
    }
    let nodes = parse_blocks(body, diags);
    Some(ComponentDef { name, params, body: nodes })
}
