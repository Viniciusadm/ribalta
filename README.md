# Ribalta

Ribalta is a text language for themed visuals. This repository holds the compiler, its bindings, the renderers and the editor support. Everything here depends only on the IR contract in [crates/ribalta/IR.md](crates/ribalta/IR.md).

| Part | What it is |
| --- | --- |
| [`ribalta`](crates/ribalta) | The compiler. Turns text into an intermediate representation (IR) with diagnostics in pt/en. Builds as a Rust library and as WebAssembly for the browser. |
| [`ribalta-cli`](crates/cli) | Library and `ribalta` binary with the commands that only need the language: `check`, `fmt`, `graph` and `lsp`. `--asset-base` (or `RIBALTA_ASSET_BASE`) tells `check --check-assets` where `asset:id` files live. The app's CLI embeds these commands. |
| [`ribalta-testkit`](crates/testkit) | Test helpers shared by the language and the app: the formatter corpus checks and a mock HTTP server for CLI tests. Only a dev-dependency. |
| [`ribalta-jni`](crates/ribalta-jni) | JNI bindings of the compiler, used by the Android reader. |
| [`@ribaltalang/compiler`](packages/compiler) | npm package that loads `ribalta.wasm` and exposes the compiler and the IR types to TypeScript. `npm run wasm -w @ribaltalang/compiler` builds the module. |
| [`@ribaltalang/renderer`](packages/renderer) | Draws the IR with React. Draws one page (`PageView`), a page scaled to its box (`ScaledPage`), a whole document as a scrolling column (`DocumentView`), a static SVG (`StaticPage`) or every page for printing (`PrintPages`). The page that shows a document provides a `RendererHostContext` that turns asset ids and font families into addresses. |
| [`@ribaltalang/codemirror`](packages/codemirror) | CodeMirror 6 support: highlighting, autocompletion with its pt/en labels, snippets and the mapping of diagnostics to editor ranges. |
| [`@ribaltalang/player`](packages/player) | Headless runtime that plays a document: navigation and steps, `@state` actions, clock and timeouts, keyboard and swipe, sounds, `@persist` and SCORM. The screens around it belong to each host. |
| [`android`](android) | Native Android reader: a Kotlin renderer of the IR. |
| [`editors/vscode`](editors/vscode) | VS Code extension (syntax highlighting + `ribalta lsp`). |

## Requirements

- Rust **1.94+** (edition 2024)
- WebAssembly target for the web app: `rustup target add wasm32-unknown-unknown`

## Compiler (`ribalta`)

Main public API:

| Function | Purpose |
| --- | --- |
| `compile(source)` | Compiles a document |
| `compile_with_prelude(prelude, source)` | Compiles with the project theme (`prelude`) applied |
| `compile_localized(prelude, source, locale)` | Same, with messages in pt or en |
| `compile_with_options(prelude, source, locale, options)` | Same, with the host's defaults, such as the page size |
| `check_prelude(prelude)` | Validates a project theme |
| `*_to_json` | Variants that return JSON, used by the WebAssembly bridge |

The result contains the `document` (its pages as IR) and the list of `diagnostics` (severity, code, line, column, message and hint). An error does not stop compilation: the compiler returns everything it could build. The IR is specified in [crates/ribalta/IR.md](crates/ribalta/IR.md), the contract for writing a renderer outside the web app.

### Page size

Every page has a width and a height, and a `null` height means the page grows with its content. The host sets the default when it compiles (`CompileOptions { page: PageSize { width, height } }`; without it, pages are 1920 wide with automatic height), and the source can change it:

```ribalta
@page width=800 height=auto

# A whole document
---{width=1080 height=1920}
# A vertical page
```

Check a file from the command line:

```bash
cargo run -p ribalta --example check -- document.ribalta
cargo run -p ribalta --example check -- --prelude theme.ribalta document.ribalta
```

Build the WebAssembly module (`npm run wasm -w @ribaltalang/compiler` does this and copies the file into the package):

```bash
cargo build --release --target wasm32-unknown-unknown -p ribalta
```

The release profile optimizes the `ribalta` crate for size (`opt-level = "s"`, LTO).

## Documentation

[`docs`](docs) holds the pages about the language, in pt and en (`en/`): writing, page sizes and documents, layout, styling, themes, components, data, interactivity, charts, images and the reference of syntax, attributes, blocks, themes, icons and errors. They are also published as [`@ribaltalang/docs`](packages/docs) for sites that host them, and their `ribalta` examples are the corpus of the formatter and Android tests.

## Tests

```bash
cargo test
npm install && npm test
```

`cargo test` covers the compiler (`crates/ribalta/tests`) and the language CLI (`crates/cli/tests`), including the conformance cases in `crates/ribalta/conformance` that the web and Android renderers also run, and the formatter over every `ribalta` example in [`docs`](docs). `npm test` builds `ribalta.wasm` and runs [`tests`](tests) against the npm packages: the same conformance cases, the expression fixtures, the icon data and the documentation (every page in both languages, every error code documented).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
