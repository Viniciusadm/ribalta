---
title: Ribalta
description: A Markdown-inspired text language for themed visuals.
template: splash
hero:
  tagline: Write the text. The theme takes care of the visuals.
  actions:
    - text: Writing documents
      link: guides/writing/
      icon: right-arrow
    - text: Syntax reference
      link: reference/syntax/
      variant: minimal
---

## A document in Ribalta

```ribalta preview
--- {align=center}
Quarterly report {.kicker}
# We grew [38%]{color=yellow} this quarter
Recurring revenue above target for the third month in a row. {.lead}
```

Each `---` line starts a page. Headings, lists and emphasis are written as in Markdown; `:::` blocks build columns, cards and charts; attributes in `{…}` adjust whatever you need. Colors, fonts and spacing come from the theme, by name.

## What you will find here

- **[Guides](/en/guides/writing/)**: text, page sizes, layouts, styling, themes, header and footer, components, data, interactivity, step-by-step reveal, charts and images.
- **[Reference](/en/reference/syntax/)**: the whole syntax, attributes, blocks, themes, icons and error messages.

## Using the language in a program

| Package | What for |
| --- | --- |
| [`@ribaltalang/compiler`](https://www.npmjs.com/package/@ribaltalang/compiler) | Compiles the text in the browser or in Node.js (WebAssembly) and returns the document as JSON with its diagnostics. |
| [`@ribaltalang/renderer`](https://www.npmjs.com/package/@ribaltalang/renderer) | Draws the document with React. |
| [`@ribaltalang/player`](https://www.npmjs.com/package/@ribaltalang/player) | Navigation, steps, state, timers, keyboard and sounds of an interactive document. |
| [`@ribaltalang/codemirror`](https://www.npmjs.com/package/@ribaltalang/codemirror) | Highlighting, completion and errors in CodeMirror 6. |
| [`ribalta`](https://github.com/Viniciusadm/ribalta/tree/master/crates/ribalta) | The compiler as a Rust library. |
| [`ribalta-cli`](https://github.com/Viniciusadm/ribalta/tree/master/crates/cli) | The `ribalta` command: checks, formats, shows the route map and serves as a language server for editors. |
| [VS Code extension](https://github.com/Viniciusadm/ribalta/tree/master/editors/vscode) | Highlighting, errors as you type, completion, go to definition and formatting. |

The format of the compiled document is described in [IR.md](https://github.com/Viniciusadm/ribalta/blob/master/crates/ribalta/IR.md).
