use ribalta::{Compilation, Severity, compile, compile_with_prelude, library};
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

fn children(value: &Value, page: usize) -> Vec<Value> {
    value["document"]["pages"][page]["children"].as_array().unwrap().clone()
}

fn text(block: &Value) -> String {
    block["content"].as_array().unwrap().iter().map(|inline| inline["value"].as_str().unwrap_or("")).collect()
}

#[test]
fn let_values_fill_text_attributes_styles_and_colors() {
    let value = clean(
        "@let compacto = true\n@let espaco = $compacto ? sm : lg\n@let nome = \"Ana Souza\"\n@let base = coral\n@color marca $base\n@style cartao {.card pad=$espaco}\n\n# Olá, $nome. {color=marca}\n::: box {.cartao}\nTotal: $(2 * 21)\n:::",
    );
    let blocks = children(&value, 0);
    assert_eq!(text(&blocks[0]), "Olá, Ana Souza.");
    assert_eq!(blocks[0]["style"]["color"], "#f2603d");
    assert_eq!(blocks[1]["style"]["pad"], json!([16.0, 16.0]));
    assert_eq!(text(&blocks[1]["children"][0]), "Total: 42");
}

#[test]
fn undeclared_dollar_words_stay_as_text() {
    let value = clean("@let nome = x\n\nCusta US$ 10, $100 ou $talvez. Escape: \\$nome");
    assert_eq!(text(&children(&value, 0)[0]), "Custa US$ 10, $100 ou $talvez. Escape: $nome");
}

#[test]
fn interpolated_text_is_not_inline_editable() {
    let value = clean("@let nome = Ana\n\n# Olá $nome\n# Olá mundo");
    let blocks = children(&value, 0);
    assert!(blocks[0].get("source").is_none());
    assert!(blocks[1].get("source").is_some());
}

#[test]
fn data_tables_drive_for_loops_inside_layouts() {
    let value = clean(
        "@data equipe\n| nome | cargo | anos |\n| --- | --- | --- |\n| Ana | CTO | 7 |\n| Bruno | Design | 3 |\n@end\n\n::: grid {cols=2}\n::: for p, i in $equipe\n::: box\n### $i. $p.nome\n$p.cargo ($(round($p.anos / 2, 1)))\n:::\n:::\n:::",
    );
    let grid = &children(&value, 0)[0];
    let cells = grid["children"].as_array().unwrap();
    assert_eq!(cells.len(), 2);
    assert_eq!(text(&cells[0]["children"][0]), "1. Ana");
    assert_eq!(text(&cells[1]["children"][1]), "Design (1.5)");
}

