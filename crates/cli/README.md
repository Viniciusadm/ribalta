# ribalta-cli

Command line for the [Ribalta](https://viniciusadm.github.io/ribalta/) language: check, format and map `.ribalta` files, and a language server for editors.

```sh
cargo install ribalta-cli
```

This installs the `ribalta` binary:

```sh
ribalta check document.ribalta
ribalta check --prelude theme.ribalta document.ribalta
ribalta fmt document.ribalta
ribalta graph document.ribalta
ribalta lsp
```

- `--lang en` (or `RIBALTA_LANG=en`) shows diagnostics in English; Portuguese is the default.
- `check --check-assets` with `--asset-base <url>` (or `RIBALTA_ASSET_BASE`) verifies that every `asset:id` exists.
- `lsp` speaks the Language Server Protocol over standard input and output, for any editor.

The crate is also a library (`ribalta_cli`) for tools that embed these commands: `ribalta_cli::run(command, lang, asset_base)`.

## License

MIT or Apache-2.0, at your option.
