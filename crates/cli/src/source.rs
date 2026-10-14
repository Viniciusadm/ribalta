use std::io::Read;
use std::path::Path;

use crate::error::{CliError, CliResult};

pub fn is_stdin(path: &Path) -> bool {
    path.as_os_str() == "-"
}

pub fn read_source(path: &Path) -> CliResult<String> {
    if is_stdin(path) {
        return read_stdin();
    }
    std::fs::read_to_string(path).map_err(|error| CliError::io("ler", path, error))
}

pub fn read_stdin() -> CliResult<String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|error| CliError::new(format!("não foi possível ler a entrada padrão: {error}")))?;
    Ok(text)
}

pub fn display_name(path: &Path) -> String {
    if is_stdin(path) { "<entrada padrão>".to_owned() } else { path.display().to_string() }
}