#[test]
fn lists_generated_by_a_loop_are_merged() {
    let value = clean("@let etapas = [Ideia, Protótipo, Lançamento]\n\n::: for t, i in $etapas\n1. $t {step=$i}\n:::");
    let blocks = children(&value, 0);
    assert_eq!(blocks.len(), 1);
    let items = blocks[0]["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[2]["step"], 3);
    assert_eq!(value["document"]["pages"][0]["steps"], 3);
}

#[test]
fn ranges_repeat_blocks() {
    let value = clean("::: row\n::: for n in 1..4\n:: icon star\n:::\n:::");
    assert_eq!(children(&value, 0)[0]["children"].as_array().unwrap().len(), 4);
}

#[test]
fn static_conditions_choose_a_single_branch() {
    let value = clean(
        "@let publico = executivo\n@let extra = false\n\n::: if $publico == tecnico\nA\n:::\n::: else if $publico == executivo\nB\n:::\n::: else\nC\n:::\nD {if=$extra}\nE {if=\"not $extra\"}\n- um\n- dois {if=$extra}",
    );
    let blocks = children(&value, 0);
    let texts: Vec<String> = blocks.iter().filter(|block| block["type"] == "paragraph").map(text).collect();
    assert_eq!(texts, ["B", "E"]);
    assert_eq!(blocks.last().unwrap()["items"].as_array().unwrap().len(), 1);
}

#[test]
fn state_conditions_keep_every_branch_with_a_runtime_condition() {
    let value = clean("@state modo = a\n\n::: if $modo == a\nA\n:::\n::: else if $modo == b\nB\n:::\n::: else\nC\n:::\n[x]{click=\"$modo = b\"}\n[y]{click=\"$modo = c\"}");
    let document = &value["document"];
    assert_eq!(document["state"], json!({"modo": "a"}));
    let blocks = children(&value, 0);
    let is_a = json!({"op": "binary", "operator": "==", "left": {"op": "state", "name": "modo"}, "right": {"op": "lit", "value": "a"}});
    let is_b = json!({"op": "binary", "operator": "==", "left": {"op": "state", "name": "modo"}, "right": {"op": "lit", "value": "b"}});
    assert_eq!(blocks[0]["when"], is_a);
    assert_eq!(blocks[1]["when"], json!({"op": "binary", "operator": "and", "left": {"op": "not", "arg": is_a}, "right": is_b}));
    assert_eq!(blocks[2]["when"], json!({"op": "not", "arg": {"op": "binary", "operator": "or", "left": is_a, "right": is_b}}));
}

#[test]
fn variants_actions_and_dynamic_text() {
    let value = clean(
        "@state ativo = false\n@state total = 1\n@style destaque {color=coral weight=bold}\n\n::: box {.destaque=$ativo click=\"toggle $ativo; $total = $total + 1; go #fim\"}\nCliques: $total, dobro: $($total * 2)\n:::\n--- {#fim}\nFim",
    );
    let block = &children(&value, 0)[0];
    assert_eq!(block["variants"], json!([{"when": {"op": "state", "name": "ativo"}, "style": {"color": "#f2603d", "weight": 700}}]));
    assert_eq!(
        block["click"],
        json!([
            {"type": "toggle", "name": "ativo"},
            {"type": "set", "name": "total", "value": {"op": "binary", "operator": "+", "left": {"op": "state", "name": "total"}, "right": {"op": "lit", "value": 1.0}}},
            {"type": "go", "target": "fim"}
        ])
    );
    let content = block["children"][0]["content"].as_array().unwrap();
    assert_eq!(content[0], json!({"type": "text", "value": "Cliques: "}));
    assert_eq!(content[1], json!({"type": "expr", "expr": {"op": "state", "name": "total"}}));
    assert_eq!(content[3]["expr"]["operator"], "*");
}

#[test]
fn step_comparisons_extend_the_page_and_highlight_items() {
    let value = clean("@style apagado {opacity=0.4}\n\n- A {step=1 .apagado=\"$step > 1\"}\n- B {step=2}\nSó na etapa 4 {if=\"$step == 4\"}");
    let page = &value["document"]["pages"][0];
    assert_eq!(page["steps"], 4);
    assert_eq!(page["children"][0]["items"][0]["variants"][0]["when"]["left"], json!({"op": "step"}));
}

#[test]
fn page_navigation_attributes_and_local_state() {
    let value = clean("@transition fade\n\n# Início\n[ver](#extra)\n--- {#extra hidden next=#fim transition=zoom}\n@state aberto = false\n# Extra\n[abrir]{click=\"toggle $aberto\"}\n--- {#fim steps=2}\n# Fim");
    let pages = value["document"]["pages"].as_array().unwrap();
    assert_eq!(pages[0]["transition"], "fade");
    assert_eq!(pages[1]["hidden"], true);
    assert_eq!(pages[1]["next"], "fim");
    assert_eq!(pages[1]["transition"], "zoom");
    assert_eq!(pages[1]["state"], json!({"aberto": false}));
    assert_eq!(pages[1]["title"], "Extra");
    assert_eq!(pages[2]["steps"], 2);
    assert!(value["document"].get("state").is_none());
}

#[test]
fn components_receive_records_and_validate_variants() {
    let source = "@data planos\n| id | nome | preco |\n| --- | --- | --- |\n| pro | Plano Pro | 49 |\n@end\n@component cartao {p tipo=claro|escuro}\n::: box {.cartao-$tipo}\n## $p.nome\nR$ $($p.preco * 12)\n:::\n@end\n@style cartao-claro {bg=white}\n@style cartao-escuro {bg=ink}\n\n::: for p in $planos\n:: cartao {p=$p}\n:: cartao {p=$p tipo=escuro}\n:::";
    let value = clean(source);
    let blocks = children(&value, 0);
    assert_eq!(text(&blocks[0]["children"][0]), "Plano Pro");
    assert_eq!(text(&blocks[0]["children"][1]), "R$ 588");
    assert_ne!(blocks[0]["style"]["bg"], blocks[1]["style"]["bg"]);
    let (_, compilation) = output(&source.replace("tipo=escuro}", "tipo=escur}"));
    assert_eq!(codes(&compilation), ["R418"]);
}

#[test]
fn component_usage_accepts_if_and_click() {
    let value = clean("@state n = 0\n@component selo {texto}\n**$texto**\n@end\n\n:: selo {texto=A if=\"$n > 0\" click=\"$n = 0\"}\n:: selo {texto=B if=false}");
    let blocks = children(&value, 0);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0]["when"]["operator"], ">");
    assert_eq!(blocks[0]["click"][0]["type"], "set");
}

#[test]
fn charts_and_tables_read_data() {
    let value = clean("@data vendas\n| mes | receita | custo |\n| --- | --- | --- |\n| Jan | 10 | 4 |\n| Fev | 12,5 | 5 |\n@end\n\n:: chart {data=$vendas type=line}\n:: chart {data=$vendas series=custo}\n:: table {data=$vendas columns=\"mes receita\"}");
    let blocks = children(&value, 0);
    assert_eq!(blocks[0]["labels"], json!(["Jan", "Fev"]));
    assert_eq!(blocks[0]["series"].as_array().unwrap().len(), 2);
    assert_eq!(blocks[0]["series"][0]["values"], json!([10.0, 12.5]));
    assert_eq!(blocks[1]["series"].as_array().unwrap().len(), 1);
    assert_eq!(blocks[2]["type"], "table");
    assert_eq!(blocks[2]["aligns"], json!(["left", "right"]));
    assert_eq!(blocks[2]["rows"][1][1], json!([{"type": "text", "value": "12.5"}]));
}

#[test]
fn prelude_lets_are_tokens_that_documents_can_override() {
    let compilation = compile_with_prelude("@let raio = sm\n@let fundo = paper", "@let raio = lg\n\n::: box {radius=$raio bg=$fundo}\n:::");
    assert!(compilation.diagnostics.is_empty(), "{:#?}", compilation.diagnostics);
    let value = serde_json::to_value(&compilation).unwrap();
    assert_eq!(value["document"]["pages"][0]["children"][0]["style"]["radius"], 28.0);
}

