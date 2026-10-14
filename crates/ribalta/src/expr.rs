use crate::ir::BinaryOp;
use crate::text::tr;

pub const MAX_DEPTH: usize = 32;

pub const BUILTINS: &[&str] = &["len", "min", "max", "sum", "round", "random", "pick"];

#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub kind: AstKind,
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstKind {
    Number(f64),
    Text(String),
    Bool(bool),
    Var(Vec<String>),
    Not(Box<Ast>),
    Neg(Box<Ast>),
    Binary(BinaryOp, Box<Ast>, Box<Ast>),
    Range(Box<Ast>, Box<Ast>),
    Cond(Box<Ast>, Box<Ast>, Box<Ast>),
    List(Vec<Ast>),
    Call(String, Vec<Ast>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprError {
    pub code: &'static str,
    pub message: String,
    pub hint: Option<String>,
    pub from: usize,
    pub to: usize,
}

impl ExprError {
    pub fn new(code: &'static str, message: String, from: usize, to: usize) -> Self {
        Self { code, message, hint: None, from, to }
    }

    pub fn hint(mut self, hint: String) -> Self {
        self.hint = Some(hint);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Text(String),
    Word(String),
    Var(Vec<String>),
    Op(&'static str),
    End,
}

const OPERATORS: &[&str] = &["==", "!=", "<=", ">=", "&&", "||", "..", "<", ">", "+", "-", "*", "/", "%", "(", ")", "[", "]", ",", "?", ":", "!"];

fn syntax(message: String, from: usize, to: usize) -> ExprError {
    ExprError::new("R603", message, from, to)
}

pub fn name_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte.is_ascii_alphanumeric() || byte == b'_' || (byte == b'-' && bytes.get(i + 1).is_some_and(u8::is_ascii_alphabetic) && i > start) {
            i += 1;
        } else {
            break;
        }
    }
    i
}

pub fn path_at(source: &str, start: usize) -> Option<(Vec<String>, usize)> {
    let bytes = source.as_bytes();
    if !bytes.get(start).is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_') {
        return None;
    }
    let mut end = name_end(source, start);
    let mut path = vec![source[start..end].to_string()];
    while bytes.get(end) == Some(&b'.') && bytes.get(end + 1).is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_') {
        let next = name_end(source, end + 1);
        path.push(source[end + 1..next].to_string());
        end = next;
    }
    Some((path, end))
}

fn tokenize(source: &str) -> Result<Vec<(Token, usize, usize)>, ExprError> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if byte == b'$' {
            if bytes.get(i + 1) == Some(&b'(') {
                tokens.push((Token::Op("("), i, i + 2));
                i += 2;
                continue;
            }
            match path_at(source, i + 1) {
                Some((path, end)) => {
                    tokens.push((Token::Var(path), start, end));
                    i = end;
                }
                None => return Err(syntax(tr("`$` precisa ser seguido do nome de uma variável", "`$` must be followed by a variable name"), start, start + 1)),
            }
            continue;
        }
        if byte == b'"' || byte == b'\'' {
            let mut value = String::new();
            let mut j = i + 1;
            let mut closed = false;
            while j < bytes.len() {
                let c = source[j..].chars().next().expect("in bounds");
                if c == '\\' && j + 1 < bytes.len() {
                    let next = source[j + 1..].chars().next().expect("in bounds");
                    value.push(next);
                    j += 1 + next.len_utf8();
                    continue;
                }
                j += c.len_utf8();
                if c as u32 == u32::from(byte) {
                    closed = true;
                    break;
                }
                value.push(c);
            }
            if !closed {
                return Err(syntax(tr("texto sem aspas de fechamento", "text without closing quotes"), start, bytes.len()));
            }
            tokens.push((Token::Text(value), start, j));
            i = j;
            continue;
        }
        if byte.is_ascii_digit() {
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if bytes.get(j) == Some(&b'.') && bytes.get(j + 1).is_some_and(u8::is_ascii_digit) {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
            }
            if bytes.get(j).is_some_and(|next| next.is_ascii_alphabetic() || *next == b'%') {
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || matches!(bytes[j], b'%' | b'_')) {
                    j += 1;
                }
                tokens.push((Token::Word(source[start..j].to_string()), start, j));
            } else {
                let value = source[start..j].parse::<f64>().map_err(|_| syntax(tr("número inválido", "invalid number"), start, j))?;
                tokens.push((Token::Number(value), start, j));
            }
            i = j;
            continue;
        }
        let current = source[i..].chars().next().expect("in bounds");
        if current.is_alphabetic() || current == '_' || current == '#' {
            let mut j = i + current.len_utf8();
            while j < bytes.len() {
                let next = source[j..].chars().next().expect("in bounds");
                let after = source[j + next.len_utf8()..].chars().next();
                if next.is_alphanumeric() || next == '_' || (next == '-' && after.is_some_and(char::is_alphanumeric)) {
                    j += next.len_utf8();
                } else if next == '/' && after.is_some_and(|c| c.is_ascii_digit()) && current != '#' {
                    j += 1;
                } else {
                    break;
                }
            }
            tokens.push((Token::Word(source[start..j].to_string()), start, j));
            i = j;
            continue;
        }
        if let Some(op) = OPERATORS.iter().find(|op| source[i..].starts_with(**op)) {
            tokens.push((Token::Op(op), start, start + op.len()));
            i += op.len();
            continue;
        }
        if byte == b'=' {
            return Err(syntax(tr("use `==` para comparar valores", "use `==` to compare values"), start, start + 1));
        }
        let c = source[i..].chars().next().expect("in bounds");
        return Err(syntax(tr(format!("caractere inesperado: `{c}`"), format!("unexpected character: `{c}`")), start, start + c.len_utf8()));
    }
    tokens.push((Token::End, bytes.len(), bytes.len()));
    Ok(tokens)
}

