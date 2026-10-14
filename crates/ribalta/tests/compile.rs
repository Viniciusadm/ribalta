use ribalta::{Compilation, Severity, compile};
use serde_json::{Value, json};

fn output(source: &str) -> (Value, Compilation) {
    let compilation = compile(source);
    (serde_json::to_value(&compilation).unwrap(), compilation)
}

fn codes(compilation: &Compilation) -> Vec<&'static str> {
    compilation.diagnostics.iter().map(|d| d.code).collect()
}

fn clean(source: &str) -> Value {
    let (value, compilation) = output(source);
    assert!(compilation.diagnostics.is_empty(), "unexpected diagnostics: {:#?}", compilation.diagnostics);
    value
}

#[test]
fn splits_pages_on_separator_and_takes_titles_from_headings() {
    let value = clean("# Primeiro\n\nTexto\n---\n## Segundo\n---\nSem título");
    let pages = value["document"]["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0]["title"], "Primeiro");
    assert_eq!(pages[1]["title"], "Segundo");
    assert_eq!(pages[2]["title"], "Página 3");
    assert_eq!(pages[1]["line"], 4);
}

#[test]
fn preamble_sets_metadata_theme_and_custom_tokens() {
    let value = clean("@title Minha palestra\n@theme ink\n@color marca #ff6b4a\n@font titulo \"Bebas Neue\" 400\n\n# Oi {color=marca font=titulo}");
    let document = &value["document"];
    assert_eq!(document["meta"]["title"], "Minha palestra");
    assert_eq!(document["theme"]["name"], "ink");
    let heading = &document["pages"][0]["children"][0];
    assert_eq!(heading["style"]["color"], "#ff6b4a");
    assert_eq!(heading["style"]["font"], "Bebas Neue");
    assert!(document["theme"]["fontFamilies"].as_array().unwrap().contains(&json!({ "family": "Bebas Neue", "weights": [400] })));
}

#[test]
fn themes_describe_their_animations_as_data() {
    let motion = &clean("# Oi")["document"]["theme"]["motion"];
    assert_eq!(motion["enter"]["up"], json!({ "duration": 360, "easing": [0.25, 0.1, 0.25, 1.0], "from": { "opacity": 0.0, "dy": { "unit": "px", "value": 16.0 } } }));
    assert_eq!(motion["transitions"]["slide"]["from"]["dx"], json!({ "unit": "pct", "value": 6.0 }));
    assert!(motion["transitions"].get("none").is_none());
    assert_eq!(motion["reveal"]["duration"], 420);
}

#[test]
fn themes_describe_element_decorations_with_their_own_colors() {
    let elements = &clean("@color accent #123456\n\n# Oi")["document"]["theme"]["elements"];
    assert_eq!(elements["list"]["markerColor"], "#123456");
    assert_eq!(elements["quote"]["barColor"], "#123456");
    assert_eq!(elements["code"]["font"], "JetBrains Mono");
    assert_eq!(elements["shadows"]["md"], json!({ "x": 0.0, "y": 10.0, "blur": 30.0, "color": "#0f121e29" }));
    assert_eq!(elements["icon"], json!({ "size": 96.0, "stroke": 1.75 }));
    let layout = &clean("# Oi")["document"]["theme"]["layout"];
    assert_eq!(layout["gaps"], json!({ "stack": 32.0, "row": 16.0, "columns": 32.0, "grid": 32.0, "box": 20.0 }));
    assert_eq!(layout["band"], json!({ "gap": 16.0, "edgeScale": 0.4, "heightScale": 0.6 }));
}

#[test]
fn documents_carry_the_drawings_of_the_icons_they_use() {
    let value = clean("::: columns\n:: icon rocket\n- item\n:::\n---\n:: icon rocket\n:: icon check");
    let icons = value["document"]["icons"].as_object().unwrap();
    assert_eq!(icons.keys().collect::<Vec<_>>(), ["check", "rocket"]);
    assert!(icons["rocket"].as_array().unwrap().iter().all(|element| element["tag"].is_string() && element["attrs"].is_object()));
    assert!(clean("# Sem ícones")["document"].get("icons").is_none());
    assert!(ribalta::icons::names().count() > 200);
}

#[test]
fn newlines_inside_a_paragraph_are_line_breaks() {
    let value = clean("PRIMEIRA\nSEGUNDA");
    let content = &value["document"]["pages"][0]["children"][0]["content"];
    assert_eq!(content, &json!([{"type": "text", "value": "PRIMEIRA"}, {"type": "break"}, {"type": "text", "value": "SEGUNDA"}]));
}

#[test]
fn inline_markup_and_spans_with_attributes() {
    let value = clean("Mas ainda **não** estava [pronto]{.coral weight=bold} e *talvez* ~~não~~ `x`");
    let content = value["document"]["pages"][0]["children"][0]["content"].as_array().unwrap().clone();
    let types: Vec<&str> = content.iter().map(|node| node["type"].as_str().unwrap()).collect();
    assert_eq!(types, ["text", "strong", "text", "span", "text", "emphasis", "text", "strike", "text", "code"]);
    assert_eq!(content[3]["style"]["color"], "#f2603d");
    assert_eq!(content[3]["style"]["weight"], 700);
}

#[test]
fn trailing_attributes_need_a_space_and_escape_works() {
    let value = clean("# Título {size=5xl .accent}\n\nPreço \\{fixo}");
    let children = &value["document"]["pages"][0]["children"];
    assert_eq!(children[0]["style"]["size"], 150.0);
    assert_eq!(children[0]["content"][0]["value"], "Título");
    assert_eq!(children[1]["content"][0]["value"], "Preço {fixo}");
}

#[test]
fn columns_and_nested_containers() {
    let value = clean("::: columns {widths=\"2 1\" gap=xl}\n::: stack\n# Esquerda\nTexto\n:::\n![Foto](https://example.com/a.png){fit=cover radius=lg}\n:::");
    let layout = &value["document"]["pages"][0]["children"][0];
    assert_eq!(layout["type"], "layout");
    assert_eq!(layout["layout"], "columns");
    assert_eq!(layout["widths"], json!([2.0, 1.0]));
    assert_eq!(layout["style"]["gap"], 72.0);
    assert_eq!(layout["children"][0]["children"].as_array().unwrap().len(), 2);
    assert_eq!(layout["children"][1]["type"], "image");
    assert_eq!(layout["children"][1]["fit"], "cover");
}

#[test]
fn steps_reveal_children_and_list_items_in_order() {
    let value = clean("# Etapas\n::: steps\nPrimeiro ponto.\n\n- a\n- b\n:::\nFinal {step=5}");
    let page = &value["document"]["pages"][0];
    let steps = &page["children"][1]["children"];
    assert_eq!(steps[0]["step"], 1);
    assert_eq!(steps[1]["items"][0]["step"], 2);
    assert_eq!(steps[1]["items"][1]["step"], 3);
    assert_eq!(page["children"][2]["step"], 5);
    assert_eq!(page["steps"], 5);
}

#[test]
fn page_attributes_and_ids() {
    let value = clean("--- {#capa .dark align=center}\n# Oi");
    let page = &value["document"]["pages"][0];
    assert_eq!(page["id"], "capa");
    assert_eq!(page["style"]["align"], "center");
    assert_eq!(page["style"]["bg"], json!({ "kind": "solid", "color": "#151821" }));
}

#[test]
fn components_expand_with_parameters_and_content() {
    let source = "@component option {letter color=accent}\n::: row {gap=md}\n::: box {bg=$color}\n**$letter**\n:::\n$content\n:::\n@end\n\n::: option {letter=A color=blue}\nTentaria voltar\n:::";
    let value = clean(source);
    let row = &value["document"]["pages"][0]["children"][0];
    assert_eq!(row["layout"], "row");
    assert_eq!(row["children"][0]["style"]["bg"], json!({ "kind": "solid", "color": "#2f6fdb" }));
    assert_eq!(row["children"][0]["children"][0]["content"][0]["type"], "strong");
    assert_eq!(row["children"][0]["children"][0]["content"][0]["children"][0]["value"], "A");
    assert_eq!(row["children"][1]["content"][0]["value"], "Tentaria voltar");
}

#[test]
fn component_errors_point_to_the_invocation() {
    let source = "@component card {title}\n::: box\n# $title {colr=red}\n:::\n@end\n\n::: card {titel=x}\n:::";
    let (_, compilation) = output(source);
    let found = codes(&compilation);
    assert!(found.contains(&"R407"), "{found:?}");
    assert!(found.contains(&"R409"), "{found:?}");
    let unknown = compilation.diagnostics.iter().find(|d| d.code == "R209").unwrap();
    assert_eq!(unknown.line, 7);
    assert!(unknown.message.contains("no componente `card`"));
}

#[test]
fn recursive_components_are_rejected() {
    let (_, compilation) = output("@component loop\n::: loop\n:::\n@end\n\n::: loop\n:::");
    assert!(codes(&compilation).contains(&"R404"));
}

#[test]
fn unknown_attribute_suggests_closest_name() {
    let (_, compilation) = output("# Título {colr=coral}");
    let diagnostic = &compilation.diagnostics[0];
    assert_eq!(diagnostic.code, "R209");
    assert_eq!(diagnostic.line, 1);
    assert_eq!(diagnostic.column, 11);
    assert_eq!(diagnostic.end_column, 15);
    assert!(diagnostic.hint.as_deref().unwrap().contains("`color`"));
}

#[test]
fn invalid_values_explain_what_is_accepted() {
    let (_, compilation) = output("Texto {color=corall size=gigante}");
    let messages: Vec<&str> = compilation.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(messages.iter().any(|m| m.contains("cor desconhecida")));
    assert!(messages.iter().any(|m| m.contains("tamanho inválido")));
    assert!(compilation.diagnostics[0].hint.as_deref().unwrap().contains("`coral`"));
}

#[test]
fn unclosed_container_is_reported_at_its_opening_line() {
    let (value, compilation) = output("# A\n::: columns\nTexto\n---\n# B");
    let diagnostic = compilation.diagnostics.iter().find(|d| d.code == "R102").unwrap();
    assert_eq!(diagnostic.line, 2);
    assert_eq!(value["document"]["pages"].as_array().unwrap().len(), 2);
}

#[test]
fn stray_closing_fence_is_reported() {
    let (_, compilation) = output("Texto\n:::");
    assert_eq!(codes(&compilation), ["R101"]);
}

#[test]
fn separators_inside_code_fences_do_not_split_pages() {
    let value = clean("```yaml\n---\nkey: value\n```");
    let pages = value["document"]["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0]["children"][0]["code"], "---\nkey: value");
    assert_eq!(pages[0]["children"][0]["lang"], "yaml");
}

#[test]
fn dangerous_urls_are_rejected() {
    let (value, compilation) = output("[clique](javascript:alert(1))\n\n![x](javascript:alert(1))\n\nTexto {image=\"https://a.com/x.png);background:red\"}");
    assert_eq!(codes(&compilation).iter().filter(|code| **code == "R301").count(), 3);
    let first = &value["document"]["pages"][0]["children"][0]["content"];
    assert_eq!(first[0]["type"], "text");
    let json = serde_json::to_string(&value["document"]).unwrap();
    assert!(!json.contains("javascript:alert"));
}

#[test]
fn raw_html_is_plain_text() {
    let value = clean("<script>alert(1)</script>");
    assert_eq!(value["document"]["pages"][0]["children"][0]["content"][0]["value"], "<script>alert(1)</script>");
}

#[test]
fn notes_are_collected_and_not_rendered() {
    let value = clean("# Oi\n::: notes\nFale devagar.\n\nPausa.\n:::");
    let page = &value["document"]["pages"][0];
    assert_eq!(page["notes"], "Fale devagar.\n\nPausa.");
    assert_eq!(page["children"].as_array().unwrap().len(), 1);
}

#[test]
fn charts_carry_ready_to_draw_geometry() {
    let bar = clean("::: chart {values}\n| Mês | A |\n| --- | --- |\n| Jan | 10 |\n| Fev | 20 |\n:::");
    let scene = &bar["document"]["pages"][0]["children"][0]["scene"];
    assert_eq!((scene["width"].as_f64(), scene["height"].as_f64()), (Some(1200.0), Some(600.0)));
    let shapes = scene["shapes"].as_array().unwrap();
    let ticks: Vec<_> = shapes.iter().filter(|shape| shape["shape"] == "text" && shape["anchor"] == "end").map(|shape| shape["text"].as_str().unwrap()).collect();
    assert_eq!(ticks, ["0", "5", "10", "15", "20"]);
    let first_bar = shapes.iter().find(|shape| shape["shape"] == "rect").unwrap();
    assert_eq!(first_bar, &json!({ "shape": "rect", "x": 174.6, "y": 276.0, "w": 382.8, "h": 252.0, "radius": 8.0, "ink": { "kind": "color", "color": "#f2603d" }, "opacity": 1.0 }));
    assert!(shapes.iter().any(|shape| shape["shape"] == "text" && shape["text"] == "20" && shape["weight"] == 700));
    assert_eq!(shapes.iter().find(|shape| shape["shape"] == "line").unwrap()["ink"], json!({ "kind": "text" }));

    let pie = clean("::: chart {type=donut values}\n| Fatia | V |\n| --- | --- |\n| A | 1 |\n| B | 3 |\n:::");
    let scene = &pie["document"]["pages"][0]["children"][0]["scene"];
    assert_eq!(scene["height"], 664.0);
    let sectors: Vec<_> = scene["shapes"].as_array().unwrap().iter().filter(|shape| shape["shape"] == "sector").collect();
    assert_eq!(sectors.len(), 2);
    assert_eq!((sectors[0]["start"].as_f64().unwrap().round(), sectors[0]["end"].as_f64().unwrap().round(), sectors[1]["end"].as_f64().unwrap().round()), (-90.0, 0.0, 270.0));
    assert_eq!(sectors[0]["stroke"], json!({ "kind": "background" }));
    assert!(sectors[0]["inner"].as_f64().unwrap() > 0.0);
    let texts: Vec<_> = scene["shapes"].as_array().unwrap().iter().filter(|shape| shape["shape"] == "text").map(|shape| shape["text"].as_str().unwrap()).collect();
    assert_eq!(texts, ["25%", "75%", "A", "B"]);
}

#[test]
fn charts_read_series_from_a_table() {
    let value = clean("::: chart {type=line colors=\"coral blue\"}\n| Mês | Vendas | Custos |\n| --- | ---: | ---: |\n| Jan | 30 | 20,5 |\n| Fev | 42 | 21 |\n:::");
    let chart = &value["document"]["pages"][0]["children"][0];
    assert_eq!(chart["chart"], "line");
    assert_eq!(chart["labels"], json!(["Jan", "Fev"]));
    assert_eq!(chart["series"][1]["values"], json!([20.5, 21.0]));
    assert_eq!(chart["series"][0]["color"], "#f2603d");
    assert_eq!(chart["legend"], true);
}

#[test]
fn chart_cells_must_be_numbers() {
    let (_, compilation) = output("::: chart\n| A | B |\n|---|---|\n| x | abc |\n:::");
    let diagnostic = compilation.diagnostics.iter().find(|d| d.code == "R112").unwrap();
    assert_eq!(diagnostic.line, 4);
    assert_eq!(diagnostic.column, 7);
}

#[test]
fn tables_lists_quotes_and_icons() {
    let value = clean("| A | B |\n|:-:|--:|\n| 1 | 2 |\n\n1. um\n2. dois\n   - sub\n\n> Citação\n> longa\n\n:: icon rocket {size=3xl color=accent}");
    let children = value["document"]["pages"][0]["children"].as_array().unwrap().clone();
    assert_eq!(children[0]["aligns"], json!(["center", "right"]));
    assert_eq!(children[1]["ordered"], true);
    assert_eq!(children[1]["items"][1]["children"][0]["items"][0]["content"][0]["value"], "sub");
    assert_eq!(children[2]["type"], "quote");
    assert_eq!(children[3]["name"], "rocket");
}

#[test]
fn unknown_blocks_icons_and_themes_suggest_alternatives() {
    let (_, compilation) = output("@theme papper\n\n::: colums\n:::\n:: icon rockt");
    let hints: Vec<String> = compilation.diagnostics.iter().filter_map(|d| d.hint.clone()).collect();
    assert!(hints.iter().any(|hint| hint.contains("`paper`")));
    assert!(hints.iter().any(|hint| hint.contains("`columns`")));
    assert!(hints.iter().any(|hint| hint.contains("`rocket`")));
}

#[test]
fn definitions_after_the_first_page_are_errors() {
    let (_, compilation) = output("# Oi\n@style a {color=red}");
    assert_eq!(codes(&compilation), ["R103"]);
}

#[test]
fn user_styles_compose_and_validate_once() {
    let (value, compilation) = output("@style base {size=xl}\n@style hero {.base weight=black colr=red}\n\n# A {.hero}\n# B {.hero}");
    let errors: Vec<_> = compilation.diagnostics.iter().filter(|d| d.severity == Severity::Error).collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].line, 2);
    let heading = &value["document"]["pages"][0]["children"][1];
    assert_eq!(heading["style"]["size"], 52.0);
    assert_eq!(heading["style"]["weight"], 900);
}