#[test]
fn expression_and_scope_errors_are_reported_before_rendering() {
    let cases: &[(&str, &[&str])] = &[
        ("# $(1 +)", &["R603"]),
        ("@let a = 1\n\n# $(a == )", &["R603"]),
        ("::: if $nada\nx\n:::", &["R601"]),
        ("@let a = 1\n\n::: if $a\nx\n:::", &["R602"]),
        ("@let a = [1]\n\n# $($a.campo)", &["R602"]),
        ("@state s = x\n\n# Oi {color=$s}", &["R604"]),
        ("@state s = x\n@let t = $s", &["R604"]),
        ("@let a = 1\n\n[x]{click=\"$a = 2\"}", &["R605"]),
        ("@let a = 1\n\n::: for x in $a\n:::", &["R606"]),
        ("::: for x in 1..900\n:::", &["R606"]),
        ("::: else\nx\n:::", &["R607"]),
        ("@state s = a\n\n::: if $s == b\nx\n:::", &["R608"]),
        ("@let 1x = 2", &["R609"]),
        ("@let a = 1\n@state a = 2", &["R610"]),
        ("@let step = 1", &["R610"]),
        ("@state n = 1\n\n[x]{click=\"$n = sim\"}", &["R602"]),
        ("[x]{click=\"pular\"}", &["R611"]),
        ("[x]{click=\"go #nada\"}", &["R701"]),
        ("--- {#a hidden}\nA", &["R702"]),
        ("@data d\nsem tabela\n@end", &["R609"]),
    ];
    for (source, expected) in cases {
        let (_, compilation) = output(source);
        assert_eq!(&codes(&compilation), expected, "source: {source}\n{:#?}", compilation.diagnostics);
    }
}

#[test]
fn broken_internal_links_are_warnings_but_go_targets_are_errors() {
    let (_, compilation) = output("[a](#nada)\n[b]{click=\"go #nada\"}");
    let severities: Vec<Severity> = compilation.diagnostics.iter().map(|d| d.severity).collect();
    assert_eq!(severities, [Severity::Warning, Severity::Error]);
}

#[test]
fn library_tracks_variables_as_definitions_with_dependencies() {
    let source = "@let raio = lg\n@data equipe\n| nome |\n| --- |\n| Ana |\n@end\n@component cartao {titulo}\n::: box {radius=$raio}\n## $titulo\n:::\n@end\n@component lista {}\n::: for p in $equipe\n:: cartao {titulo=$p.nome}\n:::\n@end\n";
    let definitions = library::definitions(source);
    let data = definitions.iter().find(|definition| definition.name == "equipe").unwrap();
    assert_eq!((data.start_line, data.end_line), (2, 6));
    let lista = definitions.iter().find(|definition| definition.name == "lista").unwrap();
    assert!(lista.deps.iter().any(|dep| dep.name == "equipe"));
    let extraction = library::extract(source, &[library::DefRef { kind: library::DefKind::Component, name: "cartao".into() }]);
    assert!(extraction.source.contains("@let raio = lg"), "{}", extraction.source);
    assert!(extraction.diagnostics.is_empty(), "{:#?}", extraction.diagnostics);
}

fn literal(value: &Value) -> String {
    match value {
        Value::String(text) => serde_json::to_string(text).unwrap(),
        Value::Array(items) => format!("[{}]", items.iter().map(literal).collect::<Vec<_>>().join(", ")),
        other => other.to_string(),
    }
}

#[test]
fn static_evaluation_matches_the_shared_expression_fixtures() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/expressions.json")).unwrap();
    for case in cases {
        let definitions: String = case["values"].as_object().unwrap().iter().map(|(name, value)| format!("@let {name} = {}\n", literal(value))).collect();
        let source = format!("{definitions}\n# $({})", case["expr"].as_str().unwrap());
        let value = clean(&source);
        assert_eq!(text(&children(&value, 0)[0]), case["expected"].as_str().unwrap(), "{source}");
    }
}

#[test]
fn merging_a_snippet_can_rename_a_conflicting_variable() {
    let theme = "@let raio = sm\n";
    let snippet = "@let raio = lg\n@component cartao {titulo}\n::: box {radius=$raio}\n## $titulo\n:::\n@end\n";
    let conflicts = library::conflicts(theme, snippet);
    assert_eq!(conflicts.len(), 1);
    let resolutions = [library::Resolution::Rename { kind: library::DefKind::Variable, name: "raio".into(), to: "raio-cartao".into() }];
    let merged = library::merge(theme, snippet, &resolutions, None).expect("merge");
    assert!(merged.source.contains("@let raio-cartao = lg"), "{}", merged.source);
    assert!(merged.source.contains("radius=$raio-cartao"), "{}", merged.source);
    assert!(ribalta::check_prelude(&merged.source).is_empty());
}

