use std::collections::{BTreeSet, VecDeque};

use serde::Serialize;
use serde_json::Value;

use crate::Outcome;
use crate::args::{GraphArgs, GraphFormat, Lang};
use crate::source::{display_name, read_source};
use crate::diagnostics::{self, Diagnostic, Severity};
use crate::error::{CliError, CliResult};
use crate::output::print_json;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub title: String,
    pub line: u64,
    pub hidden: bool,
    pub reachable: bool,
    pub ending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: &'static str,
    pub conditional: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Anchor {
    pub id: String,
    pub page: String,
}

#[derive(Debug, Serialize)]
pub struct Graph {
    pub pages: Vec<Node>,
    pub edges: Vec<Edge>,
    pub anchors: Vec<Anchor>,
}

pub fn run(args: &GraphArgs, lang: Lang) -> CliResult<Outcome> {
    let source = read_source(&args.file)?;
    let theme = args.theme.as_deref().map(read_source).transpose()?.unwrap_or_default();
    let compilation = ribalta::compile_localized(&theme, &source, lang.locale());
    let found: Vec<Diagnostic> = compilation.diagnostics.iter().map(Into::into).collect();
    if found.iter().any(|diagnostic| diagnostic.severity == Severity::Error) {
        let file = display_name(&args.file);
        eprint!("{}", diagnostics::render(&found, Some(&source), &file, lang));
        eprintln!("{file}: {}", diagnostics::summary(&found, lang));
        return Ok(Outcome::CompileErrors);
    }
    let document = serde_json::to_value(&compilation.document).map_err(|error| CliError::new(error.to_string()))?;
    let graph = build(&document);
    match args.format {
        GraphFormat::Text => print!("{}", text(&graph, lang)),
        GraphFormat::Json => print_json(&graph),
        GraphFormat::Mermaid => print!("{}", mermaid(&graph)),
        GraphFormat::Dot => print!("{}", dot(&graph)),
    }
    Ok(Outcome::Success)
}

pub fn build(document: &Value) -> Graph {
    let pages: &[Value] = document["pages"].as_array().map(Vec::as_slice).unwrap_or_default();
    let ids: Vec<String> = pages.iter().map(|page| page["id"].as_str().unwrap_or_default().to_owned()).collect();
    let hidden = |index: usize| pages[index]["hidden"].as_bool().unwrap_or(false);
    let following = |index: usize| -> Option<usize> {
        if let Some(target) = pages[index]["next"].as_str()
            && let Some(position) = ids.iter().position(|id| id == target)
        {
            return Some(position);
        }
        (index + 1..pages.len()).find(|&candidate| !hidden(candidate))
    };

    let mut edges = BTreeSet::new();
    let mut returns = vec![false; pages.len()];
    for (index, page) in pages.iter().enumerate() {
        let locked = page["locked"].as_bool().unwrap_or(false);
        if !hidden(index)
            && !locked
            && let Some(target) = following(index)
        {
            let kind = if page["next"].is_string() { "next" } else { "flow" };
            edges.insert(Edge { from: ids[index].clone(), to: ids[target].clone(), kind, conditional: false });
        }
        let mut found = Vec::new();
        walk(page, None, false, &mut found);
        for (reference, kind, conditional) in found {
            let target = match reference {
                Reference::Page(id) => ids.iter().position(|candidate| *candidate == id),
                Reference::Next => following(index),
                Reference::Back => {
                    returns[index] = true;
                    None
                }
            };
            if let Some(target) = target {
                edges.insert(Edge { from: ids[index].clone(), to: ids[target].clone(), kind, conditional });
            }
        }
    }
    let position = |id: &str| ids.iter().position(|candidate| candidate == id);
    let mut two_way = vec![false; pages.len()];
    let mut restarts = vec![false; pages.len()];
    for edge in &edges {
        let Some(index) = position(&edge.from) else { continue };
        two_way[index] |= edges.iter().any(|other| other.from == edge.to && other.to == edge.from);
        restarts[index] |= position(&edge.to) == Some(0);
    }
    let mut edges: Vec<Edge> = edges.into_iter().collect();
    edges.sort_by_key(|edge| (position(&edge.from), position(&edge.to), edge.kind, edge.conditional));

    let mut reachable = vec![false; pages.len()];
    let mut queue = VecDeque::from([0]);
    while let Some(index) = queue.pop_front() {
        if index >= pages.len() || reachable[index] {
            continue;
        }
        reachable[index] = true;
        for edge in edges.iter().filter(|edge| edge.from == ids[index]) {
            if let Some(target) = ids.iter().position(|id| *id == edge.to) {
                queue.push_back(target);
            }
        }
    }

    let nodes = pages
        .iter()
        .enumerate()
        .map(|(index, page)| Node {
            id: ids[index].clone(),
            title: page["title"].as_str().unwrap_or_default().to_owned(),
            line: page["line"].as_u64().unwrap_or_default(),
            hidden: hidden(index),
            reachable: reachable[index],
            ending: index > 0
                && !returns[index]
                && (restarts[index] || !two_way[index])
                && edges.iter().all(|edge| edge.from != ids[index] || position(&edge.to).is_some_and(|target| target < index)),
        })
        .collect();
    let anchors = document["anchors"]
        .as_object()
        .map(|map| {
            map.iter()
                .filter_map(|(id, index)| Some(Anchor { id: id.clone(), page: ids.get(usize::try_from(index.as_u64()?).ok()?)?.clone() }))
                .collect()
        })
        .unwrap_or_default();
    Graph { pages: nodes, edges, anchors }
}

