use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct CliError(String);

pub type CliResult<T> = Result<T, CliError>;

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn io(action: &str, path: &Path, error: std::io::Error) -> Self {
        Self(format!("não foi possível {action} `{}`: {error}", path.display()))
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}