#[test]
fn components_can_change_a_state_passed_to_them_and_build_go_targets() {
    let source = "@state q1 = pendente\n@component botao {n estado}\n[abrir]{click=\"$estado = feito; go #pergunta-$n\"}\n@end\n\n:: botao {n=1 estado=$q1}\n--- {#pergunta-1 hidden}\n@state aberto = false\n:: botao {n=1 estado=$aberto}";
    let (value, compilation) = output(source);
    assert_eq!(codes(&compilation), ["R602"]);
    let click = &children(&value, 0)[0]["content"][0]["click"];
    assert_eq!(click, &json!([{"type": "set", "name": "q1", "value": {"op": "lit", "value": "feito"}}, {"type": "go", "target": "pergunta-1"}]));
    let (_, compilation) = output("@let x = 1\n@component botao {estado}\n[a]{click=\"$estado = 2\"}\n@end\n\n:: botao {estado=$x}");
    assert_eq!(codes(&compilation), ["R605"]);
}

#[test]
fn on_enter_collects_style_and_separator_actions_in_order() {
    let value = clean(
        "@state visitas = 0\n@state visto = false\n@style final {on-enter=\"$visitas = $visitas + 1\"}\n\n# Início\n[ir](#fim)\n--- {#fim hidden .final on-enter=\"toggle $visto; $tentativas = 1\"}\n@state tentativas = 0\n# Fim",
    );
    let pages = value["document"]["pages"].as_array().unwrap();
    assert!(pages[0].get("onEnter").is_none());
    assert_eq!(
        pages[1]["onEnter"],
        json!([
            {"type": "set", "name": "visitas", "value": {"op": "binary", "operator": "+", "left": {"op": "state", "name": "visitas"}, "right": {"op": "lit", "value": 1.0}}},
            {"type": "toggle", "name": "visto"},
            {"type": "set", "name": "tentativas", "value": {"op": "lit", "value": 1.0}}
        ])
    );
}

#[test]
fn on_enter_only_changes_state_and_is_validated_like_click() {
    let (_, compilation) = output(
        "@state n = 0\n\n# A\n--- {on-enter=\"go #a\"}\n# B\n--- {on-enter=\"go back\"}\n# C\n--- {on-enter=\"$n = verdade; $nada = 1\"}\n# D\n--- {on-enter}\n# E {on-enter=\"$n = 1\"}",
    );
    assert_eq!(codes(&compilation), vec!["R612", "R612", "R602", "R601", "R203", "R209"]);
    let misplaced = compilation.diagnostics.iter().find(|d| d.code == "R209").unwrap();
    assert!(misplaced.hint.as_deref().unwrap().contains("separador"));
}

#[test]
fn rejected_on_enter_navigation_does_not_reach_hidden_pages() {
    let (value, compilation) = output("# A\n--- {on-enter=\"go #extra\"}\n# B\n--- {#extra hidden}\n# Extra");
    assert_eq!(codes(&compilation), vec!["R612", "R702"]);
    assert!(value["document"]["pages"][1].get("onEnter").is_none());
}

#[test]
fn nav_off_locks_pages_and_game_mode_locks_every_page() {
    let value = clean("# A\n--- {nav=off}\n# B\n--- {nav=on}\n# C");
    let pages = value["document"]["pages"].as_array().unwrap();
    assert!(pages[0].get("locked").is_none());
    assert_eq!(pages[1]["locked"], true);
    assert!(pages[2].get("locked").is_none());
    assert!(value["document"]["meta"].get("mode").is_none());

    let game = clean("@mode game\n@style livre {nav=on}\n\n# A\n--- {.livre}\n# B\n--- {nav=on}\n# C");
    let pages = game["document"]["pages"].as_array().unwrap();
    assert_eq!(game["document"]["meta"]["mode"], "game");
    assert_eq!(pages[0]["locked"], true);
    assert!(pages[1].get("locked").is_none());
    assert!(pages[2].get("locked").is_none());
}

#[test]
fn game_mode_comes_from_the_base_theme_and_validates_values() {
    let compilation = compile_with_prelude("@mode game\n", "# A");
    let value = serde_json::to_value(&compilation).unwrap();
    assert_eq!(value["document"]["meta"]["mode"], "game");
    assert_eq!(value["document"]["pages"][0]["locked"], true);
    assert!(ribalta::check_prelude("@mode game\n").is_empty());

    let (_, compilation) = output("@mode jogo\n\n# A\n--- {nav=talvez}\n# B");
    assert_eq!(codes(&compilation), vec!["R534", "R301"]);
}

#[test]
fn state_lists_hold_unique_text_and_change_through_list_actions() {
    let value = clean(
        "@state mochila = [mapa, mapa, chave]\n@state finais = []\n\n# Início\n[pegar]{click=\"add $mochila tocha; remove $mochila mapa\"}\n[esvaziar]{click=\"clear $mochila\"}\n--- {on-enter=\"add $finais vitoria\"}\n# Fim",
    );
    assert_eq!(value["document"]["state"], json!({"mochila": ["mapa", "chave"], "finais": []}));
    let block = &children(&value, 0)[1];
    let actions: Vec<Value> = block["content"].as_array().unwrap().iter().filter_map(|inline| inline.get("click").cloned()).collect();
    assert_eq!(
        actions,
        vec![
            json!([
                {"type": "add", "name": "mochila", "value": {"op": "lit", "value": "tocha"}},
                {"type": "remove", "name": "mochila", "value": {"op": "lit", "value": "mapa"}}
            ]),
            json!([{"type": "clear", "name": "mochila"}])
        ]
    );
    assert_eq!(value["document"]["pages"][1]["onEnter"], json!([{"type": "add", "name": "finais", "value": {"op": "lit", "value": "vitoria"}}]));
}

