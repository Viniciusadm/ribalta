fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (prelude_path, path) = match args.as_slice() {
        [flag, prelude, path] if flag == "--prelude" => (Some(prelude), path),
        [path] => (None, path),
        _ => panic!("usage: check [--prelude theme.ribalta] document.ribalta"),
    };
    let prelude = prelude_path.map(|path| std::fs::read_to_string(path).expect("read prelude")).unwrap_or_default();
    let source = std::fs::read_to_string(path).expect("read");
    if !prelude.is_empty() {
        for d in ribalta::check_prelude(&prelude) {
            println!("theme {:?} {} {}:{} {} {}", d.severity, d.code, d.line, d.column, d.message, d.hint.clone().unwrap_or_default());
        }
    }
    let compilation = ribalta::compile_with_prelude(&prelude, &source);
    for d in &compilation.diagnostics {
        println!("{:?} {} {}:{} {} {}", d.severity, d.code, d.line, d.column, d.message, d.hint.clone().unwrap_or_default());
    }
    println!("{} pages", compilation.document.pages.len());
}
