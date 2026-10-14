pub mod args;
pub mod commands;
pub mod diagnostics;
pub mod error;
pub mod output;
pub mod source;

use std::process::ExitCode;

use reqwest::Url;

pub use args::{Lang, LanguageCommand};

use crate::error::CliResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    CompileErrors,
}

impl Outcome {
    pub fn exit_code(self) -> ExitCode {
        match self {
            Self::Success => ExitCode::SUCCESS,
            Self::CompileErrors => ExitCode::from(2),
        }
    }
}

pub fn run(command: LanguageCommand, lang: Lang, asset_base: Option<&Url>) -> CliResult<Outcome> {
    match command {
        LanguageCommand::Check(args) => commands::check::run(&args, lang, asset_base),
        LanguageCommand::Fmt(args) => commands::fmt::run(&args),
        LanguageCommand::Graph(args) => commands::graph::run(&args, lang),
        LanguageCommand::Lsp => commands::lsp::run(lang),
    }
}