#[test]
fn color_opacity_and_gradients() {
    let value = clean("::: box {bg=coral..blue border=\"2 coral/50\"}\n:::");
    let style = &value["document"]["pages"][0]["children"][0]["style"];
    assert_eq!(style["bg"], json!({ "kind": "linear", "angle": 135.0, "stops": [{ "color": "#f2603d", "at": 0.0 }, { "color": "#2f6fdb", "at": 1.0 }] }));
    assert_eq!(style["border"]["color"], "#f2603d80");
    assert_eq!(style["border"]["width"], 2.0);
}

#[test]
fn columns_are_reported_in_utf16_units() {
    let (_, compilation) = output("Ação 🚀 {colr=red}");
    assert_eq!(compilation.diagnostics[0].column, 10);
}

#[test]
fn compiler_never_panics_on_malformed_input() {
    let inputs = [
        "",
        "---",
        ":::",
        "::",
        "{",
        "# {",
        "[a]{",
        "![](",
        "**",
        "| a",
        "@component",
        "@component x {",
        "::: x {a=\"b}\n",
        "```",
        "- \n  - \n    -",
        "> > > x",
        "$content",
        "\u{feff}# Ç {.}",
        "[[[[]]]]{}",
    ];
    for input in inputs {
        let _ = compile(input);
    }
}

