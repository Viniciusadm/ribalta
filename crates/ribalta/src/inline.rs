use crate::attrs::{Attrs, matching_brace, parse_attrs};
use crate::diagnostic::Diagnostics;
use crate::text::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum RawInline {
    Text(String),
    Strong(Vec<RawInline>),
    Emphasis(Vec<RawInline>),
    Strike(Vec<RawInline>),
    Code(String),
    Link { href: String, span: Span, children: Vec<RawInline> },
    Span { attrs: Attrs, children: Vec<RawInline> },
    Image { alt: String, src: String, span: Span, attrs: Option<Attrs> },
}

pub fn parse_inline(text: &str, line: usize, offset: usize, diags: &mut Diagnostics) -> Vec<RawInline> {
    let mut parser = InlineParser { src: text, line, offset, diags };
    parser.parse(0, text.len())
}

struct InlineParser<'a, 'd> {
    src: &'a str,
    line: usize,
    offset: usize,
    diags: &'d mut Diagnostics,
}

impl InlineParser<'_, '_> {
    fn parse(&mut self, start: usize, end: usize) -> Vec<RawInline> {
        let mut out = Vec::new();
        let mut buffer = String::new();
        let mut i = start;
        while i < end {
            let rest = &self.src[i..end];
            let c = rest.chars().next().expect("non-empty");
            match c {
                '\\' => {
                    match rest[1..].chars().next() {
                        Some(next) if next.is_ascii_punctuation() => {
                            buffer.push(next);
                            i += 1 + next.len_utf8();
                        }
                        _ => {
                            buffer.push('\\');
                            i += 1;
                        }
                    }
                    continue;
                }
                '`' => {
                    if let Some(close) = self.find(i + 1, end, "`") {
                        flush(&mut buffer, &mut out);
                        out.push(RawInline::Code(self.src[i + 1..close].to_string()));
                        i = close + 1;
                        continue;
                    }
                }
                '*' if rest.starts_with("**") => {
                    if let Some(next) = self.delimited(i, end, "**", &mut buffer, &mut out, RawInline::Strong) {
                        i = next;
                        continue;
                    }
                }
                '*' => {
                    if let Some(next) = self.delimited(i, end, "*", &mut buffer, &mut out, RawInline::Emphasis) {
                        i = next;
                        continue;
                    }
                }
                '~' if rest.starts_with("~~") => {
                    if let Some(next) = self.delimited(i, end, "~~", &mut buffer, &mut out, RawInline::Strike) {
                        i = next;
                        continue;
                    }
                }
                '!' if rest.starts_with("![") => {
                    if let Some((node, next)) = self.image(i, end) {
                        flush(&mut buffer, &mut out);
                        out.push(node);
                        i = next;
                        continue;
                    }
                }
                '[' => {
                    if let Some((node, next)) = self.bracket(i, end) {
                        flush(&mut buffer, &mut out);
                        out.push(node);
                        i = next;
                        continue;
                    }
                }
                _ => {}
            }
            buffer.push(c);
            i += c.len_utf8();
        }
        flush(&mut buffer, &mut out);
        out
    }

    fn delimited(
        &mut self,
        i: usize,
        end: usize,
        delimiter: &str,
        buffer: &mut String,
        out: &mut Vec<RawInline>,
        build: fn(Vec<RawInline>) -> RawInline,
    ) -> Option<usize> {
        let inner_start = i + delimiter.len();
        let first = self.src[inner_start..end].chars().next()?;
        if first.is_whitespace() {
            return None;
        }
        let close = if delimiter == "*" { self.find_single_star(inner_start, end)? } else { self.find(inner_start, end, delimiter)? };
        if close == inner_start || self.src[..close].ends_with(char::is_whitespace) {
            return None;
        }
        flush(buffer, out);
        let children = self.parse(inner_start, close);
        out.push(build(children));
        Some(close + delimiter.len())
    }

    fn find(&self, from: usize, end: usize, pattern: &str) -> Option<usize> {
        let bytes = self.src.as_bytes();
        let mut i = from;
        while i < end {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i..end].starts_with(pattern.as_bytes()) {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn find_single_star(&self, from: usize, end: usize) -> Option<usize> {
        let bytes = self.src.as_bytes();
        let mut i = from;
        while i < end {
            match bytes[i] {
                b'\\' => i += 2,
                b'*' => {
                    let mut run = i;
                    while run < end && bytes[run] == b'*' {
                        run += 1;
                    }
                    if run - i == 1 {
                        return Some(i);
                    }
                    i = run;
                }
                _ => i += 1,
            }
        }
        None
    }

    fn close_bracket(&self, open: usize, end: usize) -> Option<usize> {
        let bytes = self.src.as_bytes();
        let mut depth = 0;
        let mut i = open;
        while i < end {
            match bytes[i] {
                b'\\' => i += 1,
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        None
    }

    fn close_paren(&self, open: usize, end: usize) -> Option<usize> {
        let bytes = self.src.as_bytes();
        let mut depth = 0;
        for (i, byte) in bytes.iter().enumerate().take(end).skip(open) {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn attached_attrs(&mut self, at: usize, end: usize) -> Option<(Attrs, usize)> {
        if self.src.as_bytes().get(at) != Some(&b'{') {
            return None;
        }
        let close = matching_brace(&self.src.as_bytes()[..end], at)?;
        let attrs = parse_attrs(&self.src[at + 1..close], self.line, self.offset + at + 1, self.diags);
        Some((attrs, close + 1))
    }

    fn image(&mut self, i: usize, end: usize) -> Option<(RawInline, usize)> {
        let close = self.close_bracket(i + 1, end)?;
        if self.src.as_bytes().get(close + 1) != Some(&b'(') {
            return None;
        }
        let paren = self.close_paren(close + 1, end)?;
        let alt = self.src[i + 2..close].to_string();
        let src = self.src[close + 2..paren].trim().to_string();
        let span = Span::new(self.line, self.offset + close + 2, self.offset + paren);
        let (attrs, next) = match self.attached_attrs(paren + 1, end) {
            Some((attrs, next)) => (Some(attrs), next),
            None => (None, paren + 1),
        };
        Some((RawInline::Image { alt, src, span, attrs }, next))
    }

    fn bracket(&mut self, i: usize, end: usize) -> Option<(RawInline, usize)> {
        let close = self.close_bracket(i, end)?;
        match self.src.as_bytes().get(close + 1) {
            Some(b'(') => {
                let paren = self.close_paren(close + 1, end)?;
                let children = self.parse(i + 1, close);
                let href = self.src[close + 2..paren].trim().to_string();
                let span = Span::new(self.line, self.offset + close + 2, self.offset + paren);
                Some((RawInline::Link { href, span, children }, paren + 1))
            }
            Some(b'{') => {
                let (attrs, next) = self.attached_attrs(close + 1, end)?;
                let children = self.parse(i + 1, close);
                Some((RawInline::Span { attrs, children }, next))
            }
            _ => None,
        }
    }
}

fn flush(buffer: &mut String, out: &mut Vec<RawInline>) {
    if !buffer.is_empty() {
        out.push(RawInline::Text(std::mem::take(buffer)));
    }
}