pub fn parse(source: &str) -> Result<Ast, ExprError> {
    let tokens = tokenize(source)?;
    if tokens.len() == 1 {
        return Err(syntax(tr("expressão vazia", "empty expression"), 0, source.len()));
    }
    let mut parser = Parser { tokens, pos: 0, depth: 0 };
    let ast = parser.expression()?;
    let (token, from, to) = parser.peek().clone();
    if token != Token::End {
        return Err(syntax(tr(format!("trecho inesperado: `{}`", &source[from..to]), format!("unexpected text: `{}`", &source[from..to])), from, source.len()));
    }
    Ok(ast)
}

struct Parser {
    tokens: Vec<(Token, usize, usize)>,
    pos: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> &(Token, usize, usize) {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> (Token, usize, usize) {
        let token = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn is_op(&self, op: &str) -> bool {
        matches!(&self.peek().0, Token::Op(current) if *current == op)
    }

    fn is_word(&self, word: &str) -> bool {
        matches!(&self.peek().0, Token::Word(current) if current == word)
    }

    fn expect(&mut self, op: &str) -> Result<usize, ExprError> {
        if self.is_op(op) {
            Ok(self.advance().2)
        } else {
            let (_, from, to) = self.peek().clone();
            Err(syntax(tr(format!("esperava `{op}`"), format!("expected `{op}`")), from, to.max(from + 1)))
        }
    }

    fn expression(&mut self) -> Result<Ast, ExprError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            let (_, from, to) = self.peek().clone();
            return Err(syntax(tr("expressão aninhada demais", "expression nested too deeply"), from, to));
        }
        let test = self.or()?;
        let result = if self.is_op("?") {
            self.advance();
            let then = self.expression()?;
            self.expect(":")?;
            let otherwise = self.expression()?;
            let (from, to) = (test.from, otherwise.to);
            Ast { kind: AstKind::Cond(Box::new(test), Box::new(then), Box::new(otherwise)), from, to }
        } else {
            test
        };
        self.depth -= 1;
        Ok(result)
    }

    fn binary(&mut self, op: BinaryOp, left: Ast, right: Ast) -> Ast {
        let (from, to) = (left.from, right.to);
        Ast { kind: AstKind::Binary(op, Box::new(left), Box::new(right)), from, to }
    }

