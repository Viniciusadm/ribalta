use std::process::ExitCode;

use clap::Parser;
use ribalta_cli::error::{CliError, CliResult};
use ribalta_cli::{Lang, LanguageCommand, Outcome};
use reqwest::Url;

#[derive(Debug, Parser)]
#[command(name = "ribalta", version, about = "Linha de comando da linguagem Ribalta: verifica, formata e mapeia arquivos .ribalta")]
struct Cli {
    #[arg(long, global = true, env = "RIBALTA_LANG", value_enum, default_value_t = Lang::Pt, help = "Idioma dos diagnósticos")]
    lang: Lang,

    #[arg(long, global = true, env = "RIBALTA_ASSET_BASE", value_name = "URL", help = "Endereço que, somado ao id, busca cada `asset:id` em `check --check-assets`")]
    asset_base: Option<String>,

    #[command(subcommand)]
    command: LanguageCommand,
}

fn asset_base(value: &str) -> CliResult<Url> {
    let mut url = Url::parse(value).map_err(|error| CliError::new(format!("endereço de assets inválido `{value}`: {error}")))?;
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

fn run(cli: Cli) -> CliResult<Outcome> {
    let base = cli.asset_base.as_deref().map(asset_base).transpose()?;
    ribalta_cli::run(cli.command, cli.lang, base.as_ref())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(outcome) => outcome.exit_code(),
        Err(error) => {
            eprintln!("erro: {error}");
            ExitCode::FAILURE
        }
    }
}