#[test]
fn membership_and_length_work_on_state_lists_at_runtime() {
    let value = clean(
        "@state mochila = [mapa]\n@state rota = mar\n\n# A\nPorta {if=\"mapa in $mochila\"}\nItens: $(len($mochila))\nRota {if=\"$rota in [mar, vila]\"}\n[x]{click=\"$rota = vila\"}",
    );
    let blocks = children(&value, 0);
    assert_eq!(blocks[1]["when"], json!({"op": "binary", "operator": "in", "left": {"op": "lit", "value": "mapa"}, "right": {"op": "state", "name": "mochila"}}));
    assert_eq!(blocks[2]["content"][1]["expr"], json!({"op": "call", "name": "len", "args": [{"op": "state", "name": "mochila"}]}));
    assert_eq!(
        blocks[2]["when"],
        json!({"op": "binary", "operator": "or",
            "left": {"op": "binary", "operator": "==", "left": {"op": "state", "name": "rota"}, "right": {"op": "lit", "value": "mar"}},
            "right": {"op": "binary", "operator": "==", "left": {"op": "state", "name": "rota"}, "right": {"op": "lit", "value": "vila"}}})
    );
}

#[test]
fn list_errors_explain_the_list_actions() {
    let (_, compilation) = output(
        "@state mochila = []\n@state n = 0\n@state errado = [1, 2]\n\n# A\n[a]{click=\"add $n chave\"}\n[b]{click=\"add $mochila 3\"}\n[c]{click=\"$mochila = vazio\"}\n[d]{click=\"toggle $mochila\"}\n[e]{click=\"add $mochila\"}\nIgual {if=\"$mochila == $mochila\"}\nNúmero {if=\"$n in $mochila\"}\nTexto {if=\"a in $n\"}\nDigitação {if=\"chav in $mochila\"}",
    );
    assert_eq!(codes(&compilation), vec!["R602", "R602", "R602", "R602", "R602", "R611", "R602", "R602", "R602", "R608"]);
    let typo = compilation.diagnostics.iter().find(|d| d.code == "R608").unwrap();
    assert!(typo.hint.as_deref().unwrap().contains("add $mochila chav"));
}

#[test]
fn components_read_and_change_document_state_directly() {
    let value = clean(
        "@state item = nenhum\n@state visto = false\n\n@component painel {titulo}\n::: box\n## $titulo\nItem: $item {if=$visto}\n[pegar]{click=\"$item = chave; toggle $visto\"}\n:::\n@end\n\n# Início\n:: painel {titulo=Teste}",
    );
    let panel = &children(&value, 0)[1];
    assert_eq!(panel["children"][1]["when"], json!({"op": "state", "name": "visto"}));
    let click = &panel["children"][2]["content"][0]["click"];
    assert_eq!(click[0], json!({"type": "set", "name": "item", "value": {"op": "lit", "value": "chave"}}));
    assert_eq!(click[1], json!({"type": "toggle", "name": "visto"}));
}

#[test]
fn components_explain_names_that_only_exist_where_they_are_used() {
    let (_, compilation) = output("@component mostra\nValor: $local\n@end\n\n# A\n---\n@state local = 1\n:: mostra");
    let diagnostic = &compilation.diagnostics[0];
    assert_eq!(diagnostic.code, "R410");
    assert!(diagnostic.message.contains("só existe onde o componente é usado"));
    assert!(diagnostic.hint.as_deref().unwrap().contains(":: mostra {local=$local}"));

    let (_, compilation) = output("@component c\nTexto $nada\n@end\n\n# A\n:: c");
    assert_eq!(compilation.diagnostics[0].hint.as_deref(), Some("Declare-o em @component c {nada}"));
}

#[test]
fn random_and_pick_draw_inside_actions_and_mark_the_document() {
    let value = clean(
        "@state dado = 0\n@state baralho = [ás, rei]\n@state carta = nenhuma\n\n# Mesa\n[rolar]{click=\"$dado = random(1, 6)\"}\n\n[jogar]{click=\"$carta = pick([pedra, papel]); add $baralho pick($baralho)\"}",
    );
    assert_eq!(value["document"]["meta"]["random"], true);
    let blocks = children(&value, 0);
    assert_eq!(
        blocks[1]["content"][0]["click"][0]["value"],
        json!({"op": "call", "name": "random", "args": [{"op": "lit", "value": 1.0}, {"op": "lit", "value": 6.0}]})
    );
    let play = &blocks[2]["content"][0]["click"];
    assert_eq!(play[0]["value"], json!({"op": "call", "name": "pick", "args": [{"op": "lit", "value": "pedra"}, {"op": "lit", "value": "papel"}]}));
    assert_eq!(play[1]["value"], json!({"op": "call", "name": "pick", "args": [{"op": "state", "name": "baralho"}]}));
    assert!(clean("@state n = 0\n\n# A\n[x]{click=\"$n = 1\"}")["document"]["meta"].get("random").is_none());
}

#[test]
fn drawing_outside_actions_is_explained() {
    let (_, compilation) = output("@state dado = 0\n@let fixo = random(1, 6)\n\n# A\nAgora: $(random(1, 6))\n\nSe {if=\"pick([a, b]) == a\"}");
    assert_eq!(codes(&compilation), vec!["R613", "R613", "R613"]);
    assert!(compilation.diagnostics[0].hint.as_deref().unwrap().contains("$dado = random(1, 6)"));
}