    fn or(&mut self) -> Result<Ast, ExprError> {
        let mut left = self.and()?;
        while self.is_word("or") || self.is_op("||") {
            self.advance();
            let right = self.and()?;
            left = self.binary(BinaryOp::Or, left, right);
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Ast, ExprError> {
        let mut left = self.not()?;
        while self.is_word("and") || self.is_op("&&") {
            self.advance();
            let right = self.not()?;
            left = self.binary(BinaryOp::And, left, right);
        }
        Ok(left)
    }

    fn not(&mut self) -> Result<Ast, ExprError> {
        if self.is_word("not") || self.is_op("!") {
            let (_, from, _) = self.advance();
            let arg = self.not()?;
            let to = arg.to;
            return Ok(Ast { kind: AstKind::Not(Box::new(arg)), from, to });
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Ast, ExprError> {
        let left = self.range()?;
        let op = match &self.peek().0 {
            Token::Op("==") => BinaryOp::Eq,
            Token::Op("!=") => BinaryOp::Ne,
            Token::Op("<") => BinaryOp::Lt,
            Token::Op("<=") => BinaryOp::Le,
            Token::Op(">") => BinaryOp::Gt,
            Token::Op(">=") => BinaryOp::Ge,
            Token::Word(word) if word == "in" => BinaryOp::In,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.range()?;
        Ok(self.binary(op, left, right))
    }

    fn range(&mut self) -> Result<Ast, ExprError> {
        let left = self.additive()?;
        if self.is_op("..") {
            self.advance();
            let right = self.additive()?;
            let (from, to) = (left.from, right.to);
            return Ok(Ast { kind: AstKind::Range(Box::new(left), Box::new(right)), from, to });
        }
        Ok(left)
    }

    fn additive(&mut self) -> Result<Ast, ExprError> {
        let mut left = self.multiplicative()?;
        loop {
            let op = match &self.peek().0 {
                Token::Op("+") => BinaryOp::Add,
                Token::Op("-") => BinaryOp::Sub,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.multiplicative()?;
            left = self.binary(op, left, right);
        }
    }

    fn multiplicative(&mut self) -> Result<Ast, ExprError> {
        let mut left = self.unary()?;
        loop {
            let op = match &self.peek().0 {
                Token::Op("*") => BinaryOp::Mul,
                Token::Op("/") => BinaryOp::Div,
                Token::Op("%") => BinaryOp::Mod,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.unary()?;
            left = self.binary(op, left, right);
        }
    }

    fn unary(&mut self) -> Result<Ast, ExprError> {
        if self.is_op("-") {
            let (_, from, _) = self.advance();
            let arg = self.unary()?;
            let to = arg.to;
            return Ok(Ast { kind: AstKind::Neg(Box::new(arg)), from, to });
        }
        self.atom()
    }

    fn atom(&mut self) -> Result<Ast, ExprError> {
        let (token, from, to) = self.advance();
        let kind = match token {
            Token::Number(value) => AstKind::Number(value),
            Token::Text(value) => AstKind::Text(value),
            Token::Var(path) => AstKind::Var(path),
            Token::Word(word) => match word.as_str() {
                "true" => AstKind::Bool(true),
                "false" => AstKind::Bool(false),
                "and" | "or" | "not" => return Err(syntax(tr(format!("esperava um valor antes de `{word}`"), format!("expected a value before `{word}`")), from, to)),
                _ if self.is_op("(") => {
                    if !BUILTINS.contains(&word.as_str()) {
                        let pending = syntax(tr(format!("função desconhecida: `{word}`"), format!("unknown function: `{word}`")), from, to);
                        return Err(match crate::text::suggest(&word, BUILTINS.iter().copied()) {
                            Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                            None => pending.hint(tr(format!("Funções disponíveis: {}", BUILTINS.join(", ")), format!("Available functions: {}", BUILTINS.join(", ")))),
                        });
                    }
                    self.advance();
                    let mut args = Vec::new();
                    if !self.is_op(")") {
                        loop {
                            args.push(self.expression()?);
                            if self.is_op(",") {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    let end = self.expect(")")?;
                    return Ok(Ast { kind: AstKind::Call(word, args), from, to: end });
                }
                _ => AstKind::Text(word),
            },
            Token::Op("(") => {
                let inner = self.expression()?;
                let end = self.expect(")")?;
                return Ok(Ast { kind: inner.kind, from, to: end });
            }
            Token::Op("[") => {
                let mut items = Vec::new();
                if !self.is_op("]") {
                    loop {
                        items.push(self.expression()?);
                        if self.is_op(",") {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                let end = self.expect("]")?;
                return Ok(Ast { kind: AstKind::List(items), from, to: end });
            }
            Token::End => return Err(syntax(tr("a expressão terminou antes do esperado", "the expression ended too early"), from, to.max(from))),
            Token::Op(op) => return Err(syntax(tr(format!("`{op}` inesperado"), format!("unexpected `{op}`")), from, to)),
        };
        Ok(Ast { kind, from, to })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawAction {
    Set { name: String, name_from: usize, name_to: usize, value: Ast },
    Toggle { name: String, from: usize, to: usize },
    Add { name: String, from: usize, to: usize, value: Ast },
    Remove { name: String, from: usize, to: usize, value: Ast },
    Clear { name: String, from: usize, to: usize },
    Play { name: String, from: usize, to: usize },
    Stop { name: String, from: usize, to: usize },
    Go { target: String, from: usize, to: usize },
    Back,
    Next,
    Prev,
}

pub fn parse_actions(source: &str) -> Result<Vec<RawAction>, ExprError> {
    let mut actions = Vec::new();
    let mut start = 0;
    for (index, part) in split_actions(source) {
        start = index;
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let offset = index + (part.len() - part.trim_start().len());
        actions.push(action(trimmed, offset)?);
    }
    if actions.is_empty() {
        return Err(invalid_action(tr("nenhuma ação informada", "no action given"), start, source.len()).hint(action_hint()));
    }
    Ok(actions)
}

fn invalid_action(message: String, from: usize, to: usize) -> ExprError {
    ExprError::new("R611", message, from, to)
}

fn action_hint() -> String {
    tr(
        "Ações: $variavel = valor, toggle $variavel, add $lista valor, remove $lista valor, clear $lista, play som, stop som, go #id-da-pagina, go back, go next, go prev. Separe várias com `;`",
        "Actions: $variable = value, toggle $variable, add $list value, remove $list value, clear $list, play sound, stop sound, go #page-id, go back, go next, go prev. Separate several with `;`",
    )
}

fn split_actions(source: &str) -> Vec<(usize, &str)> {
    let mut parts = Vec::new();
    let mut quote: Option<char> = None;
    let mut start = 0;
    for (index, c) in source.char_indices() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == ';' => {
                parts.push((start, &source[start..index]));
                start = index + 1;
            }
            None => {}
        }
    }
    parts.push((start, &source[start..]));
    parts
}

fn action(text: &str, offset: usize) -> Result<RawAction, ExprError> {
    let end = offset + text.len();
    if let Some(rest) = text.strip_prefix("go ") {
        let target = rest.trim();
        let target_from = offset + text.len() - rest.trim_start().len();
        return match target {
            "back" => Ok(RawAction::Back),
            "next" => Ok(RawAction::Next),
            "prev" => Ok(RawAction::Prev),
            _ => match target.strip_prefix('#') {
                Some(id) if !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '$' | '.')) => {
                    Ok(RawAction::Go { target: id.to_string(), from: target_from, to: target_from + target.len() })
                }
                _ => Err(invalid_action(tr(format!("destino inválido: `{target}`"), format!("invalid destination: `{target}`")), target_from, end)
                    .hint(tr("Use go #id-da-pagina, go back, go next ou go prev", "Use go #page-id, go back, go next or go prev"))),
            },
        };
    }
    if let Some(rest) = text.strip_prefix("toggle ") {
        let name_from = offset + text.len() - rest.trim_start().len();
        let rest = rest.trim();
        return match rest.strip_prefix('$').and_then(|name| path_at(name, 0)) {
            Some((path, length)) if path.len() == 1 && length + 1 == rest.len() => Ok(RawAction::Toggle { name: path[0].clone(), from: name_from, to: name_from + rest.len() }),
            _ => Err(invalid_action(tr("`toggle` espera uma variável, como toggle $detalhes", "`toggle` expects a variable, such as toggle $details"), name_from, end)),
        };
    }
    for keyword in ["play", "stop"] {
        if let Some(rest) = text.strip_prefix(keyword).and_then(|rest| rest.strip_prefix(' ')) {
            let from = offset + text.len() - rest.trim_start().len();
            let name = rest.trim();
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
                return Err(invalid_action(tr(format!("`{keyword}` espera o nome de um som, como {keyword} porta"), format!("`{keyword}` expects a sound name, such as {keyword} door")), from, end));
            }
            let (name, to) = (name.to_string(), from + name.len());
            return Ok(if keyword == "play" { RawAction::Play { name, from, to } } else { RawAction::Stop { name, from, to } });
        }
    }
    for keyword in ["add", "remove"] {
        if let Some(rest) = text.strip_prefix(keyword).and_then(|rest| rest.strip_prefix(' ')) {
            return list_action(keyword, rest, text, offset);
        }
    }
    if let Some(rest) = text.strip_prefix("clear ") {
        let name_from = offset + text.len() - rest.trim_start().len();
        let rest = rest.trim();
        return match rest.strip_prefix('$').and_then(|name| path_at(name, 0)) {
            Some((path, length)) if path.len() == 1 && length + 1 == rest.len() => Ok(RawAction::Clear { name: path[0].clone(), from: name_from, to: name_from + rest.len() }),
            _ => Err(invalid_action(tr("`clear` espera uma lista, como clear $mochila", "`clear` expects a list, such as clear $backpack"), name_from, end)),
        };
    }
    if let Some(after) = text.strip_prefix('$')
        && let Some((path, length)) = path_at(after, 0) {
            let rest = &after[length..];
            let trimmed = rest.trim_start();
            if trimmed.starts_with('=') && !trimmed.starts_with("==") {
                if path.len() > 1 {
                    return Err(invalid_action(tr("só é possível alterar uma variável inteira, não um campo", "only a whole variable can be changed, not a field"), offset, offset + 1 + length));
                }
                let value_text = &trimmed[1..];
                let value_from = offset + 1 + length + (rest.len() - trimmed.len()) + 1;
                let shift = |mut error: ExprError| {
                    error.from += value_from;
                    error.to += value_from;
                    error
                };
                let mut value = parse(value_text).map_err(shift)?;
                relocate(&mut value, value_from);
                return Ok(RawAction::Set { name: path[0].clone(), name_from: offset, name_to: offset + 1 + length, value });
            }
        }
    Err(invalid_action(tr(format!("ação inválida: `{text}`"), format!("invalid action: `{text}`")), offset, end).hint(action_hint()))
}

fn list_action(keyword: &str, rest: &str, text: &str, offset: usize) -> Result<RawAction, ExprError> {
    let end = offset + text.len();
    let name_from = offset + text.len() - rest.trim_start().len();
    let rest = rest.trim_start();
    let expected = || {
        invalid_action(
            tr(format!("`{keyword}` espera uma lista e um valor, como {keyword} $mochila chave"), format!("`{keyword}` expects a list and a value, such as {keyword} $backpack key")),
            name_from,
            end,
        )
    };
    let Some((path, length)) = rest.strip_prefix('$').and_then(|name| path_at(name, 0)) else {
        return Err(expected());
    };
    let after = &rest[1 + length..];
    if path.len() != 1 || !after.starts_with(char::is_whitespace) || after.trim().is_empty() {
        return Err(expected());
    }
    let value_from = name_from + 1 + length + (after.len() - after.trim_start().len());
    let mut value = parse(after.trim()).map_err(|mut error| {
        error.from += value_from;
        error.to += value_from;
        error
    })?;
    relocate(&mut value, value_from);
    let (name, to) = (path[0].clone(), name_from + 1 + length);
    Ok(if keyword == "add" { RawAction::Add { name, from: name_from, to, value } } else { RawAction::Remove { name, from: name_from, to, value } })
}

pub fn relocate(ast: &mut Ast, by: usize) {
    ast.from += by;
    ast.to += by;
    match &mut ast.kind {
        AstKind::Not(inner) | AstKind::Neg(inner) => relocate(inner, by),
        AstKind::Binary(_, left, right) | AstKind::Range(left, right) => {
            relocate(left, by);
            relocate(right, by);
        }
        AstKind::Cond(test, then, otherwise) => {
            relocate(test, by);
            relocate(then, by);
            relocate(otherwise, by);
        }
        AstKind::List(items) | AstKind::Call(_, items) => items.iter_mut().for_each(|item| relocate(item, by)),
        _ => {}
    }
}
