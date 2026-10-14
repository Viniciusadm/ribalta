# Ribalta for VS Code

Language support for [Ribalta](https://viniciusadm.github.io/ribalta/), a text language for themed visuals.

- Syntax highlighting for `.ribalta` files
- Diagnostics from the Ribalta compiler, with the `R###` codes and hints
- Completion for directives, blocks, components, attributes and their values, styles, colors, `$variables`, page ids after `#` and icons
- Go to definition for `#page-id`, `$variable`, `.style` and components, including the ones declared in the base theme
- Outline with pages, styles, components and variables
- Formatting (`ribalta fmt`), which refuses any change that would alter the compiled document
- Command **Ribalta: Show route map**

## Requirements

The extension runs `<path> lsp` and `<path> graph`, from the language's command line:

```sh
cargo install ribalta-cli
```

## Settings

| Setting | Default | Description |
| --- | --- | --- |
| `ribalta.path` | `ribalta` | Path to the `ribalta` binary |
| `ribalta.theme` | empty | Base theme applied before each document, relative to each file's folder. When empty, a `theme.ribalta` next to the file is used |
| `ribalta.language` | `auto` | Language of the compiler messages: `auto`, `pt` or `en` |

## Development

```sh
npm install
npm run build      # bundles src/extension.ts into dist/extension.js
npm run typecheck
npm run package    # produces ribalta-<version>.vsix
```

Press F5 with this folder open in VS Code to start an Extension Development Host.
