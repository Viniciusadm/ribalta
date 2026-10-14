use ribalta::highlight::highlight;
use ribalta::ir::TokenKind::{self, *};

fn kinds(code: &str, lang: &str) -> Vec<Vec<(TokenKind, std::string::String)>> {
    highlight(code, Some(lang)).into_iter().map(|line| line.into_iter().filter(|token| !token.text.trim().is_empty()).map(|token| (token.kind, token.text)).collect()).collect()
}

fn find(code: &str, lang: &str, text: &str) -> TokenKind {
    kinds(code, lang).into_iter().flatten().find(|(_, token)| token.trim() == text).unwrap_or_else(|| panic!("no token {text}")).0
}

#[test]
fn javascript_tokens_get_their_kinds() {
    let code = "// soma\nconst total = add(1, 2.5e-3) + `x ${y}`;\nclass Deck extends Base {}";
    assert_eq!(find(code, "js", "// soma"), Comment);
    assert_eq!(find(code, "js", "const"), Keyword);
    assert_eq!(find(code, "js", "add"), Function);
    assert_eq!(find(code, "js", "2.5e-3"), Number);
    assert_eq!(find(code, "js", "`x ${y}`"), String);
    assert_eq!(find(code, "js", "Deck"), Type);
    assert_eq!(find(code, "js", "total"), Plain);
    assert_eq!(find(code, "js", "+"), Operator);
}

#[test]
fn lines_are_split_and_text_is_preserved() {
    let code = "/* um\ndois */\nlet a = 1;\n\n";
    let lines = highlight(code, Some("ts"));
    assert_eq!(lines.len(), 5);
    let joined = lines.iter().map(|line| line.iter().map(|token| token.text.as_str()).collect::<std::string::String>()).collect::<Vec<_>>().join("\n");
    assert_eq!(joined, code);
    assert_eq!(lines[0][0].kind, Comment);
    assert_eq!(lines[1][0].kind, Comment);
}

#[test]
fn languages_with_their_own_rules() {
    assert_eq!(find("def f(x):\n    \"\"\"doc\n    more\"\"\"\n    return None", "python", "None"), Constant);
    assert_eq!(find("@app.get\ndef f(): pass", "python", "@app.get"), Function);
    assert_eq!(find("fn f<'a>(x: &'a str) -> char { 'x' }", "rust", "'a"), Type);
    assert_eq!(find("fn f<'a>(x: &'a str) -> char { 'x' }", "rust", "'x'"), String);
    assert_eq!(find("echo $HOME # casa", "bash", "$HOME"), Variable);
    assert_eq!(find("SELECT id FROM users", "sql", "FROM"), Keyword);
    assert_eq!(find("{\"name\": \"Folio\", \"ok\": true}", "json", "\"name\""), Property);
    assert_eq!(find("{\"name\": \"Folio\", \"ok\": true}", "json", "\"Folio\""), String);
    assert_eq!(find("title: Folio\nlist:\n  - one", "yaml", "title"), Property);
    assert_eq!(find("#include <stdio.h>\nint main() {}", "c", "#include"), Keyword);
}

#[test]
fn markup_and_css() {
    let html = "<!-- nota -->\n<a href=\"/x\" data-on>Oi &amp; tchau</a>";
    assert_eq!(find(html, "html", "<!-- nota -->"), Comment);
    assert_eq!(find(html, "html", "a"), Tag);
    assert_eq!(find(html, "html", "href"), Attribute);
    assert_eq!(find(html, "html", "\"/x\""), String);
    assert_eq!(find(html, "html", "&amp;"), Constant);
    let css = ".card:hover { color: #fff; margin: 0 2px; --gap: calc(1rem + 2px); }";
    assert_eq!(find(css, "css", ".card:hover"), Tag);
    assert_eq!(find(css, "css", "color"), Property);
    assert_eq!(find(css, "css", "#fff"), Number);
    assert_eq!(find(css, "css", "2px"), Number);
    assert_eq!(find(css, "css", "calc"), Function);
}

#[test]
fn unknown_languages_are_plain_text() {
    let lines = highlight("qualquer coisa\naqui", Some("folio"));
    assert_eq!(lines.len(), 2);
    assert!(lines.iter().flatten().all(|token| token.kind == Plain));
    assert_eq!(highlight("x", None)[0][0].kind, Plain);
}
