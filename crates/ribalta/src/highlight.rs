use crate::ir::{Token, TokenKind};

struct Grammar {
    line_comments: &'static [&'static str],
    block_comment: Option<(&'static str, &'static str)>,
    quotes: &'static [char],
    multiline_quotes: &'static [char],
    triple_quotes: bool,
    keywords: &'static [&'static str],
    constants: &'static [&'static str],
    types: &'static [&'static str],
    capitalized_types: bool,
    ignore_case: bool,
    sigil: Option<char>,
    dollar_idents: bool,
    dash_idents: bool,
    keys: bool,
    directives: bool,
    lifetimes: bool,
    decorators: bool,
}

const BASE: Grammar = Grammar {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\''],
    multiline_quotes: &[],
    triple_quotes: false,
    keywords: &[],
    constants: &["true", "false", "null"],
    types: &[],
    capitalized_types: true,
    ignore_case: false,
    sigil: None,
    dollar_idents: false,
    dash_idents: false,
    keys: false,
    directives: false,
    lifetimes: false,
    decorators: false,
};

const JAVASCRIPT: Grammar = Grammar {
    quotes: &['"', '\'', '`'],
    multiline_quotes: &['`'],
    keywords: &[
        "as", "async", "await", "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete", "do", "else", "enum", "export", "extends", "finally", "for",
        "from", "function", "get", "if", "implements", "import", "in", "instanceof", "interface", "keyof", "let", "new", "of", "private", "protected", "public", "readonly", "return",
        "satisfies", "set", "static", "super", "switch", "this", "throw", "try", "type", "typeof", "var", "void", "while", "with", "yield",
    ],
    constants: &["true", "false", "null", "undefined", "NaN", "Infinity"],
    types: &["any", "boolean", "never", "number", "object", "string", "symbol", "unknown", "bigint"],
    dollar_idents: true,
    decorators: true,
    ..BASE
};

const PYTHON: Grammar = Grammar {
    line_comments: &["#"],
    block_comment: None,
    triple_quotes: true,
    keywords: &[
        "and", "as", "assert", "async", "await", "break", "case", "class", "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in",
        "is", "lambda", "match", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
    ],
    constants: &["True", "False", "None", "self"],
    types: &["int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes"],
    decorators: true,
    ..BASE
};

const RUBY: Grammar = Grammar {
    line_comments: &["#"],
    block_comment: None,
    keywords: &[
        "alias", "and", "begin", "break", "case", "class", "def", "defined?", "do", "else", "elsif", "end", "ensure", "for", "if", "in", "module", "next", "not", "or", "redo", "require",
        "rescue", "retry", "return", "self", "super", "then", "unless", "until", "when", "while", "yield",
    ],
    constants: &["true", "false", "nil"],
    sigil: Some('@'),
    ..BASE
};

const RUST: Grammar = Grammar {
    quotes: &['"'],
    keywords: &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
        "ref", "return", "self", "Self", "static", "struct", "super", "trait", "type", "unsafe", "use", "where", "while",
    ],
    constants: &["true", "false", "None", "Some", "Ok", "Err"],
    types: &["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64", "bool", "char", "str"],
    lifetimes: true,
    ..BASE
};

const GO: Grammar = Grammar {
    quotes: &['"', '\'', '`'],
    multiline_quotes: &['`'],
    keywords: &[
        "break", "case", "chan", "const", "continue", "default", "defer", "else", "fallthrough", "for", "func", "go", "goto", "if", "import", "interface", "map", "package", "range",
        "return", "select", "struct", "switch", "type", "var",
    ],
    constants: &["true", "false", "nil", "iota"],
    types: &["bool", "byte", "error", "float32", "float64", "int", "int8", "int16", "int32", "int64", "rune", "string", "uint", "uint8", "uint16", "uint32", "uint64", "any"],
    ..BASE
};

const C_FAMILY: Grammar = Grammar {
    keywords: &[
        "abstract", "async", "await", "break", "case", "catch", "class", "const", "constexpr", "continue", "data", "default", "defer", "delete", "do", "else", "enum", "export", "extends",
        "extension", "final", "finally", "for", "fun", "func", "guard", "if", "implements", "import", "in", "inline", "interface", "internal", "is", "late", "let", "namespace", "new",
        "object", "open", "operator", "override", "package", "private", "protocol", "protected", "public", "required", "return", "sealed", "static", "struct", "super", "switch", "template",
        "this", "throw", "throws", "try", "typedef", "typename", "union", "using", "val", "var", "virtual", "void", "volatile", "when", "where", "while", "with", "yield",
    ],
    constants: &["true", "false", "null", "nullptr", "nil", "NULL"],
    types: &["int", "long", "short", "char", "float", "double", "bool", "boolean", "byte", "string", "unsigned", "signed", "auto", "size_t", "dynamic", "Int", "String", "Bool", "Double"],
    directives: true,
    decorators: true,
    ..BASE
};