#[test]
fn hover_is_a_variant_condition() {
    let value = clean(
        "@state plano = pro\n@style realce {border=\"4 accent\" shadow=lg}\n\n# Planos\n::: box {.card .realce=hover click=\"$plano = basic\"}\nBásico\n:::\n\n[Pro]{.coral=hover .realce=\"$plano == pro\"}\n\n- Item {.realce=hover}",
    );
    let blocks = children(&value, 0);
    assert_eq!(blocks[1]["variants"][0]["when"], json!({"op": "hover"}));
    assert_eq!(blocks[1]["variants"][0]["style"]["shadow"], "lg");
    let span = &blocks[2]["content"][0];
    assert_eq!(span["variants"][0], json!({"when": {"op": "hover"}, "style": {"color": "#f2603d"}}));
    assert_eq!(span["variants"][1]["when"]["op"], "binary");
    assert_eq!(blocks[3]["items"][0]["variants"][0]["when"], json!({"op": "hover"}));
}

#[test]
fn pages_with_elapsed_or_timeout_run_a_clock() {
    let value = clean(
        "@state pontos = 0\n\n# Início\n--- {timeout=30 on-timeout=\"$pontos = 0; go #esgotado\"}\nRestam $(30 - $elapsed) s\n\nDica {if=\"$elapsed >= 10\"}\n--- {#esgotado hidden}\n# Tempo esgotado\n--- {timeout=5s}\n# Créditos",
    );
    let pages = value["document"]["pages"].as_array().unwrap();
    assert!(pages[0].get("clock").is_none());
    assert_eq!(pages[1]["timeout"], 30.0);
    assert_eq!(pages[1]["clock"], true);
    assert_eq!(pages[1]["onTimeout"], json!([{"type": "set", "name": "pontos", "value": {"op": "lit", "value": 0.0}}, {"type": "go", "target": "esgotado"}]));
    assert_eq!(pages[1]["children"][1]["when"]["left"], json!({"op": "elapsed"}));
    assert!(pages[2].get("clock").is_none());
    assert_eq!(pages[3]["timeout"], 5.0);
    assert!(pages[3].get("onTimeout").is_none());
}

#[test]
fn timer_errors_are_explained() {
    let (_, compilation) = output("@state elapsed = 0\n\n# A\n--- {on-timeout=\"go next\"}\n# B\n--- {timeout=0}\n# C\n--- {timeout=muito on-timeout=\"go next\"}\n# D");
    assert_eq!(codes(&compilation), vec!["R610", "R614", "R301", "R301"]);
}

#[test]
fn input_binds_text_and_number_state() {
    let value = clean(
        "@state nome = \"\"\n@state idade = 0\n\n# Quem é você?\n:: input {bind=$nome placeholder=\"Seu nome\" on-submit=\"go #fim\"}\n:: input {bind=$idade}\nSenha certa {if=\"$nome == abracadabra\"}\n--- {#fim}\n# Fim",
    );
    let blocks = children(&value, 0);
    assert_eq!(blocks[1]["type"], "input");
    assert_eq!(blocks[1]["name"], "nome");
    assert_eq!(blocks[1]["placeholder"], "Seu nome");
    assert_eq!(blocks[1]["onSubmit"], json!([{"type": "go", "target": "fim"}]));
    assert!(blocks[1].get("numeric").is_none());
    assert_eq!(blocks[2]["numeric"], true);
}

#[test]
fn input_errors_point_to_the_state() {
    let (_, compilation) = output("@state visto = false\n@let fixo = a\n\n# A\n:: input\n:: input {bind=nome}\n:: input {bind=$visto}\n:: input {bind=$fixo}\n:: input {bind=$nada}");
    assert_eq!(codes(&compilation), vec!["R116", "R116", "R602", "R605", "R601"]);
}

#[test]
fn key_shortcuts_trigger_click_actions() {
    let value = clean(
        "@state y = 0\n@component botao {rotulo}\n**$rotulo**\n@end\n\n# Mapa\n[Norte]{click=\"$y = $y - 1\" key=up}\n\n- Sul {click=\"$y = $y + 1\" key=ArrowDown}\n\n::: box {click=\"$y = 0\" key=R}\nVoltar ao centro\n:::\n\n:: botao {rotulo=Pular click=\"go next\" key=space}\n---\n# Fim",
    );
    let blocks = children(&value, 0);
    assert_eq!(blocks[1]["content"][0]["key"], "ArrowUp");
    assert_eq!(blocks[2]["items"][0]["key"], "ArrowDown");
    assert_eq!(blocks[3]["key"], "r");
    assert_eq!(blocks[4]["key"], "Space");
}

#[test]
fn key_errors_and_warnings() {
    let (_, compilation) = output("# A\n[x]{click=\"go next\" key=ctrl}\n\n[y]{key=a}\n---\n# B");
    assert_eq!(codes(&compilation), vec!["R301", "R210"]);
    assert_eq!(compilation.diagnostics[1].severity, Severity::Warning);
}