enum Reference {
    Page(String),
    Next,
    Back,
}

fn walk(value: &Value, kind: Option<&'static str>, conditional: bool, found: &mut Vec<(Reference, &'static str, bool)>) {
    match value {
        Value::Array(items) => items.iter().for_each(|item| walk(item, kind, conditional, found)),
        Value::Object(map) => {
            let conditional = conditional || map.get("when").is_some_and(|when| !when.is_null());
            if let Some(kind) = kind {
                match map.get("type").and_then(Value::as_str) {
                    Some("go") => {
                        if let Some(target) = map.get("target").and_then(Value::as_str) {
                            found.push((Reference::Page(target.to_owned()), kind, conditional));
                        }
                        return;
                    }
                    Some("next") => {
                        found.push((Reference::Next, kind, conditional));
                        return;
                    }
                    Some("back" | "prev") => {
                        found.push((Reference::Back, kind, conditional));
                        return;
                    }
                    _ => {}
                }
            }
            if map.get("type").and_then(Value::as_str) == Some("link")
                && let Some(target) = map.get("href").and_then(Value::as_str).and_then(|href| href.strip_prefix('#'))
            {
                found.push((Reference::Page(target.to_owned()), "link", conditional));
            }
            for (name, child) in map {
                let child_kind = match name.as_str() {
                    "click" => Some("click"),
                    "onEnter" => Some("enter"),
                    "onTimeout" => Some("timeout"),
                    "onSubmit" => Some("submit"),
                    "when" | "variants" => continue,
                    _ => kind,
                };
                walk(child, child_kind, conditional, found);
            }
        }
        _ => {}
    }
}

fn kind_label(kind: &str, lang: Lang) -> &'static str {
    match (kind, lang) {
        ("flow", Lang::Pt) => "seta",
        ("flow", Lang::En) => "arrow",
        ("next", _) => "next=",
        ("click", Lang::Pt) => "clique",
        ("click", Lang::En) => "click",
        ("enter", _) => "on-enter",
        ("timeout", _) => "on-timeout",
        ("submit", _) => "on-submit",
        _ => "link",
    }
}