const PHP: Grammar = Grammar {
    line_comments: &["//", "#"],
    keywords: &[
        "abstract", "as", "break", "case", "catch", "class", "const", "continue", "default", "do", "echo", "else", "elseif", "enum", "extends", "final", "finally", "fn", "for", "foreach",
        "function", "if", "implements", "interface", "match", "namespace", "new", "private", "protected", "public", "readonly", "require", "return", "static", "switch", "throw", "trait",
        "try", "use", "while", "yield",
    ],
    constants: &["true", "false", "null", "TRUE", "FALSE", "NULL"],
    sigil: Some('$'),
    ..BASE
};

const SHELL: Grammar = Grammar {
    line_comments: &["#"],
    block_comment: None,
    quotes: &['"', '\'', '`'],
    keywords: &[
        "case", "do", "done", "elif", "else", "esac", "export", "fi", "for", "function", "if", "in", "local", "return", "select", "then", "until", "while", "sudo", "cd", "echo", "exit",
    ],
    constants: &["true", "false"],
    capitalized_types: false,
    sigil: Some('$'),
    dash_idents: true,
    ..BASE
};

const SQL: Grammar = Grammar {
    line_comments: &["--"],
    keywords: &[
        "add", "all", "alter", "and", "as", "asc", "between", "by", "case", "create", "delete", "desc", "distinct", "drop", "else", "end", "exists", "from", "full", "group", "having", "in",
        "index", "inner", "insert", "into", "is", "join", "key", "left", "like", "limit", "not", "offset", "on", "or", "order", "outer", "primary", "references", "right", "select", "set",
        "table", "then", "union", "unique", "update", "values", "when", "where", "with",
    ],
    constants: &["true", "false", "null"],
    types: &["int", "integer", "bigint", "varchar", "char", "text", "boolean", "date", "timestamp", "decimal", "float", "json", "uuid", "serial"],
    capitalized_types: false,
    ignore_case: true,
    ..BASE
};

const JSON: Grammar = Grammar { line_comments: &[], block_comment: None, quotes: &['"'], capitalized_types: false, keys: true, ..BASE };

const YAML: Grammar = Grammar {
    line_comments: &["#"],
    block_comment: None,
    constants: &["true", "false", "null", "yes", "no", "on", "off", "~"],
    capitalized_types: false,
    dash_idents: true,
    keys: true,
    ..BASE
};

enum Lexer {
    Generic(&'static Grammar),
    Markup,
    Css,
}

fn lexer(lang: &str) -> Option<Lexer> {
    let grammar = match lang.to_ascii_lowercase().as_str() {
        "js" | "javascript" | "jsx" | "mjs" | "cjs" | "ts" | "typescript" | "tsx" => &JAVASCRIPT,
        "py" | "python" => &PYTHON,
        "rb" | "ruby" => &RUBY,
        "rs" | "rust" => &RUST,
        "go" | "golang" => &GO,
        "c" | "h" | "cpp" | "c++" | "hpp" | "cs" | "csharp" | "c#" | "java" | "kt" | "kotlin" | "swift" | "dart" | "scala" | "objc" | "objectivec" => &C_FAMILY,
        "php" => &PHP,
        "sh" | "bash" | "shell" | "zsh" | "console" => &SHELL,
        "sql" | "mysql" | "postgres" | "postgresql" | "sqlite" => &SQL,
        "json" | "jsonc" | "json5" => &JSON,
        "yaml" | "yml" | "toml" => &YAML,
        "html" | "xml" | "svg" | "markup" | "vue" | "svelte" => return Some(Lexer::Markup),
        "css" | "scss" | "sass" | "less" => return Some(Lexer::Css),
        _ => return None,
    };
    Some(Lexer::Generic(grammar))
}

pub fn highlight(code: &str, lang: Option<&str>) -> Vec<Vec<Token>> {
    let chars: Vec<char> = code.chars().collect();
    let pieces = match lang.and_then(lexer) {
        Some(Lexer::Generic(grammar)) => generic(&chars, grammar),
        Some(Lexer::Markup) => markup(&chars),
        Some(Lexer::Css) => css(&chars),
        None => vec![(TokenKind::Plain, code.to_string())],
    };
    into_lines(pieces)
}

fn into_lines(pieces: Vec<(TokenKind, String)>) -> Vec<Vec<Token>> {
    let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
    for (kind, text) in pieces {
        for (index, part) in text.split('\n').enumerate() {
            if index > 0 {
                lines.push(Vec::new());
            }
            if part.is_empty() {
                continue;
            }
            let line = lines.last_mut().expect("there is always a line");
            match line.last_mut() {
                Some(last) if last.kind == kind => last.text.push_str(part),
                _ => line.push(Token { kind, text: part.to_string() }),
            }
        }
    }
    lines
}

struct Cursor<'a> {
    chars: &'a [char],
    at: usize,
    out: Vec<(TokenKind, String)>,
}