#[test]
fn compiler_survives_combinations_of_partial_syntax() {
    let pieces = [
        "ã", "🚀", "*", "**", "~~", "`", "[", "]", "(", ")", "{", "}", "!", "\\", "\n", " ", ":::", "::", "#", "- ", "1. ", "|", "> ",
        "$", "$x", "=", "\"", ".", "@", "---", "```", "{a=\"ç", "x",
    ];
    for a in pieces {
        for b in pieces {
            for c in pieces {
                let source = format!("{a}{b}{c}{a}");
                let _ = compile(&source);
                let _ = compile(&format!("@component x {{p}}\n{source}\n$content\n@end\n::: x {{p={a}}}\n{source}\n:::"));
            }
        }
    }
}

const COMPARISON: &str = "@component comparacao {antes depois}\n::: columns\n::: stack\n## $antes\n$content\n:::\n::: stack\n## $depois\n$lado-b\n:::\n:::\n@end\n\n";

#[test]
fn named_slots_receive_their_own_content() {
    let source = format!("{COMPARISON}::: comparacao {{antes=ANTES depois=DEPOIS}}\nPlanilhas\n::: slot lado-b\nPainel\n:::\n:::");
    let value = clean(&source);
    let columns = &value["document"]["pages"][0]["children"][0];
    let left = &columns["children"][0]["children"];
    let right = &columns["children"][1]["children"];
    assert_eq!(left[0]["content"][0]["value"], "ANTES");
    assert_eq!(left[1]["content"][0]["value"], "Planilhas");
    assert_eq!(right[0]["content"][0]["value"], "DEPOIS");
    assert_eq!(right[1]["content"][0]["value"], "Painel");
}

