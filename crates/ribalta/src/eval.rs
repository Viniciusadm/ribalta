use std::collections::BTreeMap;

use crate::expr::{Ast, AstKind, ExprError};
use crate::ir::{BinaryOp, Builtin, Expr};
use crate::text::{suggest, tr};
use crate::value::{Type, Value};

pub const MAX_ITEMS: usize = 500;

#[derive(Debug, Clone)]
pub enum Binding {
    Value(Value),
    State(Type),
    Step,
    Elapsed,
    Dynamic(Expr, Type),
    Record(String, BTreeMap<String, Type>),
}

#[derive(Debug, Clone)]
pub enum Folded {
    Static(Value),
    Dynamic(Expr, Type),
}

impl Folded {
    pub fn ty(&self) -> Type {
        match self {
            Folded::Static(value) => value.ty(),
            Folded::Dynamic(_, ty) => *ty,
        }
    }

    pub fn into_expr(self) -> Option<Expr> {
        match self {
            Folded::Static(value) => value.scalar().map(|value| Expr::Lit { value }),
            Folded::Dynamic(expr, _) => Some(expr),
        }
    }
}

pub trait Scope {
    fn binding(&mut self, name: &str) -> Option<Binding>;
    fn names(&self) -> Vec<String>;
    fn note_step(&mut self, _step: u32) {}
    fn note_comparison(&mut self, _state: &str, _value: &str, _from: usize, _to: usize) {}
    fn allows_random(&self) -> bool {
        false
    }
    fn note_random(&mut self) {}
    fn note_elapsed(&mut self) {}
    fn allows_items(&self) -> bool {
        true
    }
}

fn item_in_action(name: &str, ast: &Ast) -> ExprError {
    ExprError::new(
        "R620",
        tr(format!("ações não podem usar `${name}`, que só existe enquanto a repetição é desenhada"), format!("actions cannot use `${name}`, which only exists while the repetition is drawn")),
        ast.from,
        ast.to,
    )
    .hint(tr("Use o item no texto, em `if` ou em variantes; a ação precisa de valores do @state", "Use the item in text, `if` or variants; the action needs @state values"))
}

fn mismatch(message: String, ast: &Ast) -> ExprError {
    ExprError::new("R602", message, ast.from, ast.to)
}

fn expect_type(folded: &Folded, ty: Type, ast: &Ast) -> Result<(), ExprError> {
    if folded.ty() == ty {
        Ok(())
    } else {
        Err(mismatch(tr(format!("esperava {}, mas o valor é {}", ty.label(), folded.ty().label()), format!("expected {}, but the value is {}", ty.label(), folded.ty().label())), ast))
    }
}

fn to_expr(folded: Folded, ast: &Ast) -> Result<Expr, ExprError> {
    match folded {
        Folded::Static(value) => match value.scalar() {
            Some(value) => Ok(Expr::Lit { value }),
            None => Err(mismatch(tr("listas e registros não podem depender do estado do documento", "lists and records cannot depend on the document state"), ast)),
        },
        Folded::Dynamic(expr, _) => Ok(expr),
    }
}

