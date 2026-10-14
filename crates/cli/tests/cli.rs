use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};

use ribalta_testkit::http::{MockServer, run, stderr, stdout, workdir};

const WORKDIR: &str = "ribalta-cli-it";

fn ribalta(dir: &Path, args: &[&str]) -> Output {
    ribalta_with_stdin(dir, args, "")
}

fn ribalta_with_stdin(dir: &Path, args: &[&str], stdin: &str) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ribalta"));
    command.args(args).current_dir(dir).env_remove("RIBALTA_LANG").env_remove("RIBALTA_ASSET_BASE");
    run(command, stdin)
}

struct LanguageServer {
    child: std::process::Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl LanguageServer {
    fn start(dir: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ribalta"))
            .arg("lsp")
            .current_dir(dir)
            .env_remove("RIBALTA_LANG")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        Self { child, reader }
    }

    fn send(&mut self, message: serde_json::Value) {
        let body = message.to_string();
        let stdin = self.child.stdin.as_mut().unwrap();
        write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        stdin.flush().unwrap();
    }

    fn receive(&mut self) -> serde_json::Value {
        let mut length = 0;
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line).unwrap();
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length: ") {
                length = value.parse().unwrap();
            }
        }
        let mut body = vec![0; length];
        self.reader.read_exact(&mut body).unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn until(&mut self, accept: impl Fn(&serde_json::Value) -> bool) -> serde_json::Value {
        loop {
            let message = self.receive();
            if accept(&message) {
                return message;
            }
        }
    }

    fn request(&mut self, id: u64, method: &str, params: serde_json::Value) -> serde_json::Value {
        self.send(serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        self.until(|message| message["id"] == id)["result"].clone()
    }
}

#[test]
fn check_reports_errors_with_exit_code_two() {
    let dir = workdir(WORKDIR, "check");
    std::fs::write(dir.join("ok.ribalta"), "# Olá\nPrimeira página\n").unwrap();
    std::fs::write(dir.join("bad.ribalta"), "# Título {colr=coral}\n").unwrap();

    let ok = ribalta(&dir, &["check", "ok.ribalta"]);
    assert_eq!(ok.status.code(), Some(0), "{}", stderr(&ok));
    assert!(stdout(&ok).contains("ok.ribalta: 1 página · nenhum problema encontrado"));

    let bad = ribalta(&dir, &["check", "bad.ribalta"]);
    assert_eq!(bad.status.code(), Some(2));
    assert!(stdout(&bad).contains("erro R209 · bad.ribalta:1:"), "{}", stdout(&bad));
    assert!(stdout(&bad).contains("^^^^"));

    let json = ribalta(&dir, &["--lang", "en", "check", "bad.ribalta", "--json"]);
    let report: serde_json::Value = serde_json::from_str(&stdout(&json)).unwrap();
    assert_eq!(report["diagnostics"][0]["code"], "R209");
    assert_eq!(report["pages"], 1);

    let invalid = ribalta(&dir, &["check", "ok.ribalta", "--page", "16:9"]);
    assert_eq!(invalid.status.code(), Some(2), "{}", stderr(&invalid));
    let sized = ribalta(&dir, &["check", "ok.ribalta", "--page", "1920x1080"]);
    assert_eq!(sized.status.code(), Some(0), "{}", stderr(&sized));
}

#[test]
fn fmt_checks_writes_and_reads_stdin() {
    let dir = workdir(WORKDIR, "fmt");
    std::fs::create_dir_all(dir.join("aulas")).unwrap();
    std::fs::write(dir.join("aulas/a.ribalta"), "# A   \n-----\n  :::box\n  Texto\n  :::\n").unwrap();
    std::fs::write(dir.join("b.ribalta"), "# B\n").unwrap();

    let check = ribalta(&dir, &["fmt", "--check"]);
    assert_eq!(check.status.code(), Some(1));
    assert!(stdout(&check).contains("a.ribalta"));
    assert!(!stdout(&check).contains("b.ribalta"));

    let write = ribalta(&dir, &["fmt"]);
    assert_eq!(write.status.code(), Some(0), "{}", stderr(&write));
    assert_eq!(std::fs::read_to_string(dir.join("aulas/a.ribalta")).unwrap(), "# A\n---\n::: box\nTexto\n:::\n");
    assert_eq!(ribalta(&dir, &["fmt", "--check"]).status.code(), Some(0));

    let piped = ribalta_with_stdin(&dir, &["fmt", "-"], "\n\n# C  \n\n\n\nfim\n");
    assert_eq!(stdout(&piped), "# C\n\nfim\n");
}