#[test]
fn sounds_are_declared_per_page_and_played_by_actions() {
    let value = clean(
        "# Abertura\n\n--- {on-enter=\"play chuva\"}\n:: sound https://exemplo.com/chuva.mp3 {name=chuva loop volume=40%}\n:: sound asset:3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c {autoplay}\n# A porta\n[Abrir]{click=\"stop chuva; go next\"}\n---\n# Dentro",
    );
    let page = &value["document"]["pages"][1];
    assert_eq!(page["onEnter"], json!([{"type": "play", "name": "chuva"}]));
    let blocks = page["children"].as_array().unwrap();
    assert_eq!(blocks[0], json!({"type": "sound", "name": "chuva", "src": {"kind": "url", "url": "https://exemplo.com/chuva.mp3"}, "loop": true, "volume": 0.4, "style": {}, "line": 4, "endLine": 4}));
    assert_eq!(blocks[1]["autoplay"], true);
    assert_eq!(blocks[1]["src"]["kind"], "asset");
    assert_eq!(blocks[3]["content"][0]["click"][0], json!({"type": "stop", "name": "chuva"}));
}

#[test]
fn sound_errors_are_explained() {
    let (_, compilation) = output("# A\n:: sound\n:: sound ftp://x/y.mp3 {name=a}\n:: sound https://x/y.mp3 {name=porta volume=2}\n:: sound https://x/z.mp3 {name=porta}\n[x]{click=\"play prta\"}\n---\n# B\n[y]{click=\"play porta\"}");
    assert_eq!(codes(&compilation), vec!["R117", "R301", "R301", "R616", "R615", "R615"]);
}

#[test]
fn persist_lists_document_state_to_keep() {
    let value = clean("@state finais = []\n@state moedas = 0\n@state rota = mar\n@persist finais, $moedas finais\n\n# A");
    assert_eq!(value["document"]["persist"], json!(["finais", "moedas"]));
    assert!(clean("@state n = 0\n\n# A")["document"].get("persist").is_none());
}

#[test]
fn persist_only_accepts_document_state() {
    let (_, compilation) = output("@state moedas = 0\n@let fixo = 1\n@persist moeda fixo\n\n# A");
    assert_eq!(codes(&compilation), vec!["R601", "R617"]);
    assert_eq!(compilation.diagnostics[0].hint.as_deref(), Some("Você quis dizer `$moedas`?"));
}

#[test]
fn component_parameters_cannot_hide_usage_attributes() {
    let (_, compilation) = output("@component botao {key texto click}\n**$texto**\n@end\n\n# Menu\n:: botao {texto=Ir}");
    assert_eq!(codes(&compilation), vec!["R419", "R419", "R409", "R409"]);
}

#[test]
fn scorm_reports_score_and_completion() {
    let value = clean("@state pontos = 0\n@scorm score=$pontos complete=#final,#fuga max=50 pass=35\n\n--- {#inicio}\n# A\n[Ir]{click=\"$pontos = $pontos + 10; go #final\"}\n\n--- {#final hidden}\n# Fim\n\n--- {#fuga hidden}\n# Fuga\n");
    assert_eq!(value["document"]["scorm"], json!({ "score": "pontos", "complete": ["final", "fuga"], "max": 50.0, "pass": 35.0 }));
    assert_eq!(clean("@scorm complete=#a\n\n--- {#a}\n# A")["document"]["scorm"], json!({ "complete": ["a"], "max": 100.0 }));
    assert!(clean("# A")["document"].get("scorm").is_none());
}

#[test]
fn scorm_validates_its_parameters() {
    let (_, compilation) = output("@state nome = \"Ana\"\n@state pontos = 0\n@scorm score=$nome complete=#fim\n\n--- {#final}\n# A");
    assert_eq!(codes(&compilation), vec!["R618", "R701"]);
    let (_, compilation) = output("@state pontos = 0\n@scorm score=$ponto nota=1 max=0 pass=200\n\n# A");
    assert_eq!(codes(&compilation), vec!["R601", "R618", "R618"]);
    assert_eq!(compilation.diagnostics[0].hint.as_deref(), Some("Você quis dizer `$pontos`?"));
    let (_, compilation) = output("@scorm max=10\n\n# A");
    assert_eq!(codes(&compilation), vec!["R618"]);
}

#[test]
fn externs_are_runtime_values_with_a_preview_default() {
    let value = clean("@extern pergunta = \"Qual é a capital?\"\n@extern rodada = 3\n@extern revelada = false\n@extern opcoes = [A, B]\n\n# $pergunta\n\nPergunta $rodada\n\nBrasília {if=\"$revelada\"}\n\nTem B {if=\"B in $opcoes\"}");
    let document = &value["document"];
    assert_eq!(document["externs"], json!(["pergunta", "rodada", "revelada", "opcoes"]));
    assert_eq!(document["state"], json!({"pergunta": "Qual é a capital?", "rodada": 3.0, "revelada": false, "opcoes": ["A", "B"]}));
    let blocks = children(&value, 0);
    assert_eq!(blocks[0]["content"][0], json!({"type": "expr", "expr": {"op": "state", "name": "pergunta"}}));
    assert_eq!(blocks[2]["when"], json!({"op": "state", "name": "revelada"}));
    assert!(clean("@state n = 0\n\n# A")["document"].get("externs").is_none());
}

#[test]
fn externs_do_not_warn_about_values_no_action_adds() {
    clean("@extern resultado = acerto\n@extern vistos = []\n\n# A\nErrou {if=\"$resultado == erro\"}\nViu {if=\"x in $vistos\"}");
}