impl<'a> Cursor<'a> {
    fn new(chars: &'a [char]) -> Self {
        Self { chars, at: 0, out: Vec::new() }
    }

    fn done(&self) -> bool {
        self.at >= self.chars.len()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.at + offset).copied()
    }

    fn starts_with(&self, text: &str) -> bool {
        text.chars().enumerate().all(|(index, c)| self.peek(index) == Some(c))
    }

    fn emit(&mut self, kind: TokenKind, from: usize) {
        if self.at > from {
            self.out.push((kind, self.chars[from..self.at].iter().collect()));
        }
    }

    fn take(&mut self, kind: TokenKind, count: usize) {
        let from = self.at;
        self.at = (self.at + count).min(self.chars.len());
        self.emit(kind, from);
    }

    fn until(&mut self, kind: TokenKind, from: usize, end: &str) {
        while !self.done() && !self.starts_with(end) {
            self.at += 1;
        }
        self.at = (self.at + end.chars().count()).min(self.chars.len());
        self.emit(kind, from);
    }

    fn rest_of_line(&mut self, kind: TokenKind) {
        let from = self.at;
        while self.peek(0).is_some_and(|c| c != '\n') {
            self.at += 1;
        }
        self.emit(kind, from);
    }

    fn next_significant(&self) -> Option<char> {
        self.chars[self.at..].iter().copied().find(|c| *c != ' ' && *c != '\t')
    }

    fn at_line_start(&self) -> bool {
        self.chars[..self.at].iter().rev().take_while(|c| **c != '\n').all(|c| c.is_whitespace())
    }

    fn quoted(&mut self, quote: char, multiline: bool) {
        let from = self.at;
        self.at += 1;
        while let Some(c) = self.peek(0) {
            if c == '\\' {
                self.at += 2;
                continue;
            }
            if c == '\n' && !multiline {
                break;
            }
            self.at += 1;
            if c == quote {
                break;
            }
        }
        self.at = self.at.min(self.chars.len());
        self.emit(TokenKind::String, from);
    }
}

fn is_operator(c: char) -> bool {
    matches!(c, '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' | '!' | '&' | '|' | '^' | '~' | '?' | ':')
}

fn is_punctuation(c: char) -> bool {
    matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '.')
}

