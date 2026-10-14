use serde_json::Value;

use crate::document::is_separator;
use crate::text::Locale;
use crate::{Compilation, compile_localized};

const POSITION_KEYS: [&str; 3] = ["line", "endLine", "source"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
    Normal,
    Fence,
    Notes,
}

#[derive(Debug)]
enum Entry<'a> {
    Structural(String),
    Content { text: &'a str, region: Region },
}

pub fn format_source(source: &str) -> String {
    let entries = classify(source);
    let dedented = dedent_segments(&entries);
    let mut output: Vec<String> = Vec::with_capacity(dedented.len());
    let mut previous_blank = true;
    for (line, region) in dedented {
        let blank = line.trim().is_empty();
        if region == Region::Normal && blank {
            if !previous_blank {
                output.push(String::new());
            }
            previous_blank = true;
            continue;
        }
        previous_blank = blank && region == Region::Normal;
        output.push(line);
    }
    while output.last().is_some_and(|line| line.trim().is_empty()) {
        output.pop();
    }
    if output.is_empty() {
        return String::new();
    }
    let mut text = output.join("\n");
    text.push('\n');
    text
}

pub fn same_result(prelude: &str, before: &str, after: &str) -> bool {
    let left = compile_localized(prelude, before, Locale::Pt);
    let right = compile_localized(prelude, after, Locale::Pt);
    comparable_document(&left) == comparable_document(&right) && comparable_diagnostics(&left) == comparable_diagnostics(&right)
}

fn classify(source: &str) -> Vec<Entry<'_>> {
    let mut entries = Vec::new();
    let mut region = Region::Normal;
    for text in source.lines() {
        let trimmed = text.trim();
        match region {
            Region::Fence => {
                if trimmed.starts_with("```") {
                    region = Region::Normal;
                    entries.push(Entry::Content { text, region: Region::Normal });
                } else {
                    entries.push(Entry::Content { text, region: Region::Fence });
                }
            }
            Region::Notes => {
                if trimmed == ":::" {
                    region = Region::Normal;
                    entries.push(Entry::Structural(":::".to_owned()));
                } else {
                    entries.push(Entry::Content { text: trimmed, region: Region::Notes });
                }
            }
            Region::Normal => {
                if trimmed.starts_with("```") {
                    region = Region::Fence;
                    entries.push(Entry::Content { text, region: Region::Normal });
                } else if let Some(line) = structural(trimmed) {
                    if opens_notes(&line) {
                        region = Region::Notes;
                    }
                    entries.push(Entry::Structural(line));
                } else {
                    entries.push(Entry::Content { text, region: Region::Normal });
                }
            }
        }
    }
    entries
}

fn structural(trimmed: &str) -> Option<String> {
    if is_separator(trimmed) {
        let attrs = trimmed.trim_start_matches('-').trim();
        return Some(if attrs.is_empty() { "---".to_owned() } else { format!("--- {attrs}") });
    }
    if trimmed.starts_with('@') {
        return Some(trimmed.to_owned());
    }
    let colons = if trimmed.starts_with(":::") { 3 } else if trimmed.starts_with("::") { 2 } else { return None };
    let rest = &trimmed[colons..];
    if rest.starts_with(':') {
        return None;
    }
    let rest = rest.trim();
    if rest.is_empty() {
        return (colons == 3).then(|| ":::".to_owned());
    }
    Some(format!("{} {rest}", &trimmed[..colons]))
}

fn opens_notes(line: &str) -> bool {
    line.strip_prefix("::: ").is_some_and(|rest| {
        let name_end = rest.find(|char: char| char.is_whitespace() || char == '{').unwrap_or(rest.len());
        &rest[..name_end] == "notes"
    })
}

fn dedent_segments(entries: &[Entry<'_>]) -> Vec<(String, Region)> {
    let mut output = Vec::with_capacity(entries.len());
    let mut index = 0;
    while index < entries.len() {
        match &entries[index] {
            Entry::Structural(line) => {
                output.push((line.clone(), Region::Normal));
                index += 1;
            }
            Entry::Content { text, region: Region::Normal } if text.trim().is_empty() => {
                output.push((String::new(), Region::Normal));
                index += 1;
            }
            Entry::Content { .. } => {
                let start = index;
                while index < entries.len() && !ends_segment(&entries[index]) {
                    index += 1;
                }
                output.extend(dedent(&entries[start..index]));
            }
        }
    }
    output
}

fn ends_segment(entry: &Entry<'_>) -> bool {
    match entry {
        Entry::Structural(_) => true,
        Entry::Content { text, region } => *region == Region::Normal && text.trim().is_empty(),
    }
}

fn dedent(segment: &[Entry<'_>]) -> Vec<(String, Region)> {
    let lines: Vec<(&str, Region)> = segment
        .iter()
        .map(|entry| match entry {
            Entry::Content { text, region } => (*text, *region),
            Entry::Structural(_) => unreachable!("segments only hold content"),
        })
        .collect();
    let margin = lines
        .iter()
        .filter(|(text, _)| !text.trim().is_empty())
        .map(|(text, _)| indent(text))
        .min()
        .unwrap_or(0);
    lines
        .into_iter()
        .map(|(text, region)| {
            let shifted = &text[indent(text).min(margin)..];
            let line = if region == Region::Fence { shifted.to_owned() } else { shifted.trim_end().to_owned() };
            (line, region)
        })
        .collect()
}

fn indent(text: &str) -> usize {
    text.len() - text.trim_start().len()
}

fn comparable_document(compilation: &Compilation) -> Value {
    let mut value = serde_json::to_value(&compilation.document).expect("document serializes to JSON");
    strip_positions(&mut value);
    value
}

fn strip_positions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for key in POSITION_KEYS {
                map.remove(key);
            }
            map.values_mut().for_each(strip_positions);
        }
        Value::Array(items) => items.iter_mut().for_each(strip_positions),
        _ => {}
    }
}

fn comparable_diagnostics(compilation: &Compilation) -> Vec<(String, &'static str, String, Option<String>)> {
    let mut items: Vec<_> = compilation
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let severity = serde_json::to_value(diagnostic.severity).expect("severity serializes").to_string();
            (severity, diagnostic.code, diagnostic.message.clone(), diagnostic.hint.clone())
        })
        .collect();
    items.sort();
    items
}