#[test]
fn externs_cannot_be_changed_by_the_document() {
    let (_, compilation) = output(
        "@extern rodada = 1\n@extern revelada = false\n@extern nome = \"\"\n@extern vistos = []\n\n# A\n[x]{click=\"$rodada = 2; toggle $revelada; add $vistos a; clear $vistos\"}\n:: input {bind=$nome}",
    );
    assert_eq!(codes(&compilation), vec!["R619", "R619", "R619", "R619", "R619"]);
    assert!(compilation.diagnostics[0].message.contains("$rodada"));
}

#[test]
fn externs_are_not_persisted_or_scored() {
    let (_, compilation) = output("@extern pontos = 0\n@persist pontos\n@scorm score=$pontos\n\n# A");
    assert_eq!(codes(&compilation), vec!["R619", "R619"]);
}

#[test]
fn extern_names_share_the_namespace_of_other_variables() {
    let (_, compilation) = output("@state rodada = 1\n@extern rodada = 2\n\n# A");
    assert_eq!(codes(&compilation), vec!["R610"]);
}

#[test]
fn prelude_externs_reach_the_document() {
    let compilation = compile_with_prelude("@extern placar = 0", "# $placar");
    assert!(compilation.diagnostics.is_empty(), "{:#?}", compilation.diagnostics);
    let value = serde_json::to_value(&compilation).unwrap();
    assert_eq!(value["document"]["externs"], json!(["placar"]));
}

const PLACAR: &str = "@data exemplo\n| nome | pontos | fora |\n| --- | --- | --- |\n| Ana | 300 | false |\n| Bruno | 200 | true |\n@end\n@extern placar = $exemplo\n\n";

#[test]
fn extern_record_lists_repeat_at_runtime() {
    let value = clean(&format!("{PLACAR}# Placar\n\n::: for jogador, i in $placar\n$i. $jogador.nome {{.muted=$jogador.fora}}\n\n$jogador.pontos {{if=\"not $jogador.fora\"}}\n:::\n\nTotal: $(len($placar))"));
    let document = &value["document"];
    assert_eq!(document["externs"], json!(["placar"]));
    assert_eq!(document["state"]["placar"], json!([{"nome": "Ana", "pontos": 300.0, "fora": false}, {"nome": "Bruno", "pontos": 200.0, "fora": true}]));
    let each = &children(&value, 0)[1];
    assert_eq!(each["type"], "each");
    assert_eq!(each["list"], json!({"op": "state", "name": "placar"}));
    assert_eq!(each["item"], "jogador");
    assert_eq!(each["index"], "i");
    let first = &each["children"][0];
    assert_eq!(first["content"][0], json!({"type": "expr", "expr": {"op": "item", "name": "i"}}));
    assert_eq!(first["content"][2], json!({"type": "expr", "expr": {"op": "item", "name": "jogador", "field": "nome"}}));
    assert_eq!(first["variants"][0]["when"], json!({"op": "item", "name": "jogador", "field": "fora"}));
    assert_eq!(each["children"][1]["when"], json!({"op": "not", "arg": {"op": "item", "name": "jogador", "field": "fora"}}));
    let total = &children(&value, 0)[2];
    assert_eq!(total["content"][1]["expr"], json!({"op": "call", "name": "len", "args": [{"op": "state", "name": "placar"}]}));
}

#[test]
fn state_text_lists_repeat_at_runtime() {
    let value = clean("@state mochila = [mapa]\n\n# Mochila\n\n::: for coisa in $mochila\n$coisa\n:::\n[Pegar]{click=\"add $mochila chave\"}");
    let each = &children(&value, 0)[1];
    assert_eq!(each["type"], "each");
    assert!(each.get("index").is_none());
    assert_eq!(each["children"][0]["content"][0], json!({"type": "expr", "expr": {"op": "item", "name": "coisa"}}));
}

#[test]
fn runtime_repetition_reports_misuse() {
    let (_, compilation) = output(&format!("{PLACAR}# A\n\n::: for jogador in $placar\n$jogador.idade\n:::"));
    assert_eq!(codes(&compilation), vec!["R601"]);
    assert_eq!(compilation.diagnostics[0].hint.as_deref().map(|hint| hint.contains("nome")), Some(true));

    let (_, compilation) = output(&format!("{PLACAR}@state escolha = \"\"\n\n# A\n\n::: for jogador in $placar\n[Escolher]{{click=\"$escolha = $jogador.nome\"}}\n:::"));
    assert_eq!(codes(&compilation), vec!["R620"]);

    let (_, compilation) = output(&format!("{PLACAR}# A\n\n::: for jogador in $placar\n$jogador.nome {{size=$jogador.pontos}}\n:::"));
    assert_eq!(codes(&compilation), vec!["R604"]);

    let (_, compilation) = output(&format!("{PLACAR}# $placar"));
    assert_eq!(codes(&compilation), vec!["R602"]);
}

#[test]
fn record_lists_need_an_extern_with_an_example_row() {
    let (_, compilation) = output("@data exemplo\n| nome |\n| --- |\n| Ana |\n@end\n@state placar = $exemplo\n\n# A");
    assert_eq!(codes(&compilation), vec!["R602"]);
    let (_, compilation) = output("@data vazio\n| nome |\n| --- |\n@end\n@extern placar = $vazio\n\n# A");
    assert_eq!(codes(&compilation), vec!["R602"]);
}