#[test]
fn slots_are_optional_and_unknown_slots_are_errors() {
    let empty = clean(&format!("{COMPARISON}::: comparacao {{antes=A depois=B}}\n:::"));
    assert_eq!(empty["document"]["pages"][0]["children"][0]["children"][1]["children"].as_array().unwrap().len(), 1);

    let (_, compilation) = output(&format!("{COMPARISON}::: comparacao {{antes=A depois=B}}\n::: slot lado-c\nX\n:::\n:::"));
    let diagnostic = compilation.diagnostics.iter().find(|d| d.code == "R414").unwrap();
    assert_eq!(diagnostic.line, 15);
    assert!(diagnostic.hint.as_deref().unwrap().contains("`lado-b`"));
}

#[test]
fn slot_misuse_is_reported() {
    let (_, compilation) = output(&format!(
        "{COMPARISON}::: comparacao {{antes=A depois=B}}\n::: slot lado-b\n:::\n::: slot lado-b\n:::\n::: slot\n:::\n:::\n\n::: slot solto\n:::"
    ));
    let found = codes(&compilation);
    assert!(found.contains(&"R417"), "{found:?}");
    assert!(found.contains(&"R415"), "{found:?}");
    assert!(found.contains(&"R416"), "{found:?}");
}

#[test]
fn parameters_alone_on_a_line_still_become_text() {
    let value = clean("@component titulo {texto}\n$texto\n@end\n\n:: titulo {texto=\"Olá **mundo**\"}");
    let paragraph = &value["document"]["pages"][0]["children"][0];
    assert_eq!(paragraph["type"], "paragraph");
    assert_eq!(paragraph["content"][1]["type"], "strong");
}

#[test]
fn dollar_words_outside_components_are_plain_text() {
    let value = clean("$valor");
    assert_eq!(value["document"]["pages"][0]["children"][0]["content"][0]["value"], "$valor");
}

const BRAND: &str = "@theme ink\n@color marca #12a37f\n@font titulo \"Bebas Neue\" 400\n@style destaque {color=marca weight=bold}\n@component selo {texto}\n::: box {bg=marca pad=sm}\n$texto\n:::\n@end\n";

#[test]
fn project_prelude_provides_theme_tokens_styles_and_components() {
    let compilation = ribalta::compile_with_prelude(BRAND, "# Oi {.destaque font=titulo}\n\n:: selo {texto=Novo}");
    assert!(compilation.diagnostics.is_empty(), "{:#?}", compilation.diagnostics);
    let value = serde_json::to_value(&compilation).unwrap();
    let document = &value["document"];
    assert_eq!(document["theme"]["name"], "ink");
    assert_eq!(document["pages"][0]["children"][0]["style"]["color"], "#12a37f");
    assert_eq!(document["pages"][0]["children"][0]["style"]["font"], "Bebas Neue");
    assert_eq!(document["pages"][0]["children"][1]["style"]["bg"], json!({ "kind": "solid", "color": "#12a37f" }));
}

#[test]
fn documents_override_the_project_prelude() {
    let source = "@theme paper\n@color marca #ff0000\n@component selo {texto}\n$texto\n@end\n\n# Oi {color=marca}\n\n:: selo {texto=X}";
    let compilation = ribalta::compile_with_prelude(BRAND, source);
    assert!(compilation.diagnostics.is_empty(), "{:#?}", compilation.diagnostics);
    let value = serde_json::to_value(&compilation).unwrap();
    assert_eq!(value["document"]["theme"]["name"], "paper");
    assert_eq!(value["document"]["pages"][0]["children"][0]["style"]["color"], "#ff0000");
    assert_eq!(value["document"]["pages"][0]["children"][1]["type"], "paragraph");
}