#[test]
fn graph_lists_routes_endings_and_unreachable_pages() {
    let dir = workdir(WORKDIR, "graph");
    let source = "--- {#start}\n# Início\n[Fugir]{click=\"go #exit\"}\n\n--- {#exit hidden}\n# Saída\n\n--- {#lost hidden}\n# Perdido\n[Ir]{click=\"go #exit\"}\n";
    std::fs::write(dir.join("jogo.ribalta"), source).unwrap();
    let output = ribalta(&dir, &["graph", "jogo.ribalta", "--format", "json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let graph: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(graph["edges"][0], serde_json::json!({ "from": "start", "to": "exit", "kind": "click", "conditional": false }));
    assert_eq!(graph["pages"][1]["ending"], true);
    assert_eq!(graph["pages"][2]["reachable"], false);

    let text = stdout(&ribalta(&dir, &["graph", "jogo.ribalta"]));
    assert!(text.contains("finais: #exit · inalcançáveis: #lost"), "{text}");

    std::fs::write(dir.join("jogo.ribalta"), "# Início {click=\"go #nada\"}\n").unwrap();
    assert_eq!(ribalta(&dir, &["graph", "jogo.ribalta"]).status.code(), Some(2));
}

#[test]
fn lsp_reports_completes_navigates_and_formats() {
    let dir = workdir(WORKDIR, "lsp").canonicalize().unwrap();
    std::fs::write(dir.join("theme.ribalta"), "@style destaque {color=accent}\n").unwrap();
    let file = dir.join("aula.ribalta");
    let uri = format!("file://{}", file.display());
    let theme_uri = format!("file://{}", dir.join("theme.ribalta").display());
    let mut server = LanguageServer::start(&dir);
    server.request(1, "initialize", serde_json::json!({ "capabilities": {}, "processId": null, "rootUri": null }));
    server.send(serde_json::json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));

    let text = "--- {#inicio}\n# Olá {.destaque}\n[Ir]{click=\"go #fim\"}\n\n--- {#final}\n# Fim\n";
    server.send(serde_json::json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": uri, "languageId": "ribalta", "version": 1, "text": text } } }));
    let published = server.until(|message| message["method"] == "textDocument/publishDiagnostics");
    let diagnostics = published["params"]["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0]["code"], "R701");
    assert_eq!(diagnostics[0]["range"]["start"], serde_json::json!({ "line": 2, "character": 15 }));
    assert!(diagnostics[0]["message"].as_str().unwrap().contains("dica:"));

    let completion = server.request(2, "textDocument/completion", serde_json::json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 17 } }));
    let labels: Vec<&str> = completion.as_array().unwrap().iter().map(|item| item["label"].as_str().unwrap()).collect();
    assert_eq!(labels, vec!["inicio", "final"]);

    let definition = server.request(3, "textDocument/definition", serde_json::json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": 10 } }));
    assert_eq!(definition["uri"], theme_uri);
    assert_eq!(definition["range"]["start"]["line"], 0);

    let messy = "--- {#inicio}\n# Olá   \n\n\n\nTexto\n";
    server.send(serde_json::json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": { "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "text": messy }] } }));
    let edits = server.request(4, "textDocument/formatting", serde_json::json!({ "textDocument": { "uri": uri }, "options": { "tabSize": 2, "insertSpaces": true } }));
    assert_eq!(edits[0]["newText"], "--- {#inicio}\n# Olá\n\nTexto\n");

    server.request(5, "shutdown", serde_json::Value::Null);
    server.send(serde_json::json!({ "jsonrpc": "2.0", "method": "exit" }));
    assert!(server.child.wait().unwrap().success());
}

#[test]
fn check_assets_resolves_asset_ids_through_the_asset_base() {
    let dir = workdir(WORKDIR, "check-assets");
    let server = MockServer::start(|request| match request.path.as_str() {
        "/ok.png" | "/uploads/0b6f1c7e-5d1e-4c55-9a0e-111111111111" => (200, "IMG".to_owned()),
        _ => (404, String::new()),
    });
    let origin = server.base_url.trim_end_matches("/api/public/v1").to_owned();
    let base = format!("{origin}/uploads");
    let source = format!("# Fotos\n\n![Boa]({origin}/ok.png)\n\n![Sumiu]({origin}/sumiu.png)\n\n![Enviada](asset:0b6f1c7e-5d1e-4c55-9a0e-111111111111)\n");
    std::fs::write(dir.join("fotos.ribalta"), source).unwrap();

    let offline = ribalta(&dir, &["--asset-base", &base, "check", "fotos.ribalta"]);
    assert!(offline.status.success());
    assert!(server.requests().is_empty());

    let output = ribalta(&dir, &["--asset-base", &base, "check", "fotos.ribalta", "--check-assets", "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0]["code"], "R801");
    assert_eq!(diagnostics[0]["line"], 5);
    assert!(diagnostics[0]["message"].as_str().unwrap().contains("respondeu 404"));

    let strict = ribalta(&dir, &["--asset-base", &base, "check", "fotos.ribalta", "--check-assets", "--strict"]);
    assert_eq!(strict.status.code(), Some(2));
}

#[test]
fn check_assets_skips_asset_ids_without_an_asset_base() {
    let dir = workdir(WORKDIR, "check-assets-no-base");
    std::fs::write(dir.join("fotos.ribalta"), "# Fotos\n\n![Enviada](asset:0b6f1c7e-5d1e-4c55-9a0e-111111111111)\n").unwrap();
    let output = ribalta(&dir, &["check", "fotos.ribalta", "--check-assets", "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report["diagnostics"], serde_json::json!([]));
}