fn generic(chars: &[char], grammar: &Grammar) -> Vec<(TokenKind, String)> {
    let mut cursor = Cursor::new(chars);
    let ident_char = |c: char| c.is_alphanumeric() || c == '_' || (grammar.dollar_idents && c == '$') || (grammar.dash_idents && c == '-');
    let matches_word = |list: &[&str], word: &str| if grammar.ignore_case { list.iter().any(|item| item.eq_ignore_ascii_case(word)) } else { list.contains(&word) };
    while let Some(c) = cursor.peek(0) {
        let from = cursor.at;
        if let Some((open, close)) = grammar.block_comment.filter(|(open, _)| cursor.starts_with(open)) {
            cursor.at += open.chars().count();
            cursor.until(TokenKind::Comment, from, close);
            continue;
        }
        if grammar.line_comments.iter().any(|marker| cursor.starts_with(marker)) {
            cursor.rest_of_line(TokenKind::Comment);
            continue;
        }
        if grammar.directives && c == '#' && cursor.at_line_start() {
            cursor.at += 1;
            while cursor.peek(0).is_some_and(|c| c.is_alphanumeric()) {
                cursor.at += 1;
            }
            cursor.emit(TokenKind::Keyword, from);
            continue;
        }
        if grammar.triple_quotes && (cursor.starts_with("\"\"\"") || cursor.starts_with("'''")) {
            let delimiter: String = chars[from..from + 3].iter().collect();
            cursor.at += 3;
            cursor.until(TokenKind::String, from, &delimiter);
            continue;
        }
        if grammar.lifetimes && c == '\'' {
            if cursor.peek(1) == Some('\\') || cursor.peek(2) == Some('\'') {
                cursor.quoted('\'', false);
            } else {
                cursor.at += 1;
                while cursor.peek(0).is_some_and(ident_char) {
                    cursor.at += 1;
                }
                cursor.emit(TokenKind::Type, from);
            }
            continue;
        }
        if grammar.quotes.contains(&c) {
            cursor.quoted(c, grammar.multiline_quotes.contains(&c));
            if grammar.keys && cursor.next_significant() == Some(':') {
                cursor.out.last_mut().expect("a string was emitted").0 = TokenKind::Property;
            }
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && cursor.peek(1).is_some_and(|next| next.is_ascii_digit())) {
            while let Some(next) = cursor.peek(0) {
                let exponent_sign = matches!(next, '+' | '-') && matches!(cursor.chars[cursor.at - 1], 'e' | 'E') && !cursor.chars[from..cursor.at].iter().any(|c| matches!(c, 'x' | 'X'));
                if next.is_alphanumeric() || next == '_' || exponent_sign || (next == '.' && cursor.peek(1).is_some_and(|after| after.is_ascii_digit())) {
                    cursor.at += 1;
                } else {
                    break;
                }
            }
            cursor.emit(TokenKind::Number, from);
            continue;
        }
        if grammar.decorators && c == '@' && cursor.peek(1).is_some_and(|next| next.is_alphabetic() || next == '_') {
            cursor.at += 1;
            while cursor.peek(0).is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '.')) {
                cursor.at += 1;
            }
            cursor.emit(TokenKind::Function, from);
            continue;
        }
        if grammar.sigil == Some(c) && cursor.peek(1).is_some_and(|next| next.is_alphabetic() || next == '_' || next == '{') {
            cursor.at += 1;
            if cursor.peek(0) == Some('{') {
                cursor.until(TokenKind::Variable, from, "}");
            } else {
                while cursor.peek(0).is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    cursor.at += 1;
                }
                cursor.emit(TokenKind::Variable, from);
            }
            continue;
        }
        if c.is_alphabetic() || c == '_' || (grammar.dollar_idents && c == '$') {
            while cursor.peek(0).is_some_and(ident_char) {
                cursor.at += 1;
            }
            if grammar.keywords.iter().any(|keyword| keyword.ends_with('?')) && cursor.peek(0) == Some('?') {
                let word: String = chars[from..=cursor.at].iter().collect();
                if grammar.keywords.contains(&word.as_str()) {
                    cursor.at += 1;
                }
            }
            let word: String = chars[from..cursor.at].iter().collect();
            let next = cursor.next_significant();
            let kind = if grammar.keys && next == Some(':') && cursor.peek(1) != Some(':') {
                TokenKind::Property
            } else if matches_word(grammar.keywords, &word) {
                TokenKind::Keyword
            } else if matches_word(grammar.constants, &word) {
                TokenKind::Constant
            } else if matches_word(grammar.types, &word) {
                TokenKind::Type
            } else if next == Some('(') {
                TokenKind::Function
            } else if grammar.capitalized_types && word.starts_with(|c: char| c.is_uppercase()) {
                TokenKind::Type
            } else {
                TokenKind::Plain
            };
            cursor.emit(kind, from);
            continue;
        }
        if is_operator(c) {
            while cursor.peek(0).is_some_and(is_operator) && !grammar.line_comments.iter().any(|marker| cursor.starts_with(marker)) {
                cursor.at += 1;
            }
            cursor.emit(TokenKind::Operator, from);
            continue;
        }
        if is_punctuation(c) {
            cursor.take(TokenKind::Punctuation, 1);
            continue;
        }
        cursor.take(TokenKind::Plain, 1);
    }
    cursor.out
}

