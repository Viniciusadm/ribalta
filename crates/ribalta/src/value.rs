use crate::ir::Scalar;
use crate::text::tr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<Value>),
    Record(Record),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Record(Vec<(String, Value)>);

impl Record {
    pub fn insert(&mut self, key: String, value: Value) {
        match self.0.iter_mut().find(|(existing, _)| *existing == key) {
            Some(entry) => entry.1 = value,
            None => self.0.push((key, value)),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(existing, _)| existing == key).map(|(_, value)| value)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.iter().map(|(key, _)| key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    Number,
    Text,
    List,
    Record,
    Records,
}

impl Type {
    pub fn label(self) -> String {
        match self {
            Type::Bool => tr("verdadeiro/falso", "true/false"),
            Type::Number => tr("número", "number"),
            Type::Text => tr("texto", "text"),
            Type::List => tr("lista", "list"),
            Type::Records => tr("lista de registros", "list of records"),
            Type::Record => tr("registro", "record"),
        }
    }

    pub fn is_scalar(self) -> bool {
        matches!(self, Type::Bool | Type::Number | Type::Text)
    }
}

impl Value {
    pub fn ty(&self) -> Type {
        match self {
            Value::Bool(_) => Type::Bool,
            Value::Number(_) => Type::Number,
            Value::Text(_) => Type::Text,
            Value::List(_) => Type::List,
            Value::Record(_) => Type::Record,
        }
    }

    pub fn display(&self) -> Option<String> {
        match self {
            Value::Bool(value) => Some(value.to_string()),
            Value::Number(value) => Some(format_number(*value)),
            Value::Text(value) => Some(value.clone()),
            Value::List(items) => items.iter().map(Value::display).collect::<Option<Vec<_>>>().map(|parts| parts.join(", ")),
            Value::Record(_) => None,
        }
    }

    pub fn scalar(&self) -> Option<Scalar> {
        match self {
            Value::Bool(value) => Some(Scalar::Bool(*value)),
            Value::Number(value) => Some(Scalar::Number(*value)),
            Value::Text(value) => Some(Scalar::Text(value.clone())),
            _ => None,
        }
    }

    pub fn from_cell(raw: &str) -> Value {
        match raw.trim() {
            "true" | "yes" | "sim" => Value::Bool(true),
            "false" | "no" | "não" => Value::Bool(false),
            other => Value::Text(other.to_string()),
        }
    }
}

pub fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

pub fn parse_number(value: &str) -> Option<f64> {
    let cleaned: String = value.trim().trim_end_matches('%').chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.replace(',', ".").parse::<f64>().ok().filter(|n| n.is_finite())
}