pub fn fold(scope: &mut dyn Scope, ast: &Ast) -> Result<Folded, ExprError> {
    match &ast.kind {
        AstKind::Number(value) => Ok(Folded::Static(Value::Number(*value))),
        AstKind::Text(value) => Ok(Folded::Static(Value::Text(value.clone()))),
        AstKind::Bool(value) => Ok(Folded::Static(Value::Bool(*value))),
        AstKind::Var(path) => variable(scope, path, ast),
        AstKind::Not(inner) => {
            let value = fold(scope, inner)?;
            expect_type(&value, Type::Bool, inner)?;
            Ok(match value {
                Folded::Static(Value::Bool(value)) => Folded::Static(Value::Bool(!value)),
                other => Folded::Dynamic(Expr::Not { arg: Box::new(to_expr(other, inner)?) }, Type::Bool),
            })
        }
        AstKind::Neg(inner) => {
            let value = fold(scope, inner)?;
            expect_type(&value, Type::Number, inner)?;
            Ok(match value {
                Folded::Static(Value::Number(value)) => Folded::Static(Value::Number(-value)),
                other => Folded::Dynamic(Expr::Neg { arg: Box::new(to_expr(other, inner)?) }, Type::Number),
            })
        }
        AstKind::Binary(op, left, right) => binary(scope, *op, left, right, ast),
        AstKind::Range(start, end) => {
            let (Folded::Static(Value::Number(first)), Folded::Static(Value::Number(last))) = (fold(scope, start)?, fold(scope, end)?) else {
                return Err(mismatch(tr("um intervalo precisa de dois números fixos, como 1..5", "a range needs two fixed numbers, such as 1..5"), ast));
            };
            if first.fract() != 0.0 || last.fract() != 0.0 {
                return Err(mismatch(tr("um intervalo usa números inteiros", "a range uses whole numbers"), ast));
            }
            let count = (last - first).abs() as usize + 1;
            if count > MAX_ITEMS {
                return Err(ExprError::new("R606", tr(format!("intervalo grande demais ({count} itens; limite: {MAX_ITEMS})"), format!("range too large ({count} items; limit: {MAX_ITEMS})")), ast.from, ast.to));
            }
            let step = if last >= first { 1.0 } else { -1.0 };
            Ok(Folded::Static(Value::List((0..count).map(|index| Value::Number(first + step * index as f64)).collect())))
        }
        AstKind::Cond(test, then, otherwise) => {
            let condition = fold(scope, test)?;
            expect_type(&condition, Type::Bool, test)?;
            let then_value = fold(scope, then)?;
            let otherwise_value = fold(scope, otherwise)?;
            if let Folded::Static(Value::Bool(choice)) = condition {
                return Ok(if choice { then_value } else { otherwise_value });
            }
            if let (Folded::Static(a), Folded::Static(b)) = (&then_value, &otherwise_value)
                && a == b {
                    return Ok(then_value);
                }
            if then_value.ty() != otherwise_value.ty() {
                return Err(mismatch(
                    tr(format!("os dois lados de `? :` precisam ter o mesmo tipo ({} e {})", then_value.ty().label(), otherwise_value.ty().label()), format!("both sides of `? :` must have the same type ({} and {})", then_value.ty().label(), otherwise_value.ty().label())),
                    ast,
                ));
            }
            let ty = then_value.ty();
            Ok(Folded::Dynamic(
                Expr::Cond { test: Box::new(to_expr(condition, test)?), then: Box::new(to_expr(then_value, then)?), otherwise: Box::new(to_expr(otherwise_value, otherwise)?) },
                ty,
            ))
        }
        AstKind::List(items) => {
            let mut values = Vec::new();
            for item in items {
                match fold(scope, item)? {
                    Folded::Static(value) => values.push(value),
                    Folded::Dynamic(..) => return Err(dynamic_not_allowed(item)),
                }
            }
            Ok(Folded::Static(Value::List(values)))
        }
        AstKind::Call(name, args) => call(scope, name, args, ast),
    }
}

pub fn dynamic_not_allowed(ast: &Ast) -> ExprError {
    ExprError::new("R604", tr("este valor depende do estado do documento e não pode ser usado aqui", "this value depends on the document state and cannot be used here"), ast.from, ast.to)
        .hint(tr("Para mudar o visual conforme o estado, use uma variante: {.destaque=$ativo}", "To change the look based on state, use a variant: {.highlight=$active}"))
}

