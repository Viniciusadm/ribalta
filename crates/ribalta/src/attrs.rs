use crate::diagnostic::Diagnostics;
use crate::text::{tr, Span, Text};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Attrs {
    pub classes: Vec<Text>,
    pub id: Option<Text>,
    pub pairs: Vec<Pair>,
    pub conditional: Vec<(Text, Text)>,
    pub span: Span,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pair {
    pub key: Text,
    pub value: Option<Text>,
}

pub fn parse_attrs(inner: &str, line: usize, offset: usize, diags: &mut Diagnostics) -> Attrs {
    let mut attrs = Attrs {
        span: Span::new(line, offset.saturating_sub(1), offset + inner.len() + 1),
        raw: inner.trim().to_string(),
        ..Attrs::default()
    };
    let chars: Vec<(usize, char)> = inner.char_indices().collect();
    let byte_at = |index: usize| chars.get(index).map_or(inner.len(), |(byte, _)| *byte);
    let mut i = 0;
    while i < chars.len() {
        let (_, c) = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if c == '.' || c == '#' {
            i += 1;
            while i < chars.len() && is_name_char(chars[i].1) {
                i += 1;
            }
            let name = &inner[byte_at(start + 1)..byte_at(i)];
            let text = Text::new(name, line, offset + byte_at(start + 1));
            if name.is_empty() {
                diags
                    .error("R201", Span::new(line, offset + byte_at(start), offset + byte_at(i)), tr(format!("`{c}` precisa ser seguido de um nome"), format!("`{c}` must be followed by a name")))
                    .hint(if c == '.' { tr("Exemplo: {.destaque}", "Example: {.highlight}") } else { tr("Exemplo: {#abertura}", "Example: {#opening}") });
            } else if c == '.' && i < chars.len() && chars[i].1 == '=' {
                i += 1;
                let value = parse_value(inner, &chars, &mut i, line, offset, diags);
                if value.value.is_empty() {
                    diags.error("R203", text.span, tr(format!("a condição de `.{}` está vazia", text.value), format!("the condition of `.{}` is empty", text.value)));
                } else {
                    attrs.conditional.push((text, value));
                }
            } else if c == '.' {
                attrs.classes.push(text);
            } else {
                attrs.id = Some(text);
            }
            continue;
        }
        if is_name_start(c) {
            while i < chars.len() && is_name_char(chars[i].1) {
                i += 1;
            }
            let key = Text::new(&inner[byte_at(start)..byte_at(i)], line, offset + byte_at(start));
            if i < chars.len() && chars[i].1 == '=' {
                i += 1;
                let value = parse_value(inner, &chars, &mut i, line, offset, diags);
                if value.value.is_empty() {
                    diags.error("R203", key.span, tr(format!("o atributo `{}` está sem valor depois de `=`", key.value), format!("attribute `{}` has no value after `=`", key.value)));
                }
                attrs.pairs.push(Pair { key, value: Some(value) });
            } else {
                attrs.pairs.push(Pair { key, value: None });
            }
            continue;
        }
        while i < chars.len() && !chars[i].1.is_whitespace() {
            i += 1;
        }
        diags
            .error(
                "R204",
                Span::new(line, offset + byte_at(start), offset + byte_at(i)),
                tr(format!("trecho inválido em atributos: `{}`", &inner[byte_at(start)..byte_at(i)]), format!("invalid text in attributes: `{}`", &inner[byte_at(start)..byte_at(i)])),
            )
            .hint(tr("Atributos têm a forma {.estilo #id chave=valor chave=\"valor com espaços\"}", "Attributes look like {.style #id key=value key=\"value with spaces\"}"));
    }
    attrs
}

fn parse_value(inner: &str, chars: &[(usize, char)], i: &mut usize, line: usize, offset: usize, diags: &mut Diagnostics) -> Text {
    let byte_at = |index: usize| chars.get(index).map_or(inner.len(), |(byte, _)| *byte);
    let value_start = *i;
    if *i < chars.len() && (chars[*i].1 == '"' || chars[*i].1 == '\'') {
        let quote = chars[*i].1;
        *i += 1;
        let mut value = String::new();
        let mut closed = false;
        while *i < chars.len() {
            let current = chars[*i].1;
            if current == '\\' && *i + 1 < chars.len() {
                value.push(chars[*i + 1].1);
                *i += 2;
                continue;
            }
            *i += 1;
            if current == quote {
                closed = true;
                break;
            }
            value.push(current);
        }
        if !closed {
            diags
                .error("R202", Span::new(line, offset + byte_at(value_start), offset + inner.len()), tr("aspas não fechadas no valor do atributo", "unclosed quotes in attribute value"))
                .hint(tr(format!("Feche o valor com {quote}"), format!("Close the value with {quote}")));
        }
        Text::new(value, line, offset + byte_at(value_start) + 1)
    } else {
        while *i < chars.len() && !chars[*i].1.is_whitespace() {
            *i += 1;
        }
        Text::new(&inner[byte_at(value_start)..byte_at(*i)], line, offset + byte_at(value_start))
    }
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '$'
}

pub fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '$'
}

pub fn trailing_attrs(text: &str) -> Option<(usize, usize)> {
    find_attrs(text, true)
}

pub fn attached_attrs(text: &str) -> Option<(usize, usize)> {
    find_attrs(text, false)
}

fn find_attrs(text: &str, require_space: bool) -> Option<(usize, usize)> {
    let end = text.trim_end().len();
    if !text[..end].ends_with('}') {
        return None;
    }
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < end {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'{' if !require_space || i == 0 || bytes[i - 1] == b' ' || bytes[i - 1] == b'\t' => {
                if let Some(close) = matching_brace(bytes, i)
                    && close + 1 == end {
                        return Some((i, close));
                    }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

pub fn matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;
    let mut i = open + 1;
    while i < bytes.len() {
        let byte = bytes[i];
        match quote {
            Some(q) => {
                if byte == b'\\' {
                    i += 1;
                } else if byte == q {
                    quote = None;
                }
            }
            None => match byte {
                b'"' | b'\'' => quote = Some(byte),
                b'{' => return None,
                b'}' => return Some(i),
                _ => {}
            },
        }
        i += 1;
    }
    None
}