fn markup(chars: &[char]) -> Vec<(TokenKind, String)> {
    let mut cursor = Cursor::new(chars);
    while let Some(c) = cursor.peek(0) {
        let from = cursor.at;
        if cursor.starts_with("<!--") {
            cursor.at += 4;
            cursor.until(TokenKind::Comment, from, "-->");
        } else if cursor.starts_with("<!") || cursor.starts_with("<?") {
            cursor.until(TokenKind::Keyword, from, ">");
        } else if c == '<' && cursor.peek(1).is_some_and(|next| next.is_alphabetic() || next == '/') {
            cursor.take(TokenKind::Punctuation, if cursor.peek(1) == Some('/') { 2 } else { 1 });
            let name = cursor.at;
            while cursor.peek(0).is_some_and(|c| c.is_alphanumeric() || matches!(c, '-' | ':' | '_' | '.')) {
                cursor.at += 1;
            }
            cursor.emit(TokenKind::Tag, name);
            while let Some(c) = cursor.peek(0) {
                let from = cursor.at;
                if c == '>' || cursor.starts_with("/>") {
                    cursor.take(TokenKind::Punctuation, if c == '>' { 1 } else { 2 });
                    break;
                } else if c == '"' || c == '\'' {
                    cursor.quoted(c, true);
                } else if c == '=' {
                    cursor.take(TokenKind::Operator, 1);
                } else if c.is_whitespace() {
                    cursor.take(TokenKind::Plain, 1);
                } else {
                    while cursor.peek(0).is_some_and(|c| !c.is_whitespace() && !matches!(c, '=' | '>' | '"' | '\'') && !cursor.starts_with("/>")) {
                        cursor.at += 1;
                    }
                    if cursor.at == from {
                        cursor.at += 1;
                    }
                    cursor.emit(TokenKind::Attribute, from);
                }
            }
        } else if c == '&' && chars[from..].iter().take(12).position(|c| *c == ';').is_some_and(|end| end > 1 && chars[from + 1..from + end].iter().all(|c| c.is_alphanumeric() || *c == '#')) {
            cursor.until(TokenKind::Constant, from, ";");
        } else {
            cursor.take(TokenKind::Plain, 1);
        }
    }
    cursor.out
}

fn css(chars: &[char]) -> Vec<(TokenKind, String)> {
    let mut cursor = Cursor::new(chars);
    let mut depth = 0usize;
    let word_char = |c: char| c.is_alphanumeric() || matches!(c, '-' | '_');
    while let Some(c) = cursor.peek(0) {
        let from = cursor.at;
        if cursor.starts_with("/*") {
            cursor.at += 2;
            cursor.until(TokenKind::Comment, from, "*/");
        } else if c == '"' || c == '\'' {
            cursor.quoted(c, false);
        } else if c == '@' {
            cursor.at += 1;
            while cursor.peek(0).is_some_and(word_char) {
                cursor.at += 1;
            }
            cursor.emit(TokenKind::Keyword, from);
        } else if c == '{' || c == '}' {
            depth = if c == '{' { depth + 1 } else { depth.saturating_sub(1) };
            cursor.take(TokenKind::Punctuation, 1);
        } else if matches!(c, ';' | ',' | '(' | ')') {
            cursor.take(TokenKind::Punctuation, 1);
        } else if depth > 0 && c == '#' && cursor.peek(1).is_some_and(|next| next.is_ascii_hexdigit()) {
            cursor.at += 1;
            while cursor.peek(0).is_some_and(|c| c.is_ascii_alphanumeric()) {
                cursor.at += 1;
            }
            cursor.emit(TokenKind::Number, from);
        } else if depth > 0 && (c.is_ascii_digit() || (matches!(c, '.' | '-') && cursor.peek(1).is_some_and(|next| next.is_ascii_digit()))) {
            cursor.at += 1;
            while cursor.peek(0).is_some_and(|c| c.is_alphanumeric() || matches!(c, '.' | '%')) {
                cursor.at += 1;
            }
            cursor.emit(TokenKind::Number, from);
        } else if word_char(c) {
            while cursor.peek(0).is_some_and(word_char) {
                cursor.at += 1;
            }
            let next = cursor.next_significant();
            let kind = if depth == 0 {
                TokenKind::Tag
            } else if next == Some(':') && property_position(&cursor.out) {
                TokenKind::Property
            } else if next == Some('(') {
                TokenKind::Function
            } else if chars[from] == '-' && chars.get(from + 1) == Some(&'-') {
                TokenKind::Variable
            } else {
                TokenKind::Plain
            };
            cursor.emit(kind, from);
        } else if depth == 0 && matches!(c, '.' | '#' | ':' | '>' | '+' | '~' | '*' | '[' | ']' | '=') {
            cursor.take(TokenKind::Tag, 1);
        } else if c == ':' || c == '!' {
            cursor.take(TokenKind::Operator, 1);
        } else {
            cursor.take(TokenKind::Plain, 1);
        }
    }
    cursor.out
}

fn property_position(out: &[(TokenKind, String)]) -> bool {
    out.iter()
        .rev()
        .find(|(kind, text)| !(*kind == TokenKind::Plain && text.trim().is_empty()) && *kind != TokenKind::Comment)
        .is_none_or(|(_, text)| matches!(text.as_str(), "{" | ";" | "}"))
}
