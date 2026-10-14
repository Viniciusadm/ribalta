use crate::attrs::{Attrs, attached_attrs, is_name_char, parse_attrs, trailing_attrs};
use crate::diagnostic::Diagnostics;
use crate::text::{tr, Line, Span, Text};

pub const RAW_DIRECTIVES: &[&str] = &["notes"];
pub const PREAMBLE_DIRECTIVES: &[&str] = &["title", "author", "date", "description", "theme", "version", "page", "color", "font", "style", "component", "header", "footer", "overlay", "let", "state", "extern", "data", "transition", "mode", "persist", "scorm", "end"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub enum Node {
    Heading { level: u8, text: Text, attrs: Option<Attrs>, line: usize },
    Paragraph { lines: Vec<Text>, attrs: Option<Attrs>, line: usize },
    List(List),
    Quote { children: Vec<Node>, line: usize, end: usize },
    Code { lang: Option<String>, code: String, attrs: Option<Attrs>, line: usize, end: usize },
    Table(Table),
    Image { alt: Text, src: Text, attrs: Option<Attrs>, line: usize },
    Directive(Directive),
    Placeholder { name: String, span: Span },
}

#[derive(Debug, Clone)]
pub struct List {
    pub ordered: bool,
    pub start: u32,
    pub items: Vec<ListItem>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub lines: Vec<Text>,
    pub attrs: Option<Attrs>,
    pub children: Vec<List>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub header: Vec<Text>,
    pub aligns: Vec<ColumnAlign>,
    pub rows: Vec<Vec<Text>>,
    pub line: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub name: Text,
    pub value: Option<Text>,
    pub attrs: Option<Attrs>,
    pub children: Vec<Node>,
    pub raw: Option<String>,
    pub container: bool,
    pub line: usize,
    pub end: usize,
}

impl Node {
    pub fn end_line(&self) -> usize {
        match self {
            Node::Heading { line, .. } | Node::Image { line, .. } => *line,
            Node::Paragraph { lines, line, .. } => lines.last().map_or(*line, |text| text.span.line),
            Node::List(list) => list.end_line(),
            Node::Quote { end, .. } | Node::Code { end, .. } => *end,
            Node::Table(table) => table.end,
            Node::Directive(directive) => directive.end,
            Node::Placeholder { span, .. } => span.line,
        }
    }
}

impl List {
    pub fn end_line(&self) -> usize {
        self.items.iter().fold(self.line, |end, item| {
            let text = item.lines.last().map_or(item.line, |text| text.span.line);
            item.children.iter().map(List::end_line).fold(end.max(text), usize::max)
        })
    }
}

pub fn parse_blocks(lines: &[Line<'_>], diags: &mut Diagnostics) -> Vec<Node> {
    let mut parser = BlockParser { lines, pos: 0, diags };
    parser.sequence(None)
}

struct BlockParser<'a, 'd> {
    lines: &'a [Line<'a>],
    pos: usize,
    diags: &'d mut Diagnostics,
}

impl<'a> BlockParser<'a, '_> {
    fn sequence(&mut self, opener: Option<&Text>) -> Vec<Node> {
        let mut nodes = Vec::new();
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let trimmed = line.trimmed();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                self.pos += 1;
                continue;
            }
            if trimmed == ":::" {
                self.pos += 1;
                if opener.is_some() {
                    return nodes;
                }
                self.diags
                    .error("R101", line.full_span(), tr("`:::` fecha um bloco, mas nenhum bloco está aberto aqui", "`:::` closes a block, but no block is open here"))
                    .hint(tr("Remova esta linha ou abra um bloco antes, como `::: columns`", "Remove this line or open a block before it, such as `::: columns`"));
                continue;
            }
            if let Some(node) = self.block() {
                nodes.push(node);
            }
        }
        if let Some(name) = opener {
            self.diags
                .error("R102", name.span, tr(format!("o bloco `{}` foi aberto mas nunca fechado", name.value), format!("block `{}` was opened but never closed", name.value)))
                .hint(tr("Adicione uma linha contendo apenas `:::` no fim do bloco", "Add a line containing only `:::` at the end of the block"));
        }
        nodes
    }

    fn last_line(&self) -> usize {
        self.lines[self.pos.saturating_sub(1)].number
    }

    fn block(&mut self) -> Option<Node> {
        let line = self.lines[self.pos];
        let trimmed = line.text.trim_start();
        let indent = line.indent();
        if trimmed.starts_with(":::") {
            return Some(self.directive(line, indent, 3, true));
        }
        if trimmed.starts_with("::") {
            self.pos += 1;
            return Some(self.directive(line, indent, 2, false));
        }
        if trimmed.starts_with("```") {
            return Some(self.code(line, indent));
        }
        if let Some(level) = heading_level(trimmed) {
            self.pos += 1;
            return Some(heading(line, indent, level, self.diags));
        }
        if trimmed.starts_with('>') {
            return Some(self.quote());
        }
        if list_marker(trimmed).is_some() {
            return Some(Node::List(self.list(indent)));
        }
        if trimmed.starts_with('|') {
            return Some(self.table());
        }
        if let Some(name) = placeholder_name(trimmed) {
            self.pos += 1;
            return Some(Node::Placeholder { name: name.to_string(), span: line.full_span() });
        }
        if let Some(name) = preamble_directive(trimmed) {
            self.pos += 1;
            self.diags
                .error("R103", line.full_span(), tr(format!("`@{name}` só pode ser usado no início do documento, antes da primeira página"), format!("`@{name}` can only be used at the start of the document, before the first page")))
                .hint(tr("Mova esta definição para o topo do arquivo", "Move this definition to the top of the file"));
            return None;
        }
        if trimmed.starts_with("![")
            && let Some(node) = image_line(line, indent, self.diags) {
                self.pos += 1;
                return Some(node);
            }
        Some(self.paragraph())
    }

    fn directive(&mut self, line: Line<'a>, indent: usize, colons: usize, container: bool) -> Node {
        let header = line.slice(indent + colons);
        let (name, value, attrs) = directive_header(header, self.diags);
        let name = name.unwrap_or_else(|| {
            self.diags
                .error("R104", line.full_span(), tr("falta o nome do bloco depois de `:::`", "missing block name after `:::`"))
                .hint(tr("Exemplos: `::: columns`, `::: box`, `:: icon star`", "Examples: `::: columns`, `::: box`, `:: icon star`"));
            Text::new("", line.number, line.offset + indent)
        });
        let mut directive = Directive { name, value, attrs, children: Vec::new(), raw: None, container, line: line.number, end: line.number };
        if !container {
            return Node::Directive(directive);
        }
        self.pos += 1;
        if RAW_DIRECTIVES.contains(&directive.name.value.as_str()) {
            let mut raw = Vec::new();
            let mut closed = false;
            while self.pos < self.lines.len() {
                let current = self.lines[self.pos];
                self.pos += 1;
                if current.trimmed() == ":::" {
                    closed = true;
                    break;
                }
                raw.push(current.text.trim());
            }
            if !closed {
                self.diags
                    .error("R102", directive.name.span, tr(format!("o bloco `{}` foi aberto mas nunca fechado", directive.name.value), format!("block `{}` was opened but never closed", directive.name.value)))
                    .hint(tr("Adicione uma linha contendo apenas `:::` no fim do bloco", "Add a line containing only `:::` at the end of the block"));
            }
            directive.raw = Some(raw.join("\n").trim().to_string());
            directive.end = self.last_line();
            return Node::Directive(directive);
        }
        let name = directive.name.clone();
        directive.children = self.sequence(Some(&name));
        directive.end = self.last_line();
        Node::Directive(directive)
    }

    fn code(&mut self, line: Line<'a>, indent: usize) -> Node {
        let info = line.slice(indent + 3);
        let (info_text, attrs) = split_trailing(info, self.diags);
        let lang = info_text.value.trim();
        let lang = (!lang.is_empty()).then(|| lang.to_string());
        self.pos += 1;
        let mut code = Vec::new();
        let mut closed = false;
        while self.pos < self.lines.len() {
            let current = self.lines[self.pos];
            self.pos += 1;
            if current.text.trim_start().starts_with("```") {
                closed = true;
                break;
            }
            let strip = current.indent().min(indent);
            code.push(&current.text[strip..]);
        }
        if !closed {
            self.diags
                .error("R105", line.full_span(), tr("bloco de código sem fechamento", "unclosed code block"))
                .hint(tr("Feche o bloco com uma linha contendo ```", "Close the block with a line containing ```"));
        }
        Node::Code { lang, code: code.join("\n"), attrs, line: line.number, end: self.last_line() }
    }

    fn quote(&mut self) -> Node {
        let first = self.lines[self.pos].number;
        let mut inner = Vec::new();
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let indent = line.indent();
            if !line.text[indent..].starts_with('>') {
                break;
            }
            let mut start = indent + 1;
            if line.text[start..].starts_with(' ') {
                start += 1;
            }
            inner.push(line.slice(start));
            self.pos += 1;
        }
        let children = parse_blocks(&inner, self.diags);
        Node::Quote { children, line: first, end: self.last_line() }
    }

    fn list(&mut self, base: usize) -> List {
        let first = self.lines[self.pos];
        let (ordered, start, _) = list_marker(first.text.trim_start()).expect("caller checked marker");
        let mut list = List { ordered, start, items: Vec::new(), line: first.number };
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            if line.is_blank() {
                break;
            }
            let indent = line.indent();
            let trimmed = &line.text[indent..];
            match list_marker(trimmed) {
                Some(_) if indent > base && !list.items.is_empty() => {
                    let nested = self.list(indent);
                    list.items.last_mut().expect("checked non-empty").children.push(nested);
                }
                Some((item_ordered, _, marker)) if indent <= base => {
                    if indent < base || item_ordered != ordered {
                        break;
                    }
                    let content = line.slice(indent + marker);
                    let (text, attrs) = split_trailing(content, self.diags);
                    list.items.push(ListItem { lines: vec![text], attrs, children: Vec::new(), line: line.number });
                    self.pos += 1;
                }
                None if indent > base && !list.items.is_empty() && !starts_block(trimmed) => {
                    let text = line.text_at(indent, line.text.trim_end().len());
                    list.items.last_mut().expect("checked non-empty").lines.push(text);
                    self.pos += 1;
                }
                _ => break,
            }
        }
        list
    }

    fn table(&mut self) -> Node {
        let first = self.lines[self.pos].number;
        let mut rows = Vec::new();
        while self.pos < self.lines.len() && self.lines[self.pos].text.trim_start().starts_with('|') {
            rows.push(self.lines[self.pos]);
            self.pos += 1;
        }
        let mut cells: Vec<Vec<Text>> = rows.iter().map(|line| table_cells(*line)).collect();
        let separator = cells.get(1).filter(|row| row.iter().all(|cell| is_separator_cell(&cell.value))).cloned();
        let Some(separator) = separator else {
            self.diags
                .error("R106", rows[0].full_span(), tr("tabela sem linha separadora depois do cabeçalho", "table without a separator line after the header"))
                .hint(tr("Adicione uma linha como | --- | --- | logo abaixo do cabeçalho", "Add a line like | --- | --- | right below the header"));
            let header = cells.remove(0);
            return Node::Table(Table { aligns: vec![ColumnAlign::Left; header.len()], header, rows: cells, line: first, end: self.last_line() });
        };
        let header = cells.remove(0);
        cells.remove(0);
        let aligns = (0..header.len())
            .map(|index| separator.get(index).map_or(ColumnAlign::Left, |cell| column_align(&cell.value)))
            .collect();
        for (index, row) in cells.iter().enumerate() {
            if row.len() != header.len() {
                self.diags.warning(
                    "R107",
                    rows[index + 2].full_span(),
                    tr(format!("esta linha tem {} colunas, mas o cabeçalho tem {}", row.len(), header.len()), format!("this row has {} columns, but the header has {}", row.len(), header.len())),
                );
            }
        }
        Node::Table(Table { header, aligns, rows: cells, line: first, end: self.last_line() })
    }

    fn paragraph(&mut self) -> Node {
        let first = self.lines[self.pos].number;
        let mut lines = Vec::new();
        let mut attrs = None;
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let trimmed = line.text.trim_start();
            if line.is_blank() || (!lines.is_empty() && starts_block(trimmed)) || trimmed.trim_end() == ":::" {
                break;
            }
            let (text, line_attrs) = split_trailing(line.slice(line.indent()), self.diags);
            lines.push(text);
            self.pos += 1;
            if line_attrs.is_some() {
                attrs = line_attrs;
                break;
            }
        }
        Node::Paragraph { lines, attrs, line: first }
    }
}