fn text(graph: &Graph, lang: Lang) -> String {
    let pt = lang == Lang::Pt;
    let mut output = String::new();
    for node in &graph.pages {
        let mut tags = Vec::new();
        if node.hidden {
            tags.push(if pt { "oculto" } else { "hidden" });
        }
        if node.ending && node.reachable {
            tags.push(if pt { "final" } else { "ending" });
        }
        if !node.reachable {
            tags.push(if pt { "inalcançável" } else { "unreachable" });
        }
        let tags = if tags.is_empty() { String::new() } else { format!("  [{}]", tags.join(", ")) };
        output.push_str(&format!("#{} · {} ({} {}){tags}\n", node.id, node.title, if pt { "linha" } else { "line" }, node.line));
        for edge in graph.edges.iter().filter(|edge| edge.from == node.id) {
            let condition = if edge.conditional { if pt { ", condicional" } else { ", conditional" } } else { "" };
            output.push_str(&format!("  → #{} ({}{condition})\n", edge.to, kind_label(edge.kind, lang)));
        }
    }
    let list = |filter: &dyn Fn(&Node) -> bool| -> Vec<String> {
        graph.pages.iter().filter(|node| filter(node)).map(|node| format!("#{}", node.id)).collect()
    };
    let unreachable = list(&|node| !node.reachable);
    let endings = list(&|node| node.reachable && node.ending);
    output.push('\n');
    output.push_str(&format!(
        "{}: {} · {}: {} · {}: {}\n",
        if pt { "Páginas" } else { "Pages" },
        graph.pages.len(),
        if pt { "finais" } else { "endings" },
        if endings.is_empty() { "—".to_owned() } else { endings.join(" ") },
        if pt { "inalcançáveis" } else { "unreachable" },
        if unreachable.is_empty() { (if pt { "nenhum" } else { "none" }).to_owned() } else { unreachable.join(" ") },
    ));
    output
}

fn label(node: &Node) -> String {
    let title = if node.title.trim().is_empty() { String::new() } else { format!(" · {}", node.title) };
    format!("#{}{title}", node.id)
}

fn mermaid(graph: &Graph) -> String {
    let index = |id: &str| graph.pages.iter().position(|node| node.id == id).unwrap_or_default();
    let mut output = String::from("flowchart TD\n");
    for (position, node) in graph.pages.iter().enumerate() {
        let text = label(node).replace('#', "#35;").replace('"', "#quot;");
        let shape = if node.ending { format!("s{position}([\"{text}\"])") } else { format!("s{position}[\"{text}\"]") };
        output.push_str(&format!("  {shape}\n"));
    }
    for edge in &graph.edges {
        let arrow = if edge.kind == "flow" || edge.kind == "next" { "-.->" } else { "-->" };
        let text = if edge.conditional { format!("{} ?", edge.kind) } else { edge.kind.to_owned() };
        output.push_str(&format!("  s{} {arrow}|{text}| s{}\n", index(&edge.from), index(&edge.to)));
    }
    let unreachable: Vec<String> = graph.pages.iter().enumerate().filter(|(_, node)| !node.reachable).map(|(position, _)| format!("s{position}")).collect();
    let hidden: Vec<String> = graph.pages.iter().enumerate().filter(|(_, node)| node.hidden && node.reachable).map(|(position, _)| format!("s{position}")).collect();
    output.push_str("  classDef hidden stroke-dasharray: 4 3\n  classDef unreachable fill:#fde2e1,stroke:#d93025,color:#7a1c14\n");
    if !hidden.is_empty() {
        output.push_str(&format!("  class {} hidden\n", hidden.join(",")));
    }
    if !unreachable.is_empty() {
        output.push_str(&format!("  class {} unreachable\n", unreachable.join(",")));
    }
    output
}

