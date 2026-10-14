use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::Url;
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_TYPE, RETRY_AFTER};
use serde_json::Value;

use crate::args::Lang;
use crate::diagnostics::{Diagnostic, Severity};

const CONCURRENT_REQUESTS: usize = 4;
const ATTEMPTS: u32 = 4;
const MAX_WAIT_SECONDS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaKind {
    Image,
    Sound,
}

pub fn collect(document: &Value) -> BTreeMap<String, MediaKind> {
    let mut found = BTreeMap::new();
    walk(document, &mut found);
    found
}

fn walk(value: &Value, found: &mut BTreeMap<String, MediaKind>) {
    match value {
        Value::Array(items) => items.iter().for_each(|item| walk(item, found)),
        Value::Object(map) => {
            let kind = match map.get("type").and_then(Value::as_str) {
                Some("sound") => Some(MediaKind::Sound),
                Some("image") => Some(MediaKind::Image),
                _ => None,
            };
            if let (Some(kind), Some(reference)) = (kind, map.get("src").and_then(reference)) {
                found.entry(reference).or_insert(kind);
            }
            if let Some(reference) = map.get("bgImage").and_then(reference) {
                found.entry(reference).or_insert(MediaKind::Image);
            }
            map.values().for_each(|item| walk(item, found));
        }
        _ => {}
    }
}

fn reference(src: &Value) -> Option<String> {
    match src.get("kind")?.as_str()? {
        "asset" => Some(format!("asset:{}", src.get("id")?.as_str()?)),
        "url" => src.get("url")?.as_str().filter(|url| !url.starts_with("data:")).map(str::to_owned),
        _ => None,
    }
}

pub fn check(document: &Value, source: &str, asset_base: Option<&Url>, lang: Lang) -> Vec<Diagnostic> {
    let references: Vec<(String, MediaKind)> = collect(document).into_iter().collect();
    if references.is_empty() {
        return Vec::new();
    }
    let http = Client::builder()
        .user_agent(concat!("ribalta-cli/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(20))
        .build()
        .ok();
    let mut problems = Vec::new();
    for chunk in references.chunks(CONCURRENT_REQUESTS) {
        let results: Vec<Option<Problem>> = std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|(reference, kind)| {
                    let http = http.as_ref();
                    scope.spawn(move || http.and_then(|http| probe(http, asset_base, reference, *kind)))
                })
                .collect();
            handles.into_iter().map(|handle| handle.join().unwrap_or(None)).collect()
        });
        for ((reference, kind), problem) in chunk.iter().zip(results) {
            if let Some(problem) = problem {
                problems.push(diagnostic(source, reference, *kind, &problem, lang));
            }
        }
    }
    problems.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.column));
    problems
}

enum Problem {
    Unreachable(String),
    Status(u16),
    Throttled,
    Page,
}

fn probe(http: &Client, asset_base: Option<&Url>, reference: &str, kind: MediaKind) -> Option<Problem> {
    let url = match reference.strip_prefix("asset:") {
        Some(id) => asset_base?.join(id).ok()?.to_string(),
        None => reference.to_owned(),
    };
    let response = match request(http, &url) {
        Ok(response) => response,
        Err(error) => return Some(Problem::Unreachable(root_cause(&error))),
    };
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Some(Problem::Throttled);
    }
    if !response.status().is_success() {
        return Some(Problem::Status(response.status().as_u16()));
    }
    let content_type = response.headers().get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or_default().to_ascii_lowercase();
    let expected = matches!((kind, content_type.split('/').next()), (MediaKind::Image, Some("image")) | (MediaKind::Sound, Some("audio" | "video")));
    if content_type.starts_with("text/html") && !expected {
        return Some(Problem::Page);
    }
    None
}

fn request(http: &Client, url: &str) -> Result<Response, reqwest::Error> {
    let mut attempt = 0;
    loop {
        let response = match http.head(url).send() {
            Ok(response) if response.status().is_success() => response,
            _ => http.get(url).send()?,
        };
        attempt += 1;
        let throttled = matches!(response.status(), StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE);
        if !throttled || attempt >= ATTEMPTS {
            return Ok(response);
        }
        let asked = response.headers().get(RETRY_AFTER).and_then(|value| value.to_str().ok()).and_then(|value| value.trim().parse::<u64>().ok());
        let wait = asked.unwrap_or(1 << attempt).min(MAX_WAIT_SECONDS);
        std::thread::sleep(Duration::from_secs(wait));
    }
}