#[test]
fn prelude_errors_stay_out_of_document_diagnostics() {
    let broken = "@colr marca #12a37f\n@style ruim {colr=red}\n@theme nada";
    let compilation = ribalta::compile_with_prelude(broken, "# Oi");
    assert!(compilation.diagnostics.is_empty(), "{:#?}", compilation.diagnostics);
    assert_eq!(compilation.document.theme.name, "paper");

    let found: Vec<_> = ribalta::check_prelude(broken).iter().map(|d| (d.code, d.line)).collect();
    assert_eq!(found, [("R506", 1), ("R209", 2), ("R513", 3)]);
}

#[test]
fn prelude_rejects_pages_and_warns_about_titles() {
    let found: Vec<_> = ribalta::check_prelude("@title X\n@color a #fff\n\n# Page").iter().map(|d| (d.code, d.line)).collect();
    assert_eq!(found, [("R520", 1), ("R519", 4)]);
    assert!(ribalta::check_prelude(BRAND).is_empty());
}

#[test]
fn stored_assets_can_be_used_as_images_and_backgrounds() {
    let id = "3f2b8c1e-9d4a-4b7e-8f21-0a1b2c3d4e5f";
    let value = clean(&format!("--- {{image=asset:{id}}}\n![Foto](asset:{id}){{fit=cover}}"));
    let page = &value["document"]["pages"][0];
    let asset = json!({ "kind": "asset", "id": id });
    assert_eq!(page["style"]["bgImage"], asset);
    assert_eq!(page["children"][0]["src"], asset);

    let (_, compilation) = output("![x](asset:../../etc)\n\n[link](asset:3f2b8c1e-9d4a-4b7e-8f21-0a1b2c3d4e5f)");
    assert_eq!(codes(&compilation), ["R301", "R301"]);
}

#[test]
fn diagnostics_follow_the_requested_locale() {
    let source = "# Título {colr=red}\n\n::: colums\n:::";
    let english = ribalta::compile_localized("", source, ribalta::Locale::En);
    let messages: Vec<String> = english.diagnostics.iter().map(|d| format!("{} | {}", d.message, d.hint.clone().unwrap_or_default())).collect();
    assert!(messages.iter().any(|m| m.contains("unknown attribute: `colr`") && m.contains("Did you mean `color`?")), "{messages:?}");
    assert!(messages.iter().any(|m| m.contains("unknown block: `colums`")), "{messages:?}");

    let portuguese = ribalta::compile(source);
    assert!(portuguese.diagnostics.iter().any(|d| d.message.contains("atributo desconhecido")));
    assert_eq!(english.diagnostics.iter().map(|d| (d.code, d.line, d.column)).collect::<Vec<_>>(), portuguese.diagnostics.iter().map(|d| (d.code, d.line, d.column)).collect::<Vec<_>>());

    let theme = ribalta::check_prelude_localized("# heading", ribalta::Locale::En);
    assert!(theme[0].message.contains("can only contain definitions"));
    assert!(ribalta::compile_localized("@component c {x}\n::: box {colr=1}\n:::\n@end\n\n:: c {x=1}", "", ribalta::Locale::En).diagnostics.is_empty());
    let nested = ribalta::compile_localized("", "@component c {x}\n::: box {colr=1}\n:::\n@end\n\n:: c {x=1}", ribalta::Locale::En);
    assert!(nested.diagnostics[0].message.ends_with("(in component `c`)"), "{:?}", nested.diagnostics);
}

#[test]
fn every_message_has_an_english_version() {
    let samples = [
        "{.}", "{a=\"b}", "{a=}", "{!}", ":::", "::: x", "```", "| a |\n| b |", "@title", "@color", "@style x", "@component", "@nope",
        "@font a \"B@d\"", "@font a \"B\" 150", "{size=gigante}", "{pad=\"a b c\"}", "{color=x/200}", "{color=#zz}", "{color=nope}",
        "{font=nope}", "{weight=fat}", "{italic=talvez}", "{align=up}", "{valign=up}", "{shadow=huge}", "{border=\"99 x\"}",
        "{w=200%}", "{w=abc}", "{opacity=3}", "{leading=9}", "{tracking=9}", "[a](javascript:x)", "@theme nope", "@version 9",
        "@page 2:1", "@color Bad #fff", "@font Bad \"A\"", "@style Bad {a=1}", "@component stack\n@end", "--- {#a}\n---{#a}",
        "{#id}", "@style a {.a}\n\n{.a}", "{.nope}", "{step=0}", "{nope}", "![a](https://x.com/a.png){fit=stretch}", "$content",
        ":: icon\n\n:: icon nope\n\n:: spacer huge\n\n::: divider x\n:::", "::: slot x\n:::", "::: nope\n:::",
        "::: grid {cols=99}\n:::\n\n::: columns {widths=\"a\"}\n:::\n\n::: columns {widths=\"1 2\"}\nA\n:::", "::: chart\n:::",
        "::: chart {type=nope}\n| a | b |\n|---|---|\n| x | y |\n:::", "::: chart\n| a |\n|---|\n| x |\n:::",
        "@component card\n# A", "@end", "::: {.a}\n:::",
        "@footer\n$pagina\n@end\n\n# A\n---\n# B", "@header\nx", "@footer x\n@end", "@footer\na\n@end\n@footer\nb\n@end",
        "@footer\n::: steps\n- a\n:::\n::: notes\nn\n:::\n@end\n\n# A\n---\n# B", "@footer\na\n@end\n\n# A\n--- {footer=talvez}\n# B",
        "@overlay\n$pagina\n@end\n\n# A\n---\n# B", "@overlay x\n@end", "@overlay\nx", "@overlay\na\n@end\n\n# A\n--- {overlay=talvez}\n# B",
    ];
    for sample in samples {
        let source = if sample.starts_with('{') || sample.starts_with('[') || sample.starts_with('!') { format!("Texto {sample}") } else { sample.to_string() };
        for diagnostic in ribalta::compile_localized("", &source, ribalta::Locale::En).diagnostics {
            let text = format!("{} {}", diagnostic.message, diagnostic.hint.unwrap_or_default());
            assert!(!text.chars().any(|c| "áàâãéêíóôõúç".contains(c)), "Portuguese leaked into English for {sample:?}: {text}");
            for word in [" não ", " use um", "você", " inválid", "desconhecid", " precisa ", " sem ", "exemplos"] {
                assert!(!text.to_lowercase().contains(word.trim_start()) || word.trim().is_empty(), "Portuguese leaked for {sample:?}: {text}");
            }
        }
    }
}