fn variable(scope: &mut dyn Scope, path: &[String], ast: &Ast) -> Result<Folded, ExprError> {
    let name = &path[0];
    let Some(binding) = scope.binding(name) else {
        let names = scope.names();
        let pending = ExprError::new("R601", tr(format!("variável desconhecida: `${name}`"), format!("unknown variable: `${name}`")), ast.from, ast.from + name.len() + 1);
        return Err(match suggest(name, names.iter().map(String::as_str)) {
            Some(found) => pending.hint(tr(format!("Você quis dizer `${found}`?"), format!("Did you mean `${found}`?"))),
            None => pending.hint(tr("Declare-a com @let, @data ou @state no início do documento", "Declare it with @let, @data or @state at the start of the document")),
        });
    };
    let is_item = matches!(&binding, Binding::Record(..) | Binding::Dynamic(Expr::Item { .. }, _));
    if is_item && !scope.allows_items() {
        return Err(item_in_action(name, ast));
    }
    let (expr, ty) = match binding {
        Binding::Value(value) => return field_path(value, path, ast).map(Folded::Static),
        Binding::Record(item, fields) => {
            let Some(field) = path.get(1) else {
                return Err(mismatch(tr(format!("`${name}` é um registro; use um campo, como `${name}.{}`", fields.keys().next().cloned().unwrap_or_default()), format!("`${name}` is a record; use a field, such as `${name}.{}`", fields.keys().next().cloned().unwrap_or_default())), ast));
            };
            if path.len() > 2 {
                return Err(mismatch(tr(format!("`${name}.{field}` não tem campos"), format!("`${name}.{field}` has no fields")), ast));
            }
            let Some(ty) = fields.get(field).copied() else {
                let pending = ExprError::new("R601", tr(format!("`${name}` não tem o campo `{field}`"), format!("`${name}` has no field `{field}`")), ast.from, ast.to);
                let keys: Vec<String> = fields.keys().cloned().collect();
                return Err(match suggest(field, keys.iter().map(String::as_str)) {
                    Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                    None => pending.hint(tr(format!("Campos disponíveis: {}", keys.join(", ")), format!("Available fields: {}", keys.join(", ")))),
                });
            };
            return Ok(Folded::Dynamic(Expr::Item { name: item, field: Some(field.clone()) }, ty));
        }
        Binding::State(ty) => (Expr::State { name: name.clone() }, ty),
        Binding::Step => (Expr::Step, Type::Number),
        Binding::Elapsed => {
            scope.note_elapsed();
            (Expr::Elapsed, Type::Number)
        }
        Binding::Dynamic(expr, ty) => (expr, ty),
    };
    if path.len() > 1 {
        return Err(mismatch(tr(format!("`${name}` não tem campos"), format!("`${name}` has no fields")), ast));
    }
    Ok(Folded::Dynamic(expr, ty))
}

pub fn field_path(mut value: Value, path: &[String], ast: &Ast) -> Result<Value, ExprError> {
    for (index, field) in path.iter().enumerate().skip(1) {
        let shown = path[..index].join(".");
        value = field_of(value, field, &shown, ast)?;
    }
    Ok(value)
}

fn field_of(value: Value, field: &str, shown: &str, ast: &Ast) -> Result<Value, ExprError> {
    match value {
        Value::Record(record) => record.get(field).cloned().ok_or_else(|| {
            let pending = ExprError::new("R601", tr(format!("`${shown}` não tem o campo `{field}`"), format!("`${shown}` has no field `{field}`")), ast.from, ast.to);
            match suggest(field, record.keys().map(String::as_str)) {
                Some(found) => pending.hint(tr(format!("Você quis dizer `{found}`?"), format!("Did you mean `{found}`?"))),
                None => pending.hint(tr(format!("Campos disponíveis: {}", record.keys().cloned().collect::<Vec<_>>().join(", ")), format!("Available fields: {}", record.keys().cloned().collect::<Vec<_>>().join(", ")))),
            }
        }),
        Value::List(items) => items.into_iter().map(|item| field_of(item, field, shown, ast)).collect::<Result<Vec<_>, _>>().map(Value::List),
        other => Err(mismatch(tr(format!("`${shown}` é {} e não tem campos", other.ty().label()), format!("`${shown}` is {} and has no fields", other.ty().label())), ast)),
    }
}

