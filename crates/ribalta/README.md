# ribalta

The compiler of [Ribalta](https://folioslides.com.br/docs/), a text language for themed visuals. It turns Ribalta source into an intermediate representation (the IR, a JSON-serializable document) plus diagnostics in Portuguese or English.

```toml
[dependencies]
ribalta = "0.1"
```

## Usage

```rust
use ribalta::{compile_localized, Locale, Severity};

let compilation = compile_localized("", "@extern title = \"Hello\"\n\n# $title", Locale::En);
for diagnostic in &compilation.diagnostics {
    println!("{}:{} {} {}", diagnostic.line, diagnostic.column, diagnostic.code, diagnostic.message);
}
let errors = compilation.diagnostics.iter().any(|d| d.severity == Severity::Error);
let json = serde_json::to_string(&compilation.document)?;
```

| Function | Purpose |
| --- | --- |
| `compile(source)` | Compiles a document |
| `compile_with_prelude(prelude, source)` | Compiles with a theme (`prelude`) applied first |
| `compile_localized(prelude, source, locale)` | Same, with messages in `Locale::Pt` or `Locale::En` |
| `compile_with_options(prelude, source, locale, options)` | Same, with host defaults such as the page size (`CompileOptions { page: PageSize { width, height } }`) |
| `check_prelude(prelude)` | Validates a theme |
| `format_source(source)` | Formats source without changing the compiled result |
| `*_to_json` | Variants that return JSON, used by the WebAssembly build |

An error does not stop compilation: the result always carries everything the compiler could build. `document.externs` lists the values declared with `@extern`, which the host that shows the document provides at run time.

Page size: every page has a `width` and a `height` (`None` means the height follows the content). The host sets the default through `CompileOptions` (without it, pages are 1920 wide with automatic height); the source can change it with `@page 1920x1080` or `@page width=800 height=auto`, and a single page with `--- {width=1080 height=1920}`. A presentation app, for example, passes 1920 by 1080 to get slides.

The IR is specified in [IR.md](IR.md). Draw it in the browser with [`@ribaltalang/renderer`](https://www.npmjs.com/package/@ribaltalang/renderer); compile in the browser with [`@ribaltalang/compiler`](https://www.npmjs.com/package/@ribaltalang/compiler), the same crate built as WebAssembly (`cargo build --release --target wasm32-unknown-unknown -p ribalta`).

## License

MIT or Apache-2.0, at your option.
