use std::path::{Path, PathBuf};

use crate::Outcome;
use crate::args::FmtArgs;
use crate::source::{display_name, is_stdin, read_source};
use crate::error::{CliError, CliResult};

pub fn run(args: &FmtArgs) -> CliResult<Outcome> {
    let prelude = args.theme.as_deref().map(read_source).transpose()?.unwrap_or_default();
    if args.paths.iter().any(|path| is_stdin(path)) {
        if args.paths.len() > 1 {
            return Err(CliError::new("`-` não pode ser combinado com outros arquivos"));
        }
        let source = read_source(Path::new("-"))?;
        let formatted = formatted(&prelude, &source, "<entrada padrão>")?;
        print!("{formatted}");
        return Ok(Outcome::Success);
    }

    let roots = if args.paths.is_empty() { vec![PathBuf::from(".")] } else { args.paths.clone() };
    let mut files = Vec::new();
    for root in &roots {
        collect(root, &mut files)?;
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        return Err(CliError::new("nenhum arquivo .ribalta encontrado"));
    }

    let mut pending = 0;
    let mut failures = Vec::new();
    for file in &files {
        let source = read_source(file)?;
        let name = display_name(file);
        let result = match formatted(&prelude, &source, &name) {
            Ok(result) => result,
            Err(error) => {
                eprintln!("erro: {error}");
                failures.push(name);
                continue;
            }
        };
        if result == source {
            continue;
        }
        pending += 1;
        if args.check {
            println!("{name}");
        } else {
            std::fs::write(file, &result).map_err(|error| CliError::io("gravar", file, error))?;
            println!("formatado: {name}");
        }
    }

    if !failures.is_empty() {
        return Err(CliError::new(format!("{} arquivo(s) não foram formatados", failures.len())));
    }
    if args.check && pending > 0 {
        return Err(CliError::new(format!("{pending} arquivo(s) precisam ser formatados com `fmt`")));
    }
    if pending == 0 {
        eprintln!("{} arquivo(s) já estavam formatados.", files.len());
    }
    Ok(Outcome::Success)
}

fn formatted(prelude: &str, source: &str, name: &str) -> CliResult<String> {
    let result = ribalta::format_source(source);
    if result != source && !ribalta::same_result(prelude, source, &result) {
        return Err(CliError::new(format!(
            "`{name}` não foi alterado: formatar mudaria o resultado compilado. Isso é um bug do formatador; por favor, reporte com o arquivo"
        )));
    }
    Ok(result)
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> CliResult<()> {
    let metadata = std::fs::metadata(path).map_err(|error| CliError::io("ler", path, error))?;
    if metadata.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }
    let entries = std::fs::read_dir(path).map_err(|error| CliError::io("ler", path, error))?;
    for entry in entries.flatten() {
        let child = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if child.is_dir() {
            if !name.starts_with('.') && name != "node_modules" && name != "target" {
                collect(&child, files)?;
            }
        } else if name.ends_with(".ribalta") {
            files.push(child);
        }
    }
    Ok(())
}