fn binary(scope: &mut dyn Scope, op: BinaryOp, left_ast: &Ast, right_ast: &Ast, ast: &Ast) -> Result<Folded, ExprError> {
    let left = fold(scope, left_ast)?;
    let right = fold(scope, right_ast)?;
    match op {
        BinaryOp::And | BinaryOp::Or => {
            expect_type(&left, Type::Bool, left_ast)?;
            expect_type(&right, Type::Bool, right_ast)?;
            let absorbing = op == BinaryOp::Or;
            match (left, right) {
                (Folded::Static(Value::Bool(a)), Folded::Static(Value::Bool(b))) => Ok(Folded::Static(Value::Bool(if absorbing { a || b } else { a && b }))),
                (Folded::Static(Value::Bool(a)), other) | (other, Folded::Static(Value::Bool(a))) => {
                    Ok(if a == absorbing { Folded::Static(Value::Bool(a)) } else { other })
                }
                (a, b) => Ok(Folded::Dynamic(Expr::Binary { operator: op, left: Box::new(to_expr(a, left_ast)?), right: Box::new(to_expr(b, right_ast)?) }, Type::Bool)),
            }
        }
        BinaryOp::In => membership(scope, left, right, left_ast, right_ast),
        BinaryOp::Eq | BinaryOp::Ne => {
            if left.ty() != right.ty() {
                return Err(mismatch(tr(format!("não é possível comparar {} com {}", left.ty().label(), right.ty().label()), format!("cannot compare {} with {}", left.ty().label(), right.ty().label())), ast));
            }
            if left.ty() == Type::List && (matches!(left, Folded::Dynamic(..)) || matches!(right, Folded::Dynamic(..))) {
                return Err(mismatch(
                    tr("listas do estado não podem ser comparadas com `==`; use `valor in $lista` ou `len($lista)`", "state lists cannot be compared with `==`; use `value in $list` or `len($list)`"),
                    ast,
                ));
            }
            note(scope, &left, &right, right_ast);
            note(scope, &right, &left, left_ast);
            note_step(scope, &left, &right);
            note_step(scope, &right, &left);
            match (left, right) {
                (Folded::Static(a), Folded::Static(b)) => Ok(Folded::Static(Value::Bool((a == b) == (op == BinaryOp::Eq)))),
                (a, b) => Ok(Folded::Dynamic(Expr::Binary { operator: op, left: Box::new(to_expr(a, left_ast)?), right: Box::new(to_expr(b, right_ast)?) }, Type::Bool)),
            }
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let ordered = left.ty() == right.ty() && matches!(left.ty(), Type::Number | Type::Text);
            if !ordered {
                return Err(mismatch(tr(format!("não é possível comparar {} com {}", left.ty().label(), right.ty().label()), format!("cannot compare {} with {}", left.ty().label(), right.ty().label())), ast));
            }
            note_step(scope, &left, &right);
            note_step(scope, &right, &left);
            match (left, right) {
                (Folded::Static(a), Folded::Static(b)) => {
                    let ordering = match (&a, &b) {
                        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y),
                        (Value::Text(x), Value::Text(y)) => Some(x.cmp(y)),
                        _ => None,
                    };
                    let result = ordering.is_some_and(|ordering| match op {
                        BinaryOp::Lt => ordering.is_lt(),
                        BinaryOp::Le => ordering.is_le(),
                        BinaryOp::Gt => ordering.is_gt(),
                        _ => ordering.is_ge(),
                    });
                    Ok(Folded::Static(Value::Bool(result)))
                }
                (a, b) => Ok(Folded::Dynamic(Expr::Binary { operator: op, left: Box::new(to_expr(a, left_ast)?), right: Box::new(to_expr(b, right_ast)?) }, Type::Bool)),
            }
        }
        BinaryOp::Add if left.ty() == Type::Text || right.ty() == Type::Text => {
            for (side, side_ast) in [(&left, left_ast), (&right, right_ast)] {
                if !side.ty().is_scalar() {
                    return Err(mismatch(tr(format!("não é possível juntar {} a um texto", side.ty().label()), format!("cannot join {} to a text", side.ty().label())), side_ast));
                }
            }
            match (left, right) {
                (Folded::Static(a), Folded::Static(b)) => Ok(Folded::Static(Value::Text(format!("{}{}", a.display().unwrap_or_default(), b.display().unwrap_or_default())))),
                (a, b) => Ok(Folded::Dynamic(Expr::Binary { operator: op, left: Box::new(to_expr(a, left_ast)?), right: Box::new(to_expr(b, right_ast)?) }, Type::Text)),
            }
        }
        _ => {
            expect_type(&left, Type::Number, left_ast)?;
            expect_type(&right, Type::Number, right_ast)?;
            match (left, right) {
                (Folded::Static(Value::Number(a)), Folded::Static(Value::Number(b))) => {
                    if matches!(op, BinaryOp::Div | BinaryOp::Mod) && b == 0.0 {
                        return Err(mismatch(tr("divisão por zero", "division by zero"), right_ast));
                    }
                    Ok(Folded::Static(Value::Number(arithmetic(op, a, b))))
                }
                (a, b) => Ok(Folded::Dynamic(Expr::Binary { operator: op, left: Box::new(to_expr(a, left_ast)?), right: Box::new(to_expr(b, right_ast)?) }, Type::Number)),
            }
        }
    }
}