fn texts(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let own = map.get("value").and_then(Value::as_str).filter(|_| map.get("type").is_some_and(|kind| kind == "text")).unwrap_or("");
            own.to_string() + &map.iter().filter(|(key, _)| *key != "value").map(|(_, nested)| texts(nested)).collect::<String>()
        }
        Value::Array(items) => items.iter().map(texts).collect(),
        _ => String::new(),
    }
}

#[test]
fn header_and_footer_repeat_on_every_page_but_the_cover_with_fields_filled() {
    let value = clean("@title Pitch\n@author Ana\n\n@header\nLumen\n@end\n\n@footer\n$title · $author · $page / $pages\n@end\n\n# Capa\n---\n# Dois\n---\n# Três");
    let pages = value["document"]["pages"].as_array().unwrap();
    assert!(pages[0].get("header").is_none());
    assert!(pages[0].get("footer").is_none());
    assert_eq!(texts(&pages[1]["header"]), "Lumen");
    assert_eq!(texts(&pages[1]["footer"]), "Pitch · Ana · 2 / 3");
    assert_eq!(texts(&pages[2]["footer"]), "Pitch · Ana · 3 / 3");
    assert_eq!(pages[1]["footer"]["type"], "layout");
    assert!(pages[1]["footer"]["style"]["size"].is_number());
    assert!(pages[1]["footer"]["style"]["color"].is_string());
    assert_eq!(pages[1]["footer"]["line"], 8);
    assert_eq!(texts(&pages[1]["children"]), "Dois");
}

#[test]
fn covers_and_per_page_switches_decide_where_regions_appear() {
    let value = clean("@footer\nRodapé\n@end\n\n--- {cover=false}\n# A\n--- {footer=off}\n# B\n--- {cover}\n# C\n--- {cover footer=on}\n# D\n---\n# E");
    let shown: Vec<bool> = value["document"]["pages"].as_array().unwrap().iter().map(|page| page.get("footer").is_some()).collect();
    assert_eq!(shown, [true, false, false, true, true]);
}

#[test]
fn region_attributes_style_the_band_and_date_is_left_for_the_renderer() {
    let value = clean("@footer {align=center color=accent}\nHoje: $date\n@end\n\n# A\n---\n# B");
    let footer = &value["document"]["pages"][1]["footer"];
    assert_eq!(footer["style"]["align"], "center");
    let paragraph = &footer["children"][0]["content"];
    assert_eq!(paragraph[0], json!({ "type": "text", "value": "Hoje: " }));
    assert_eq!(paragraph[1], json!({ "type": "field", "name": "date" }));
}

#[test]
fn base_theme_regions_apply_until_the_document_replaces_or_disables_them() {
    let prelude = "@footer\nEmpresa · $page\n@end";
    let inherited = ribalta::compile_with_prelude(prelude, "# A\n---\n# B");
    assert!(inherited.diagnostics.is_empty(), "{:#?}", inherited.diagnostics);
    let footer = serde_json::to_value(&inherited.document.pages[1].footer).unwrap();
    assert_eq!(texts(&footer), "Empresa · 2");
    assert_eq!(footer["line"], 0);

    let replaced = ribalta::compile_with_prelude(prelude, "@footer\nPróprio\n@end\n\n# A\n---\n# B");
    assert_eq!(texts(&serde_json::to_value(&replaced.document.pages[1].footer).unwrap()), "Próprio");

    let disabled = ribalta::compile_with_prelude(prelude, "@footer none\n\n# A\n---\n# B");
    assert!(disabled.diagnostics.is_empty(), "{:#?}", disabled.diagnostics);
    assert!(disabled.document.pages[1].footer.is_none());

    let broken_theme = ribalta::compile_with_prelude("@footer\n$pagina\n@end", "# A\n---\n# B");
    assert!(broken_theme.diagnostics.is_empty());
    assert_eq!(ribalta::check_prelude("@footer\n$pagina\n@end").iter().map(|d| d.code).collect::<Vec<_>>(), ["R522"]);
}

#[test]
fn region_mistakes_are_reported_once_with_helpful_hints() {
    let (_, unknown) = output("@footer\n$pagina de $pages\n@end\n\n# A\n---\n# B\n---\n# C");
    assert_eq!(codes(&unknown), ["R522"]);
    assert_eq!(unknown.diagnostics[0].line, 1);
    assert_eq!(unknown.diagnostics[0].hint.as_deref(), Some("Você quis dizer `$page`?"));
    for (typed, expected) in [("titulo", "$title"), ("autor", "$author"), ("data", "$date"), ("paginas", "$pages")] {
        let (_, aliased) = output(&format!("@footer\n${typed}\n@end\n\n# A\n---\n# B"));
        assert_eq!(aliased.diagnostics[0].hint.as_deref(), Some(format!("Você quis dizer `{expected}`?").as_str()), "{typed}");
    }

    let (_, only_cover) = output("@header\n$autor\n@end\n\n# Só a capa");
    assert_eq!(codes(&only_cover), ["R522"]);

    let (_, unclosed) = output("@footer\nRodapé");
    assert_eq!(codes(&unclosed), ["R504"]);
    assert!(unclosed.diagnostics[0].message.contains("@footer"));

    let (_, twice) = output("@footer\na\n@end\n@footer\nb\n@end\n\n# A");
    assert_eq!(codes(&twice), ["R521"]);

    let (_, stray) = output("@footer Rodapé\n@end\n\n# A");
    assert_eq!(codes(&stray), ["R525"]);

    let (_, switch) = output("@footer\na\n@end\n\n# A\n--- {footer=talvez}\n# B");
    assert_eq!(codes(&switch), ["R301"]);
}