fn dot(graph: &Graph) -> String {
    let quote = |text: &str| format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""));
    let mut output = String::from("digraph ribalta {\n  rankdir=TB;\n  node [shape=box, style=rounded];\n");
    for node in &graph.pages {
        let mut attributes = vec![format!("label={}", quote(&label(node)))];
        if node.ending {
            attributes.push("peripheries=2".to_owned());
        }
        if node.hidden {
            attributes.push("style=\"rounded,dashed\"".to_owned());
        }
        if !node.reachable {
            attributes.push("color=red, fontcolor=red".to_owned());
        }
        output.push_str(&format!("  {} [{}];\n", quote(&node.id), attributes.join(", ")));
    }
    for edge in &graph.edges {
        let mut attributes = vec![format!("label={}", quote(edge.kind))];
        if edge.kind == "flow" || edge.kind == "next" {
            attributes.push("style=dotted".to_owned());
        }
        if edge.conditional {
            attributes.push("style=dashed".to_owned());
        }
        output.push_str(&format!("  {} -> {} [{}];\n", quote(&edge.from), quote(&edge.to), attributes.join(", ")));
    }
    output.push_str("}\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(source: &str) -> Graph {
        let compilation = ribalta::compile_localized("", source, ribalta::Locale::from_code("pt"));
        assert!(compilation.diagnostics.iter().all(|diagnostic| diagnostic.severity != ribalta::Severity::Error), "{:?}", compilation.diagnostics);
        build(&serde_json::to_value(&compilation.document).unwrap())
    }

    fn edges(graph: &Graph) -> Vec<(String, String, &'static str, bool)> {
        graph.edges.iter().map(|edge| (edge.from.clone(), edge.to.clone(), edge.kind, edge.conditional)).collect()
    }

    #[test]
    fn follows_arrows_choices_links_and_conditions() {
        let source = "@state key = false\n\n--- {#start}\n# Início\n[Abrir](#door)\n[Pegar]{click=\"$key = true; go #hall\"}\n\n--- {#hall nav=off}\n# Corredor\n::: if $key\n[Porta]{click=\"go #end\"}\n:::\n\n--- {#door hidden}\n# Porta\n[Voltar]{click=\"go back\"}\n\n--- {#end hidden}\n# Fim\n\n--- {#lost hidden}\n# Perdido\n[Ir]{click=\"go #end\"}\n";
        let result = graph(source);
        assert_eq!(
            edges(&result),
            vec![
                ("start".to_owned(), "hall".to_owned(), "click", false),
                ("start".to_owned(), "hall".to_owned(), "flow", false),
                ("start".to_owned(), "door".to_owned(), "link", false),
                ("hall".to_owned(), "end".to_owned(), "click", true),
                ("lost".to_owned(), "end".to_owned(), "click", false),
            ]
        );
        let node = |id: &str| result.pages.iter().find(|node| node.id == id).unwrap();
        assert!(node("end").ending && node("end").reachable);
        assert!(!node("door").ending);
        assert!(!node("lost").reachable);
        assert!(!node("hall").ending);
    }

    #[test]
    fn treats_pages_that_only_lead_back_as_endings() {
        let result = graph("--- {#cover}\n# Capa\n[Ir]{click=\"go #hub\"}\n\n--- {#hub hidden}\n# Hub\n[A]{click=\"go #win\"}\n\n--- {#win hidden}\n# Fim\n[Outra rota]{click=\"go #hub\"}\n[Recomeçar]{click=\"go #cover\"}\n");
        let node = |id: &str| result.pages.iter().find(|node| node.id == id).unwrap();
        assert!(node("win").ending);
        assert!(!node("hub").ending);
    }

    #[test]
    fn rooms_that_lead_back_to_the_hub_are_not_endings() {
        let result = graph("--- {#cover}\n# Capa\n[Ir]{click=\"go #hub\"}\n\n--- {#hub hidden}\n# Ponte\n[Lab]{click=\"go #lab\"}\n[Sair]{click=\"go #out\"}\n\n--- {#lab hidden}\n# Lab\n[Voltar]{click=\"go #hub\"}\n\n--- {#out hidden}\n# Fim\n[De novo]{click=\"go #cover\"}\n");
        let node = |id: &str| result.pages.iter().find(|node| node.id == id).unwrap();
        assert!(!node("lab").ending);
        assert!(node("out").ending);
    }

    #[test]
    fn treats_play_again_as_an_ending() {
        let result = graph("--- {#cover}\n# Capa\n[Jogar]{click=\"go #win\" key=enter}\n\n--- {#win hidden}\n# Venceu\n[Jogar de novo]{click=\"go #cover\"}\n");
        assert_eq!(edges(&result), vec![("cover".to_owned(), "win".to_owned(), "click", false), ("win".to_owned(), "cover".to_owned(), "click", false)]);
        assert!(result.pages[1].ending && !result.pages[0].ending);
    }

    #[test]
    fn uses_next_attribute_and_timeouts() {
        let result = graph("--- {#a next=c}\n# A\n\n--- {#b}\n# B\n\n--- {#c timeout=5 on-timeout=\"go #b\"}\n# C\n");
        assert!(edges(&result).contains(&("a".to_owned(), "c".to_owned(), "next", false)));
        assert!(edges(&result).contains(&("c".to_owned(), "b".to_owned(), "timeout", false)));
        assert!(result.pages.iter().all(|node| node.reachable));
        assert!(mermaid(&result).contains("s0 -.->|next| s2"));
        assert!(dot(&result).contains("\"c\" -> \"b\" [label=\"timeout\"]"));
    }
}