fn membership(scope: &mut dyn Scope, left: Folded, right: Folded, left_ast: &Ast, right_ast: &Ast) -> Result<Folded, ExprError> {
    if right.ty() != Type::List {
        return Err(mismatch(
            tr(format!("`in` procura um valor numa lista, mas o lado direito é {}", right.ty().label()), format!("`in` looks for a value in a list, but the right side is {}", right.ty().label())),
            right_ast,
        ));
    }
    if !left.ty().is_scalar() {
        return Err(mismatch(tr(format!("`in` procura um único valor, não {}", left.ty().label()), format!("`in` looks for a single value, not {}", left.ty().label())), left_ast));
    }
    match (left, right) {
        (Folded::Static(value), Folded::Static(Value::List(items))) => Ok(Folded::Static(Value::Bool(items.contains(&value)))),
        (left, Folded::Static(Value::List(items))) => {
            let mut alternatives: Option<Expr> = None;
            for item in items.iter().filter(|item| item.ty() == left.ty()) {
                note(scope, &left, &Folded::Static(item.clone()), right_ast);
                let Some(value) = item.scalar() else { continue };
                let test = Expr::Binary { operator: BinaryOp::Eq, left: Box::new(to_expr(left.clone(), left_ast)?), right: Box::new(Expr::Lit { value }) };
                alternatives = Some(match alternatives {
                    Some(previous) => Expr::Binary { operator: BinaryOp::Or, left: Box::new(previous), right: Box::new(test) },
                    None => test,
                });
            }
            Ok(alternatives.map_or(Folded::Static(Value::Bool(false)), |expr| Folded::Dynamic(expr, Type::Bool)))
        }
        (left, Folded::Dynamic(list, _)) => {
            if left.ty() != Type::Text {
                return Err(mismatch(
                    tr(format!("listas do estado guardam textos, mas o valor procurado é {}", left.ty().label()), format!("state lists hold text, but the value looked for is {}", left.ty().label())),
                    left_ast,
                ));
            }
            if let (Folded::Static(Value::Text(value)), Expr::State { name }) = (&left, &list) {
                scope.note_comparison(name, value, left_ast.from, left_ast.to);
            }
            Ok(Folded::Dynamic(Expr::Binary { operator: BinaryOp::In, left: Box::new(to_expr(left, left_ast)?), right: Box::new(list) }, Type::Bool))
        }
        (_, Folded::Static(_)) => unreachable!("right side is a list"),
    }
}

fn note(scope: &mut dyn Scope, state: &Folded, other: &Folded, other_ast: &Ast) {
    if let (Folded::Dynamic(Expr::State { name }, Type::Text), Folded::Static(Value::Text(value))) = (state, other) {
        scope.note_comparison(name, value, other_ast.from, other_ast.to);
    }
}

fn note_step(scope: &mut dyn Scope, step: &Folded, other: &Folded) {
    if let (Folded::Dynamic(Expr::Step, _), Folded::Static(Value::Number(value))) = (step, other)
        && *value >= 0.0
        && *value <= 99.0 {
            scope.note_step(*value as u32);
        }
}