pub fn split_trailing(line: Line<'_>, diags: &mut Diagnostics) -> (Text, Option<Attrs>) {
    match trailing_attrs(line.text) {
        Some((open, close)) => {
            let attrs = parse_attrs(&line.text[open + 1..close], line.number, line.offset + open + 1, diags);
            let end = line.text[..open].trim_end().len();
            (line.text_at(0, end), Some(attrs))
        }
        None => (line.text_at(0, line.text.trim_end().len()), None),
    }
}

pub fn directive_header(line: Line<'_>, diags: &mut Diagnostics) -> (Option<Text>, Option<Text>, Option<Attrs>) {
    let start = line.indent();
    let name_end = line.text[start..].find(|c: char| !is_name_char(c)).map_or(line.text.len(), |index| start + index);
    let name = (name_end > start).then(|| line.text_at(start, name_end));
    let rest = line.slice(name_end);
    let (value_end, attrs) = match attached_attrs(rest.text) {
        Some((open, close)) => (open, Some(parse_attrs(&rest.text[open + 1..close], rest.number, rest.offset + open + 1, diags))),
        None => (rest.text.len(), None),
    };
    let raw_value = &rest.text[..value_end];
    let value_start = raw_value.len() - raw_value.trim_start().len();
    let value = raw_value.trim();
    let value = (!value.is_empty()).then(|| rest.text_at(value_start, value_start + value.len()));
    (name, value, attrs)
}