#[test]
fn regions_never_reveal_step_by_step_nor_carry_notes() {
    let (value, compilation) = output("@footer\n::: steps\n- a\n- b\n:::\n::: notes\nsegredo\n:::\n@end\n\n# A\n---\n# B");
    assert_eq!(codes(&compilation), ["R523", "R524"]);
    let page = &value["document"]["pages"][1];
    assert_eq!(page["steps"], 0);
    assert!(page["notes"].is_null());
    assert!(!serde_json::to_string(&page["footer"]).unwrap().contains("\"step\""));
}

#[test]
fn overlay_floats_over_every_page_but_the_cover_without_the_band_defaults() {
    let value = clean("@title Pitch\n\n@overlay {align=right}\nRascunho {x=1500 y=40}\n\n$page / $pages\n@end\n\n# Capa\n---\n# Dois\n--- {overlay=off}\n# Três\n--- {cover overlay=on}\n# Quatro");
    let pages = value["document"]["pages"].as_array().unwrap();
    let shown: Vec<bool> = pages.iter().map(|page| page.get("overlay").is_some()).collect();
    assert_eq!(shown, [false, true, false, true]);
    let overlay = &pages[1]["overlay"];
    assert_eq!(texts(overlay), "Rascunho2 / 4");
    assert_eq!(overlay["type"], "layout");
    assert_eq!(overlay["style"]["align"], "right");
    assert!(overlay["style"].get("size").is_none());
    assert!(overlay["style"].get("color").is_none());
    assert_eq!(overlay["children"][0]["style"]["x"], json!({ "unit": "px", "value": 1500.0 }));
    assert_eq!(overlay["line"], 3);
    assert!(pages[1].get("header").is_none() && pages[1].get("footer").is_none());
}

#[test]
fn base_theme_overlay_applies_until_replaced_or_disabled() {
    let prelude = "@overlay\nLogo\n@end";
    let inherited = ribalta::compile_with_prelude(prelude, "# A\n---\n# B");
    assert!(inherited.diagnostics.is_empty(), "{:#?}", inherited.diagnostics);
    assert_eq!(texts(&serde_json::to_value(&inherited.document.pages[1].overlay).unwrap()), "Logo");

    let replaced = ribalta::compile_with_prelude(prelude, "@overlay\nPróprio\n@end\n\n# A\n---\n# B");
    assert_eq!(texts(&serde_json::to_value(&replaced.document.pages[1].overlay).unwrap()), "Próprio");

    let disabled = ribalta::compile_with_prelude(prelude, "@overlay none\n\n# A\n---\n# B");
    assert!(disabled.diagnostics.is_empty(), "{:#?}", disabled.diagnostics);
    assert!(disabled.document.pages[1].overlay.is_none());

    assert_eq!(ribalta::check_prelude("@overlay\n$pagina\n@end").iter().map(|d| d.code).collect::<Vec<_>>(), ["R522"]);
}

#[test]
fn overlay_mistakes_follow_the_region_rules() {
    let (_, unknown) = output("@overlay\n$pagina\n@end\n\n# A\n---\n# B");
    assert_eq!(codes(&unknown), ["R522"]);
    assert!(unknown.diagnostics[0].message.contains("@overlay"));

    let (_, unclosed) = output("@overlay\nSelo");
    assert_eq!(codes(&unclosed), ["R504"]);

    let (_, twice) = output("@overlay\na\n@end\n@overlay\nb\n@end\n\n# A");
    assert_eq!(codes(&twice), ["R521"]);

    let (_, stray) = output("@overlay Selo\n@end\n\n# A");
    assert_eq!(codes(&stray), ["R525"]);

    let (_, switch) = output("@overlay\na\n@end\n\n# A\n--- {overlay=talvez}\n# B");
    assert_eq!(codes(&switch), ["R301"]);

    let (value, steps) = output("@overlay\n::: steps\n- a\n:::\n::: notes\nn\n:::\n@end\n\n# A\n---\n# B");
    assert_eq!(codes(&steps), ["R523", "R524"]);
    assert_eq!(value["document"]["pages"][1]["steps"], 0);
}

#[test]
fn underline_is_a_boolean_style() {
    let value = clean("Texto [sublinhado]{underline} {underline=false}");
    let paragraph = &value["document"]["pages"][0]["children"][0];
    assert_eq!(paragraph["style"]["underline"], false);
    assert_eq!(paragraph["content"][1]["style"]["underline"], true);
}

#[test]
fn spans_keep_their_raw_attributes() {
    let value = clean("@style destaque {color=accent}\n\nCresceu [38%]{ .destaque  weight=bold } e ![logo](https://x.com/a.png){h=60}");
    let content = &value["document"]["pages"][0]["children"][0]["content"];
    assert_eq!(content[1]["sourceAttrs"], ".destaque  weight=bold");
    assert_eq!(content[3]["sourceAttrs"], "h=60");
}

#[test]
fn text_blocks_written_in_the_page_carry_their_source_range() {
    let value = clean("## Título {size=5xl}\n\nLinha um\n  linha dois {.lead}\n\n- item {color=accent}\n  continua\n  - aninhado");
    let children = &value["document"]["pages"][0]["children"];
    assert_eq!(children[0]["source"], json!({"line": 1, "spans": [[3, 10]]}));
    assert_eq!(children[1]["source"], json!({"line": 3, "spans": [[0, 8], [2, 12]]}));
    let item = &children[2]["items"][0];
    assert_eq!(item["source"], json!({"line": 6, "spans": [[2, 6], [2, 10]]}));
    assert_eq!(item["children"][0]["items"][0]["source"], json!({"line": 8, "spans": [[4, 12]]}));
}