pub fn arithmetic(op: BinaryOp, a: f64, b: f64) -> f64 {
    match op {
        BinaryOp::Add => a + b,
        BinaryOp::Sub => a - b,
        BinaryOp::Mul => a * b,
        BinaryOp::Div => {
            if b == 0.0 {
                0.0
            } else {
                a / b
            }
        }
        BinaryOp::Mod => {
            if b == 0.0 {
                0.0
            } else {
                a % b
            }
        }
        _ => 0.0,
    }
}

fn numbers(value: &Value, ast: &Ast) -> Result<Vec<f64>, ExprError> {
    match value {
        Value::List(items) => items
            .iter()
            .map(|item| match item {
                Value::Number(number) => Ok(*number),
                other => Err(mismatch(tr(format!("a lista tem um valor que não é número ({})", other.ty().label()), format!("the list has a value that is not a number ({})", other.ty().label())), ast)),
            })
            .collect(),
        Value::Number(number) => Ok(vec![*number]),
        other => Err(mismatch(tr(format!("esperava uma lista de números, mas o valor é {}", other.ty().label()), format!("expected a list of numbers, but the value is {}", other.ty().label())), ast)),
    }
}

fn round_to(value: f64, digits: f64) -> f64 {
    let factor = 10f64.powi(digits.clamp(0.0, 10.0) as i32);
    (value * factor).round() / factor
}

