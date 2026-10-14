use ribalta::library::{self, DefKind, DefRef, Definition, ItemKind, Resolution};
use ribalta::{Locale, Severity};

fn find<'a>(definitions: &'a [Definition], kind: DefKind, name: &str) -> &'a Definition {
    definitions.iter().find(|definition| definition.kind == kind && definition.name == name).unwrap_or_else(|| panic!("missing {kind:?} {name}"))
}

fn deps(definition: &Definition) -> Vec<(DefKind, &str)> {
    definition.deps.iter().map(|dep| (dep.kind, dep.name.as_str())).collect()
}

fn key(kind: DefKind, name: &str) -> DefRef {
    DefRef { kind, name: name.to_string() }
}

fn errors(source: &str) -> Vec<ribalta::Diagnostic> {
    ribalta::check_prelude(source).into_iter().filter(|diagnostic| diagnostic.severity == Severity::Error).collect()
}

fn codes(diagnostics: &[ribalta::Diagnostic]) -> Vec<&'static str> {
    diagnostics.iter().filter(|diagnostic| diagnostic.severity == Severity::Error).map(|diagnostic| diagnostic.code).collect()
}

const THEME: &str = "@theme ink
@color brand #ff6b4a
@color brand-soft brand/40
@color deep #102030
@font title \"Bebas Neue\" 400
@style loud {color=brand font=title}
@style framed {.loud border=\"3 deep\" bg=brand..deep}
// A card with a heading
@component card {title tone=brand}
::: box {.framed}
## $title {color=$tone}
[destaque]{.loud} e [cor]{.brand-soft}
$content
:::
@end
@component pair {left}
::: row
:: card {title=$left}
:: badge
:::
@end
@component badge
::: box {bg=deep}
$label
:::
@end
@footer
[rodapé]{.loud}
@end
";

#[test]
fn definitions_report_lines_params_and_slots() {
    let definitions = library::definitions(THEME);
    let card = find(&definitions, DefKind::Component, "card");
    assert_eq!((card.start_line, card.end_line), (9, 15));
    assert_eq!(card.params.iter().map(|param| (param.name.as_str(), param.default.as_deref())).collect::<Vec<_>>(), [("title", None), ("tone", Some("brand"))]);
    assert!(card.uses_content);
    assert!(card.slots.is_empty());
    let badge = find(&definitions, DefKind::Component, "badge");
    assert_eq!(badge.slots, ["label"]);
    assert!(!badge.uses_content);
    let setting = find(&definitions, DefKind::Setting, "theme");
    assert_eq!((setting.start_line, setting.end_line), (1, 1));
    let footer = find(&definitions, DefKind::Footer, "footer");
    assert_eq!((footer.start_line, footer.end_line), (27, 29));
}

#[test]
fn dependencies_cover_classes_values_fonts_components_and_inline_spans() {
    let definitions = library::definitions(THEME);
    assert_eq!(deps(find(&definitions, DefKind::Color, "brand-soft")), [(DefKind::Color, "brand")]);
    assert_eq!(deps(find(&definitions, DefKind::Style, "loud")), [(DefKind::Color, "brand"), (DefKind::Font, "title")]);
    assert_eq!(deps(find(&definitions, DefKind::Style, "framed")), [(DefKind::Style, "loud"), (DefKind::Color, "deep"), (DefKind::Color, "brand")]);
    let card = deps(find(&definitions, DefKind::Component, "card"));
    for expected in [(DefKind::Style, "framed"), (DefKind::Style, "loud"), (DefKind::Color, "brand-soft"), (DefKind::Color, "brand")] {
        assert!(card.contains(&expected), "{card:?} lacks {expected:?}");
    }
    assert_eq!(deps(find(&definitions, DefKind::Component, "pair")), [(DefKind::Component, "card"), (DefKind::Component, "badge")]);
    assert_eq!(deps(find(&definitions, DefKind::Component, "badge")), [(DefKind::Color, "deep")]);
    assert_eq!(deps(find(&definitions, DefKind::Footer, "footer")), [(DefKind::Style, "loud")]);
}

#[test]
fn builtins_and_params_are_never_dependencies() {
    let source = "@color brand coral/50
@style plain {.muted .card color=accent bg=surface..paper font=display border=\"2 line\"}
@component chip {tone}
::: box {bg=$tone .center}
[x]{.accent color=$tone}
:::
@end
";
    let definitions = library::definitions(source);
    assert!(find(&definitions, DefKind::Color, "brand").deps.is_empty());
    assert!(find(&definitions, DefKind::Style, "plain").deps.is_empty());
    assert!(find(&definitions, DefKind::Component, "chip").deps.is_empty());
}

#[test]
fn class_resolves_to_style_before_color() {
    let source = "@color tone #112233\n@style tone {weight=bold}\n@style uses {.tone}";
    let definitions = library::definitions(source);
    assert_eq!(deps(find(&definitions, DefKind::Style, "uses")), [(DefKind::Style, "tone")]);
    let only_color = library::definitions("@color tone #112233\n@style uses {.tone}");
    assert_eq!(deps(find(&only_color, DefKind::Style, "uses")), [(DefKind::Color, "tone")]);
}

#[test]
fn extract_takes_the_closure_in_dependency_order() {
    let extraction = library::extract(THEME, &[key(DefKind::Component, "pair")]);
    assert!(codes(&extraction.diagnostics).is_empty(), "{:#?}\n{}", extraction.diagnostics, extraction.source);
    let included: Vec<(DefKind, &str)> = extraction.included.iter().map(|item| (item.kind, item.name.as_str())).collect();
    assert_eq!(
        included,
        [
            (DefKind::Color, "brand"),
            (DefKind::Color, "brand-soft"),
            (DefKind::Color, "deep"),
            (DefKind::Font, "title"),
            (DefKind::Style, "loud"),
            (DefKind::Style, "framed"),
            (DefKind::Component, "card"),
            (DefKind::Component, "pair"),
            (DefKind::Component, "badge"),
        ]
    );
    assert!(extraction.source.contains("// A card with a heading\n@component card"));
    assert!(!extraction.source.contains("@theme"));
    assert!(!extraction.source.contains("@footer"));
    let compiled = ribalta::compile_with_prelude(&extraction.source, ":: pair {left=Oi}");
    assert!(compiled.diagnostics.is_empty(), "{:#?}", compiled.diagnostics);
}

#[test]
fn extract_orders_colors_and_styles_topologically() {
    let source = "@style b {.a weight=bold}\n@style a {color=late}\n@color late early/50\n@color early #123456";
    assert!(!errors(source).is_empty());
    let extraction = library::extract(source, &[key(DefKind::Style, "b")]);
    assert_eq!(extraction.source, "@color early #123456\n@color late early/50\n\n@style a {color=late}\n@style b {.a weight=bold}\n");
    assert!(extraction.diagnostics.is_empty(), "{:#?}", extraction.diagnostics);
}

#[test]
fn extract_reports_missing_roots() {
    let extraction = library::extract(THEME, &[key(DefKind::Style, "nope")]);
    assert_eq!(codes(&extraction.diagnostics), ["R532"]);
    assert!(extraction.source.is_empty());
}

#[test]
fn conflicts_flag_identical_and_different_definitions() {
    let found = library::conflicts("@color brand #ff0000\n@style big {size=xl}", "@color brand #ff0000\n@style big {size=2xl}\n@color other #000");
    let summary: Vec<(DefKind, &str, bool)> = found.iter().map(|conflict| (conflict.kind, conflict.name.as_str(), conflict.identical)).collect();
    assert_eq!(summary, [(DefKind::Color, "brand", true), (DefKind::Style, "big", false)]);
}

#[test]
fn merge_appends_new_definitions_with_label_and_skips_identical() {
    let theme = "@theme paper\n@color brand #ff0000\n";
    let snippet = "@color brand #ff0000\n@color glow brand/30\n@style hot {color=glow}";
    let merged = library::merge(theme, snippet, &[], Some("Kit quente")).unwrap();
    assert_eq!(merged.source, "@theme paper\n@color brand #ff0000\n\n// Biblioteca: Kit quente\n@color glow brand/30\n\n@style hot {color=glow}\n");
    assert_eq!(merged.skipped, [key(DefKind::Color, "brand")]);
    assert!(merged.renamed.is_empty());
    assert!(errors(&merged.source).is_empty());

    let unchanged = library::merge(theme, "@color brand #ff0000", &[], None).unwrap();
    assert_eq!(unchanged.source, theme);
}

#[test]
fn merge_requires_resolutions_for_different_definitions() {
    let conflicts = library::merge("@color brand #ff0000", "@color brand #00ff00\n@style x {size=xl}\n@style y {size=xl}", &[], None).unwrap_err();
    assert_eq!(conflicts.len(), 1);
    assert_eq!((conflicts[0].kind, conflicts[0].name.as_str(), conflicts[0].identical), (DefKind::Color, "brand", false));
}

#[test]
fn merge_replace_swaps_the_theme_block_in_place() {
    let theme = "@color brand #ff0000\n@color shade brand/50\n@style big {size=xl color=shade}\n";
    let snippet = "@color base #0000ff\n// novo azul\n@color brand base/80\n@style other {color=brand}";
    let resolutions = [Resolution::Replace { kind: DefKind::Color, name: "brand".into() }];
    let merged = library::merge(theme, snippet, &resolutions, None).unwrap();
    assert_eq!(merged.source, "@color base #0000ff\n// novo azul\n@color brand base/80\n@color shade brand/50\n@style big {size=xl color=shade}\n\n@style other {color=brand}\n");
    assert!(errors(&merged.source).is_empty(), "{:#?}", errors(&merged.source));
}

#[test]
fn merge_rename_rewrites_definitions_and_references() {
    let theme = "@color brand #ff0000\n@style loud {weight=bold}\n@component card {title}\n# $title\n@end\n";
    let snippet = "@color brand #00ff00
@color brand-soft brand/40
@style loud {color=brand border=\"2 brand\" bg=brand..brand-soft}
@style quiet {.loud .brand}
@component card {title tone=brand}
::: box {.loud color=$tone}
## $title
[x]{.loud} e [y]{.brand}
:::
@end
@component wrapper
::: card {title=Oi tone=brand}
:::
@end
";
    let resolutions = [
        Resolution::Rename { kind: DefKind::Color, name: "brand".into(), to: "verde".into() },
        Resolution::Rename { kind: DefKind::Style, name: "loud".into(), to: "louder".into() },
        Resolution::Rename { kind: DefKind::Component, name: "card".into(), to: "green-card".into() },
    ];
    let merged = library::merge(theme, snippet, &resolutions, Some("Verde")).unwrap();
    let source = &merged.source;
    assert!(source.contains("@color verde #00ff00"), "{source}");
    assert!(source.contains("@color brand-soft verde/40"), "{source}");
    assert!(source.contains("@style louder {color=verde border=\"2 verde\" bg=verde..brand-soft}"), "{source}");
    assert!(source.contains("@style quiet {.louder .verde}"), "{source}");
    assert!(source.contains("@component green-card {title tone=verde}"), "{source}");
    assert!(source.contains("::: box {.louder color=$tone}"), "{source}");
    assert!(source.contains("[x]{.louder} e [y]{.verde}"), "{source}");
    assert!(source.contains("::: green-card {title=Oi tone=verde}"), "{source}");
    assert!(source.starts_with("@color brand #ff0000\n@style loud {weight=bold}\n"), "{source}");
    let renamed: Vec<(DefKind, &str, &str)> = merged.renamed.iter().map(|item| (item.kind, item.from.as_str(), item.to.as_str())).collect();
    assert_eq!(renamed, [(DefKind::Color, "brand", "verde"), (DefKind::Style, "loud", "louder"), (DefKind::Component, "card", "green-card")]);
    assert!(ribalta::check_prelude(source).is_empty(), "{:#?}", ribalta::check_prelude(source));
    let compiled = ribalta::compile_with_prelude(source, ":: wrapper\n\n:: card {title=X}\n\n:: green-card {title=Y}");
    assert!(compiled.diagnostics.is_empty(), "{:#?}", compiled.diagnostics);
}

#[test]
fn merge_rename_rejects_taken_or_invalid_names() {
    let theme = "@color brand #ff0000\n@color green #00ff00";
    let snippet = "@color brand #0000ff";
    for to in ["green", "Bad Name", "accent"] {
        let resolutions = [Resolution::Rename { kind: DefKind::Color, name: "brand".into(), to: to.into() }];
        assert!(library::merge(theme, snippet, &resolutions, None).is_err(), "{to}");
    }
}

#[test]
fn merge_keep_drops_the_snippet_definition() {
    let theme = "@color brand #ff0000\n";
    let snippet = "@color brand #00ff00\n@style hot {color=brand}";
    let resolutions = [Resolution::Keep { kind: DefKind::Color, name: "brand".into() }];
    let merged = library::merge(theme, snippet, &resolutions, None).unwrap();
    assert_eq!(merged.source, "@color brand #ff0000\n\n@style hot {color=brand}\n");
    assert_eq!(merged.skipped, [key(DefKind::Color, "brand")]);
    assert!(errors(&merged.source).is_empty());
}

#[test]
fn merge_components_never_duplicate() {
    let theme = "@component card {title}\n# $title\n@end\n";
    let snippet = "@component card {title}\n## $title\n@end\n";
    assert!(library::merge(theme, snippet, &[], None).is_err());
    let replaced = library::merge(theme, snippet, &[Resolution::Replace { kind: DefKind::Component, name: "card".into() }], None).unwrap();
    assert_eq!(replaced.source, "@component card {title}\n## $title\n@end\n");
    let compiled = ribalta::compile_with_prelude(&replaced.source, ":: card {title=Oi}");
    assert!(compiled.diagnostics.is_empty(), "{:#?}", compiled.diagnostics);
}

#[test]
fn validate_item_checks_allowed_definitions_per_kind() {
    let palette = library::validate_item(ItemKind::Palette, "@color a #fff\n@font f \"Inter\"\n@style s {color=a}", None, Locale::Pt);
    assert_eq!(codes(&palette), ["R530"]);
    assert_eq!(palette[0].line, 3);
    assert!(library::validate_item(ItemKind::Palette, "@color a #fff\n@font f \"Inter\"", None, Locale::Pt).is_empty());

    let theme_with_pages = library::validate_item(ItemKind::Theme, "@color a #fff\n\n# Page", None, Locale::Pt);
    assert_eq!(codes(&theme_with_pages), ["R519"]);
    assert!(library::validate_item(ItemKind::Theme, THEME, None, Locale::Pt).is_empty());

    let style_with_component = library::validate_item(ItemKind::Style, "@style s {weight=bold}\n@component c\n# x\n@end", Some("s"), Locale::En);
    assert_eq!(codes(&style_with_component), ["R530"]);
    assert!(style_with_component[0].message.contains("cannot be used in a style"));
    assert!(library::validate_item(ItemKind::Style, "@color a #fff\n@style s {color=a}", Some("s"), Locale::Pt).is_empty());
}

#[test]
fn validate_item_requires_an_existing_entry() {
    assert_eq!(codes(&library::validate_item(ItemKind::Component, "@component c\n# x\n@end", None, Locale::Pt)), ["R531"]);
    assert_eq!(codes(&library::validate_item(ItemKind::Component, "@component c\n# x\n@end", Some("d"), Locale::Pt)), ["R532"]);
    assert_eq!(codes(&library::validate_item(ItemKind::Style, "@color s #fff", Some("s"), Locale::Pt)), ["R532"]);
    assert!(library::validate_item(ItemKind::Component, THEME.split("@footer").next().unwrap().replace("@theme ink\n", "").as_str(), Some("pair"), Locale::Pt).is_empty());
}

#[test]
fn validate_item_compiles_a_sample_usage_of_the_component() {
    let broken = "@component c {title}\n::: box {colr=1}\n# $title $missing\n:::\n@end";
    let found = library::validate_item(ItemKind::Component, broken, Some("c"), Locale::Pt);
    let found_codes = codes(&found);
    assert!(found_codes.contains(&"R209") && found_codes.contains(&"R410"), "{found:#?}");
    assert!(found.iter().all(|diagnostic| diagnostic.line == 1));

    let typed = "@component chip {tone size label}\n::: box {bg=$tone size=$size .$label}\n$content\n:::\n@end";
    assert!(library::validate_item(ItemKind::Component, typed, Some("chip"), Locale::Pt).is_empty(), "{:#?}", library::validate_item(ItemKind::Component, typed, Some("chip"), Locale::Pt));
}

#[test]
fn json_wrappers_follow_the_contract() {
    let definitions: serde_json::Value = serde_json::from_str(&ribalta::definitions_to_json("@component c {t=1}\n$content\n@end")).unwrap();
    assert_eq!(definitions[0]["kind"], "component");
    assert_eq!(definitions[0]["startLine"], 1);
    assert_eq!(definitions[0]["usesContent"], true);
    assert_eq!(definitions[0]["params"][0]["default"], "1");

    let extraction: serde_json::Value = serde_json::from_str(&ribalta::extract_to_json("@color a #fff\n@style s {color=a}", r#"[{"kind":"style","name":"s"}]"#)).unwrap();
    assert_eq!(extraction["included"], serde_json::json!([{"kind": "color", "name": "a"}, {"kind": "style", "name": "s"}]));

    let failed: serde_json::Value = serde_json::from_str(&ribalta::merge_to_json("@color a #fff", "@color a #000", "[]", "")).unwrap();
    assert_eq!(failed, serde_json::json!({"ok": false, "conflicts": [{"kind": "color", "name": "a", "identical": false}]}));
    let merged: serde_json::Value = serde_json::from_str(&ribalta::merge_to_json("@color a #fff", "@color a #000", r#"[{"action":"rename","kind":"color","name":"a","to":"b"}]"#, "Kit")).unwrap();
    assert_eq!(merged["ok"], true);
    assert_eq!(merged["source"], "@color a #fff\n\n// Biblioteca: Kit\n@color b #000\n");
    assert_eq!(merged["renamed"], serde_json::json!([{"kind": "color", "from": "a", "to": "b"}]));

    let unknown: serde_json::Value = serde_json::from_str(&ribalta::validate_item_to_json("widget", "", "", Locale::En)).unwrap();
    assert_eq!(unknown[0]["code"], "R533");
    let palette: serde_json::Value = serde_json::from_str(&ribalta::validate_item_to_json("palette", "@style s {size=xl}", "", Locale::Pt)).unwrap();
    assert_eq!(palette[0]["code"], "R530");
}

#[test]
fn overlay_is_a_definition_with_its_dependencies() {
    let source = "@style stamp {color=coral weight=bold}\n@overlay\n[Rascunho]{.stamp}\n@end\n";
    let definitions = library::definitions(source);
    let overlay = find(&definitions, DefKind::Overlay, "overlay");
    assert_eq!((overlay.start_line, overlay.end_line), (2, 4));
    assert_eq!(deps(overlay), [(DefKind::Style, "stamp")]);
}