fn root_cause(error: &(dyn std::error::Error + 'static)) -> String {
    let mut current = error;
    while let Some(next) = current.source() {
        current = next;
    }
    current.to_string()
}

fn diagnostic(source: &str, reference: &str, kind: MediaKind, problem: &Problem, lang: Lang) -> Diagnostic {
    let (line, column, end_column) = locate(source, reference);
    let noun = match (kind, lang) {
        (MediaKind::Image, Lang::Pt) => "a imagem",
        (MediaKind::Image, Lang::En) => "the image",
        (MediaKind::Sound, Lang::Pt) => "o som",
        (MediaKind::Sound, Lang::En) => "the sound",
    };
    let (message, hint) = match (problem, lang) {
        (Problem::Unreachable(detail), Lang::Pt) => (format!("não foi possível acessar {noun} `{reference}`: {detail}"), "Confira o endereço e a conexão"),
        (Problem::Unreachable(detail), Lang::En) => (format!("could not reach {noun} `{reference}`: {detail}"), "Check the address and the connection"),
        (Problem::Status(status), Lang::Pt) => (format!("{noun} `{reference}` respondeu {status}"), "O arquivo pode ter sido removido ou exigir login. Use um endereço público ou guarde o arquivo como `asset:`"),
        (Problem::Status(status), Lang::En) => (format!("{noun} `{reference}` answered {status}"), "The file may have been removed or require a login. Use a public address or store the file as an `asset:`"),
        (Problem::Throttled, Lang::Pt) => (format!("não deu para conferir {noun} `{reference}`: o servidor limitou as consultas (429)"), "Rode de novo daqui a pouco. Se continuar, o site pode estar bloqueando o acesso direto ao arquivo: use uma versão menor dele ou guarde o arquivo como `asset:`"),
        (Problem::Throttled, Lang::En) => (format!("could not check {noun} `{reference}`: the server rate-limited the requests (429)"), "Run it again in a while. If it persists, the site may block direct access to the file: use a smaller version of it or store it as an `asset:`"),
        (Problem::Page, Lang::Pt) => (format!("`{reference}` é uma página, não {noun}"), "Abra a página, clique com o botão direito no arquivo e copie o endereço dele"),
        (Problem::Page, Lang::En) => (format!("`{reference}` is a web page, not {noun}"), "Open the page, right-click the file and copy its address"),
    };
    Diagnostic { severity: Severity::Warning, code: "R801".to_owned(), message, line, column, end_column, hint: Some(hint.to_owned()) }
}

fn locate(source: &str, needle: &str) -> (usize, usize, usize) {
    for (index, text) in source.lines().enumerate() {
        let complete = text.match_indices(needle).find(|(byte, _)| {
            text[byte + needle.len()..].chars().next().is_none_or(|next| matches!(next, ')' | ' ' | '}' | '"' | '\'' | '\t'))
        });
        if let Some((byte, _)) = complete {
            let column = text[..byte].encode_utf16().count() + 1;
            return (index + 1, column, column + needle.encode_utf16().count());
        }
    }
    (1, 1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_images_backgrounds_and_sounds_once() {
        let compilation = ribalta::compile_localized(
            "",
            "--- {image=https://x.test/fundo.jpg}\n# A ![i](https://x.test/a.png)\n\n![Foto](asset:0b6f1c7e-5d1e-4c55-9a0e-111111111111)\n\n:: sound https://x.test/s.mp3 {name=s}\n\n---\n![De novo](https://x.test/fundo.jpg)\n![Embutida](data:image/png;base64,AAAA)\n",
            ribalta::Locale::from_code("pt"),
        );
        let found = collect(&serde_json::to_value(&compilation.document).unwrap());
        assert_eq!(
            found.into_iter().collect::<Vec<_>>(),
            vec![
                ("asset:0b6f1c7e-5d1e-4c55-9a0e-111111111111".to_owned(), MediaKind::Image),
                ("https://x.test/a.png".to_owned(), MediaKind::Image),
                ("https://x.test/fundo.jpg".to_owned(), MediaKind::Image),
                ("https://x.test/s.mp3".to_owned(), MediaKind::Sound),
            ]
        );
    }

    #[test]
    fn points_at_the_first_use_in_utf16_columns() {
        assert_eq!(locate("# Olá\nÁ ![x](https://a.b/c.png)\n", "https://a.b/c.png"), (2, 8, 25));
        assert_eq!(locate("# A\n", "asset:x"), (1, 1, 1));
        assert_eq!(locate("![a](https://a.b/logo.svg)\n![b](https://a.b/)\n", "https://a.b/"), (2, 6, 18));
    }
}
