use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value.as_str())
    }
}

type Handler = dyn Fn(&Request) -> (u16, String) + Send + Sync;

pub struct MockServer {
    pub base_url: String,
    requests: Arc<Mutex<Vec<Request>>>,
}

impl MockServer {
    pub fn start(handler: impl Fn(&Request) -> (u16, String) + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://127.0.0.1:{}/api/public/v1", listener.local_addr().unwrap().port());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        let handler: Arc<Handler> = Arc::new(handler);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
                    continue;
                }
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or_default().to_owned();
                let path = parts.next().unwrap_or_default().to_owned();
                let mut headers = Vec::new();
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let line = line.trim_end();
                    if line.is_empty() {
                        break;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        headers.push((key.trim().to_owned(), value.trim().to_owned()));
                    }
                }
                let length = headers
                    .iter()
                    .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                    .and_then(|(_, value)| value.parse().ok())
                    .unwrap_or(0);
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                let request = Request { method, path, headers, body: String::from_utf8(body).unwrap() };
                let (status, response) = handler(&request);
                recorded.lock().unwrap().push(request);
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                    response.len()
                );
                stream.write_all(reply.as_bytes()).unwrap();
            }
        });
        Self { base_url, requests }
    }

    pub fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }
}

pub fn workdir(prefix: &str, name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("{prefix}-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn run(mut command: Command, stdin: &str) -> Output {
    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
