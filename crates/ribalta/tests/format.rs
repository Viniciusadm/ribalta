use std::path::{Path, PathBuf};

use ribalta::{format_source, same_result};
use ribalta_testkit::corpus::{self, Sample};

fn language_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn corpus() -> Vec<Sample> {
    let root = language_root();
    corpus::samples(&[root.join("android/app/src/main/assets")], &[root.join("docs")])
}

#[test]
fn corpus_formats_without_changing_the_result() {
    corpus::assert_formatting_keeps_results(&corpus(), 300, format_source, same_result);
}

#[test]
fn indented_corpus_returns_to_the_flat_style() {
    corpus::assert_indentation_is_normalized(&corpus(), 300, format_source, same_result);
}

#[test]
fn normalizes_structure_and_whitespace() {
    let source = "\n\n@title  Demo   \r\n\r\n\r\n# Um   \r\n-----{.dark  align=center}\n  :::columns {gap=lg}\n    Esquerda  \n\n\n    Direita\n  :::\n    ::icon star\n";
    assert_eq!(
        format_source(source),
        "@title  Demo\n\n# Um\n--- {.dark  align=center}\n::: columns {gap=lg}\nEsquerda\n\nDireita\n:::\n:: icon star\n"
    );
}

#[test]
fn keeps_relative_indentation_code_and_notes() {
    let source = "    # Lista\n    - um\n      - dois\n        continua\n    - três\n\n```js\n  if (x) {   \n\n\n    y();\n  }\n```\n\n::: notes\n   primeira\n\n\n   segunda\n:::\n";
    let formatted = format_source(source);
    assert_eq!(
        formatted,
        "# Lista\n- um\n  - dois\n    continua\n- três\n\n```js\n  if (x) {   \n\n\n    y();\n  }\n```\n\n::: notes\nprimeira\n\n\nsegunda\n:::\n"
    );
    assert!(same_result("", source, &formatted));
}

#[test]
fn detects_changed_results() {
    assert!(!same_result("", "- a\n\n- b\n", "- a\n- b\n"));
    assert!(same_result("", "# A\n\n\n# B\n", "# A\n# B\n"));
}

#[test]
fn empty_and_blank_sources_format_to_nothing() {
    assert_eq!(format_source(""), "");
    assert_eq!(format_source("\n  \n\t\n"), "");
}