fn heading(line: Line<'_>, indent: usize, level: u8, diags: &mut Diagnostics) -> Node {
    let content = line.slice(indent + level as usize);
    let content = content.slice(content.indent());
    let (text, attrs) = split_trailing(content, diags);
    Node::Heading { level, text, attrs, line: line.number }
}

fn image_line(line: Line<'_>, indent: usize, diags: &mut Diagnostics) -> Option<Node> {
    let text = &line.text[indent..];
    let bytes = text.as_bytes();
    let mut depth = 0;
    let mut close = None;
    for (index, byte) in bytes.iter().enumerate().skip(1) {
        match byte {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    if bytes.get(close + 1) != Some(&b'(') {
        return None;
    }
    let paren = text[close + 1..].find(')')? + close + 1;
    let rest = text[paren + 1..].trim();
    let attrs = if rest.is_empty() {
        None
    } else if rest.starts_with('{') && rest.ends_with('}') {
        let open = paren + 1 + text[paren + 1..].find('{')?;
        let end = text.trim_end().len() - 1;
        Some(parse_attrs(&text[open + 1..end], line.number, line.offset + indent + open + 1, diags))
    } else {
        return None;
    };
    let alt = line.text_at(indent + 2, indent + close);
    let src_raw = &text[close + 2..paren];
    let src_start = indent + close + 2 + (src_raw.len() - src_raw.trim_start().len());
    let src = line.text_at(src_start, src_start + src_raw.trim().len());
    Some(Node::Image { alt, src, attrs, line: line.number })
}

fn table_cells(line: Line<'_>) -> Vec<Text> {
    let text = line.text;
    let start = line.indent() + 1;
    let mut end = text.trim_end().len();
    if end > start && text[..end].ends_with('|') && !text[..end].ends_with("\\|") {
        end -= 1;
    }
    let bytes = text.as_bytes();
    let mut cells = Vec::new();
    let mut cell_start = start;
    let mut in_code = false;
    let mut i = start;
    while i <= end {
        if i == end || (bytes[i] == b'|' && !in_code && bytes[i - 1] != b'\\') {
            let raw = &text[cell_start..i];
            let lead = raw.len() - raw.trim_start().len();
            cells.push(line.text_at(cell_start + lead, cell_start + lead + raw.trim().len()));
            cell_start = i + 1;
        } else if bytes[i] == b'`' {
            in_code = !in_code;
        }
        i += 1;
    }
    cells
}

fn is_separator_cell(cell: &str) -> bool {
    let core = cell.trim().trim_start_matches(':').trim_end_matches(':');
    !core.is_empty() && core.chars().all(|c| c == '-')
}

fn column_align(cell: &str) -> ColumnAlign {
    let cell = cell.trim();
    match (cell.starts_with(':'), cell.ends_with(':')) {
        (true, true) => ColumnAlign::Center,
        (false, true) => ColumnAlign::Right,
        _ => ColumnAlign::Left,
    }
}

pub fn heading_level(trimmed: &str) -> Option<u8> {
    let level = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    let after = trimmed.as_bytes().get(level);
    ((1..=6).contains(&level) && matches!(after, Some(b' ') | Some(b'\t'))).then_some(level as u8)
}

pub fn list_marker(trimmed: &str) -> Option<(bool, u32, usize)> {
    let bytes = trimmed.as_bytes();
    if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) && matches!(bytes.get(1), Some(b' ')) {
        return Some((false, 1, 2));
    }
    let digits = bytes.iter().take_while(|byte| byte.is_ascii_digit()).count();
    if (1..=9).contains(&digits) && matches!(bytes.get(digits), Some(b'.' | b')')) && matches!(bytes.get(digits + 1), Some(b' ')) {
        return Some((true, trimmed[..digits].parse().unwrap_or(1), digits + 2));
    }
    None
}

fn starts_block(trimmed: &str) -> bool {
    trimmed.starts_with("::")
        || trimmed.starts_with("```")
        || trimmed.starts_with('>')
        || trimmed.starts_with('|')
        || trimmed.starts_with("//")
        || heading_level(trimmed).is_some()
        || list_marker(trimmed).is_some()
        || placeholder_name(trimmed).is_some()
}

pub fn placeholder_name(trimmed: &str) -> Option<&str> {
    let name = trimmed.trim_end().strip_prefix('$')?;
    let valid = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    valid.then_some(name)
}

fn preamble_directive(trimmed: &str) -> Option<&'static str> {
    let word = trimmed.strip_prefix('@')?.split(|c: char| c.is_whitespace() || c == '{').next()?;
    PREAMBLE_DIRECTIVES.iter().copied().find(|name| *name == word)
}