#[test]
fn text_from_components_regions_and_quotes_has_no_source_range() {
    let value = clean("@component card {titulo}\n## $titulo\n@end\n@footer\nRodapé\n@end\n\n:: card {titulo=Oi}\n\n> citação\n---\nSegundo");
    let pages = &value["document"]["pages"];
    assert!(pages[0]["children"][0]["source"].is_null());
    assert!(pages[0]["children"][1]["children"][0]["source"].is_null());
    assert!(pages[1]["footer"]["children"][0]["source"].is_null());
    assert!(!pages[1]["children"][0]["source"].is_null());
}

#[test]
fn page_blocks_report_where_they_end_in_the_source() {
    let value = clean("# Título\nlinha um\nlinha dois\n\n- a\n- b\n  - c\n\n::: columns\nEsquerda\n\nDireita\n:::\n\n```js\nx\n```\n\n:: icon star");
    let children = value["document"]["pages"][0]["children"].as_array().unwrap();
    let ranges: Vec<(u64, u64)> = children.iter().map(|block| (block["line"].as_u64().unwrap(), block["endLine"].as_u64().unwrap())).collect();
    assert_eq!(ranges, vec![(1, 1), (2, 3), (5, 7), (9, 13), (15, 17), (19, 19)]);
    let cells = children[3]["children"].as_array().unwrap();
    assert_eq!(cells[1]["endLine"], 12);
}

#[test]
fn component_internals_do_not_report_end_lines() {
    let value = clean("@component card\n::: box\n# Dentro\n:::\n@end\n\n::: card\n:::");
    let card = &value["document"]["pages"][0]["children"][0];
    assert!(card["children"][0].get("endLine").is_none());
}

#[test]
fn pages_default_to_a_fixed_width_and_automatic_height() {
    let value = clean("# Oi");
    assert_eq!(value["document"]["meta"]["page"], json!({ "width": 1920, "height": null }));
    assert_eq!((value["document"]["pages"][0]["width"].clone(), value["document"]["pages"][0]["height"].clone()), (json!(1920), Value::Null));
}

#[test]
fn page_size_follows_host_then_prelude_then_document_then_separator() {
    let host = ribalta::CompileOptions { page: ribalta::PageSize { width: 1920, height: Some(1080) } };
    let size = |prelude: &str, source: &str| {
        let compilation = ribalta::compile_with_options(prelude, source, ribalta::Locale::Pt, host);
        assert!(compilation.diagnostics.is_empty(), "{:#?}", compilation.diagnostics);
        compilation.document.pages.iter().map(|page| (page.width, page.height)).collect::<Vec<_>>()
    };
    assert_eq!(size("", "# A"), [(1920, Some(1080))]);
    assert_eq!(size("@page 1440x1080", "# A"), [(1440, Some(1080))]);
    assert_eq!(size("@page 1440x1080", "@page width=800 height=auto\n\n# A"), [(800, None)]);
    assert_eq!(size("", "@page width=1080\n\n# A\n--- {height=1920}\n# B"), [(1080, Some(1080)), (1080, Some(1920))]);
}

#[test]
fn invalid_page_sizes_are_reported() {
    for source in ["@page 16:9\n\n# A", "@page width=abc\n\n# A", "@page 99999x10\n\n# A", "--- {width=0}\n# A", "--- {height=tall}\n# A"] {
        let (_, compilation) = output(source);
        assert_eq!(codes(&compilation), ["R515"], "{source}");
    }
}

#[test]
fn block_anchors_are_emitted_on_blocks_and_collected_per_page() {
    let value = clean("# Fotossíntese {#fotossintese}\n\nA fase clara acontece nos tilacoides. {#fase-clara}\n\n- Clorofila absorve luz {#clorofila}\n- Outro\n---\nSegunda {#segunda}");
    let document = &value["document"];
    assert_eq!(document["pages"][0]["children"][0]["id"], "fotossintese");
    assert_eq!(document["pages"][0]["children"][1]["id"], "fase-clara");
    assert_eq!(document["pages"][0]["children"][2]["items"][0]["id"], "clorofila");
    assert!(document["pages"][0]["children"][2]["items"][1].get("id").is_none());
    assert_eq!(document["anchors"], json!({ "fotossintese": 0, "fase-clara": 0, "clorofila": 0, "segunda": 1 }));
}

#[test]
fn documents_without_anchors_do_not_emit_the_anchors_field() {
    let value = clean("# Oi");
    assert!(value["document"].get("anchors").is_none());
    assert!(value["document"]["pages"][0]["children"][0].get("id").is_none());
}

#[test]
fn repeated_block_anchor_is_an_error() {
    let (_, compilation) = output("Um {#a}\n\nDois {#a}");
    assert_eq!(codes(&compilation), vec!["R211"]);
    assert_eq!(compilation.diagnostics[0].severity, Severity::Error);
}

#[test]
fn block_anchor_equal_to_a_page_id_is_an_error() {
    let (_, compilation) = output("--- {#a}\nUm\n--- {#b}\nDois {#a}");
    assert_eq!(codes(&compilation), vec!["R211"]);
}

#[test]
fn links_and_go_resolve_block_anchors() {
    let (_, compilation) = output("Alvo {#a}\n---\n[x](#a)\n\n[ir]{click=\"go #a\"}");
    assert!(compilation.diagnostics.is_empty(), "{:#?}", compilation.diagnostics);
}

#[test]
fn unknown_link_target_still_reports_r701() {
    let (_, compilation) = output("[x](#nada)");
    assert_eq!(codes(&compilation), vec!["R701"]);
}

#[test]
fn anchor_syntax_in_the_middle_of_a_line_stays_literal() {
    let value = clean("Texto {#a} mais texto");
    let block = &value["document"]["pages"][0]["children"][0];
    assert!(block.get("id").is_none());
    assert!(value["document"].get("anchors").is_none());
    assert_eq!(block["content"][0]["value"], "Texto {#a} mais texto");
}
