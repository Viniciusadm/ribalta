use std::fs;
use std::path::{Path, PathBuf};

pub struct Sample {
    pub name: String,
    pub prelude: String,
    pub source: String,
}

fn collect(dir: &Path, extensions: &[&str], found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "node_modules" || name == ".generated") {
                continue;
            }
            collect(&path, extensions, found);
        } else if path.extension().is_some_and(|ext| extensions.iter().any(|wanted| ext == *wanted)) {
            found.push(path);
        }
    }
}

pub fn ribalta_fences(markdown: &str) -> Vec<String> {
    let mut fences = Vec::new();
    let mut current: Option<(usize, Vec<&str>)> = None;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        let ticks = trimmed.chars().take_while(|char| *char == '`').count();
        match &mut current {
            Some((open, lines)) => {
                if ticks >= *open && trimmed[ticks..].trim().is_empty() {
                    fences.push(lines.join("\n"));
                    current = None;
                } else {
                    lines.push(line);
                }
            }
            None if ticks >= 3 && trimmed[ticks..].trim_start().starts_with("ribalta") => current = Some((ticks, Vec::new())),
            None => {}
        }
    }
    fences
}

pub fn samples(ribalta_dirs: &[PathBuf], markdown_dirs: &[PathBuf]) -> Vec<Sample> {
    let mut files = Vec::new();
    for dir in ribalta_dirs {
        collect(dir, &["ribalta"], &mut files);
    }
    let mut samples: Vec<Sample> = files
        .iter()
        .map(|path| {
            let theme = path.with_file_name("theme.ribalta");
            let prelude = if path.file_name().is_some_and(|name| name != "theme.ribalta") && theme.exists() {
                fs::read_to_string(theme).unwrap()
            } else {
                String::new()
            };
            Sample { name: path.display().to_string(), prelude, source: fs::read_to_string(path).unwrap() }
        })
        .collect();
    let mut docs = Vec::new();
    for dir in markdown_dirs {
        collect(dir, &["md", "mdx"], &mut docs);
    }
    for path in docs {
        for (index, source) in ribalta_fences(&fs::read_to_string(&path).unwrap()).into_iter().enumerate() {
            samples.push(Sample { name: format!("{}#{index}", path.display()), prelude: String::new(), source });
        }
    }
    samples
}

pub fn indent_blocks(source: &str) -> String {
    let mut depth: usize = 0;
    let mut in_fence = false;
    let mut output = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            output.push(format!("{}{line}", "  ".repeat(depth)));
            continue;
        }
        if in_fence {
            output.push(format!("{}{line}", "  ".repeat(depth)));
            continue;
        }
        if trimmed == ":::" {
            depth = depth.saturating_sub(1);
            output.push(format!("{}:::", "  ".repeat(depth)));
            continue;
        }
        if trimmed.is_empty() {
            output.push(String::new());
            continue;
        }
        output.push(format!("{}{line}", "  ".repeat(depth)));
        if trimmed.starts_with(":::") {
            depth += 1;
        }
    }
    output.join("\r\n") + "   \r\n\r\n\r\n"
}

pub fn assert_formatting_keeps_results(samples: &[Sample], minimum: usize, format: impl Fn(&str) -> String, same: impl Fn(&str, &str, &str) -> bool) {
    assert!(samples.len() > minimum, "corpus too small: {}", samples.len());
    let mut changed = 0;
    for sample in samples {
        let formatted = format(&sample.source);
        assert!(same(&sample.prelude, &sample.source, &formatted), "formatting changed the result of {}", sample.name);
        assert_eq!(format(&formatted), formatted, "formatting is not idempotent for {}", sample.name);
        if formatted != sample.source {
            changed += 1;
        }
    }
    eprintln!("{} samples, {changed} reformatted", samples.len());
}

pub fn assert_indentation_is_normalized(samples: &[Sample], minimum: usize, format: impl Fn(&str) -> String, same: impl Fn(&str, &str, &str) -> bool) {
    let mut checked = 0;
    for sample in samples {
        let indented = indent_blocks(&sample.source);
        if !same(&sample.prelude, &sample.source, &indented) {
            continue;
        }
        let formatted = format(&indented);
        assert!(same(&sample.prelude, &indented, &formatted), "formatting changed the result of indented {}", sample.name);
        assert_eq!(formatted, format(&sample.source), "indented {} did not return to the flat style", sample.name);
        checked += 1;
    }
    assert!(checked > minimum, "too few indented samples were equivalent: {checked}");
}