fn call(scope: &mut dyn Scope, name: &str, args: &[Ast], ast: &Ast) -> Result<Folded, ExprError> {
    let mut folded = Vec::new();
    for arg in args {
        folded.push(fold(scope, arg)?);
    }
    let arity = |expected: &str| ExprError::new("R603", tr(format!("`{name}` espera {expected}"), format!("`{name}` expects {expected}")), ast.from, ast.to);
    if matches!(name, "random" | "pick") {
        if !scope.allows_random() {
            return Err(ExprError::new(
                "R613",
                tr(format!("`{name}` só pode ser usado em ações (`click` e `on-enter`)"), format!("`{name}` can only be used in actions (`click` and `on-enter`)")),
                ast.from,
                ast.to,
            )
            .hint(tr("Guarde o sorteio num @state e mostre a variável: click=\"$dado = random(1, 6)\" e $dado no texto", "Store the draw in a @state and show the variable: click=\"$die = random(1, 6)\" and $die in the text")));
        }
        scope.note_random();
    }
    match name {
        "random" => {
            let [low, high] = folded.as_slice() else { return Err(arity(&tr("o menor e o maior número", "the lowest and the highest number"))) };
            expect_type(low, Type::Number, &args[0])?;
            expect_type(high, Type::Number, &args[1])?;
            let exprs = vec![to_expr(low.clone(), &args[0])?, to_expr(high.clone(), &args[1])?];
            Ok(Folded::Dynamic(Expr::Call { name: Builtin::Random, args: exprs }, Type::Number))
        }
        "pick" => {
            let [list] = folded.as_slice() else { return Err(arity(&tr("uma lista", "one list"))) };
            match list {
                Folded::Static(Value::List(items)) => {
                    let Some(first) = items.first() else {
                        return Err(mismatch(tr("`pick` precisa de uma lista com itens", "`pick` needs a list with items"), &args[0]));
                    };
                    let ty = first.ty();
                    let mut exprs = Vec::new();
                    for item in items {
                        match item.scalar() {
                            Some(value) if item.ty() == ty => exprs.push(Expr::Lit { value }),
                            _ => return Err(mismatch(tr("`pick` sorteia de uma lista de valores do mesmo tipo", "`pick` draws from a list of values of the same type"), &args[0])),
                        }
                    }
                    Ok(Folded::Dynamic(Expr::Call { name: Builtin::Pick, args: exprs }, ty))
                }
                Folded::Dynamic(expr, Type::List) => Ok(Folded::Dynamic(Expr::Call { name: Builtin::Pick, args: vec![expr.clone()] }, Type::Text)),
                other => Err(mismatch(tr(format!("`pick` espera uma lista, mas o valor é {}", other.ty().label()), format!("`pick` expects a list, but the value is {}", other.ty().label())), &args[0])),
            }
        }
        "len" => {
            let [value] = folded.as_slice() else { return Err(arity(&tr("um valor", "one value"))) };
            match value {
                Folded::Static(Value::List(items)) => Ok(Folded::Static(Value::Number(items.len() as f64))),
                Folded::Static(Value::Text(text)) => Ok(Folded::Static(Value::Number(text.chars().count() as f64))),
                Folded::Dynamic(expr, Type::List | Type::Text | Type::Records) => Ok(Folded::Dynamic(Expr::Call { name: Builtin::Len, args: vec![expr.clone()] }, Type::Number)),
                other => Err(mismatch(tr(format!("`len` espera uma lista ou texto, mas o valor é {}", other.ty().label()), format!("`len` expects a list or text, but the value is {}", other.ty().label())), &args[0])),
            }
        }
        "sum" => {
            let [value] = folded.as_slice() else { return Err(arity(&tr("uma lista", "one list"))) };
            match value {
                Folded::Static(value) => Ok(Folded::Static(Value::Number(numbers(value, &args[0])?.iter().sum()))),
                Folded::Dynamic(..) => Err(dynamic_not_allowed(&args[0])),
            }
        }
        "min" | "max" => {
            if folded.is_empty() {
                return Err(arity(&tr("uma lista ou dois números", "a list or two numbers")));
            }
            let pick = |a: f64, b: f64| if name == "min" { a.min(b) } else { a.max(b) };
            if let [Folded::Static(value)] = folded.as_slice() {
                let values = numbers(value, &args[0])?;
                return match values.into_iter().reduce(pick) {
                    Some(result) => Ok(Folded::Static(Value::Number(result))),
                    None => Err(mismatch(tr(format!("`{name}` recebeu uma lista vazia"), format!("`{name}` got an empty list")), ast)),
                };
            }
            for (value, arg) in folded.iter().zip(args) {
                expect_type(value, Type::Number, arg)?;
            }
            if folded.iter().all(|value| matches!(value, Folded::Static(_))) {
                let values: Vec<f64> = folded.iter().filter_map(|value| match value {
                    Folded::Static(Value::Number(number)) => Some(*number),
                    _ => None,
                }).collect();
                return Ok(Folded::Static(Value::Number(values.into_iter().reduce(pick).unwrap_or(0.0))));
            }
            let exprs = folded.into_iter().zip(args).map(|(value, arg)| to_expr(value, arg)).collect::<Result<Vec<_>, _>>()?;
            let builtin = if name == "min" { Builtin::Min } else { Builtin::Max };
            Ok(Folded::Dynamic(Expr::Call { name: builtin, args: exprs }, Type::Number))
        }
        "round" => {
            if folded.is_empty() || folded.len() > 2 {
                return Err(arity(&tr("um número e, opcionalmente, as casas decimais", "a number and, optionally, the decimal places")));
            }
            for (value, arg) in folded.iter().zip(args) {
                expect_type(value, Type::Number, arg)?;
            }
            match folded.as_slice() {
                [Folded::Static(Value::Number(value))] => Ok(Folded::Static(Value::Number(round_to(*value, 0.0)))),
                [Folded::Static(Value::Number(value)), Folded::Static(Value::Number(digits))] => Ok(Folded::Static(Value::Number(round_to(*value, *digits)))),
                _ => {
                    let exprs = folded.into_iter().zip(args).map(|(value, arg)| to_expr(value, arg)).collect::<Result<Vec<_>, _>>()?;
                    Ok(Folded::Dynamic(Expr::Call { name: Builtin::Round, args: exprs }, Type::Number))
                }
            }
        }
        _ => Err(ExprError::new("R603", tr(format!("função desconhecida: `{name}`"), format!("unknown function: `{name}`")), ast.from, ast.to)),
    }
}
