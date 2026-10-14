# @ribaltalang/codemirror

[CodeMirror 6](https://codemirror.net) support for [Ribalta](https://folioslides.com.br/docs/): syntax highlighting, completions (with Portuguese and English labels), snippets and a helper that turns compiler diagnostics into editor ranges.

```sh
npm install @ribaltalang/codemirror @codemirror/state @codemirror/view @codemirror/autocomplete @codemirror/lint
```

## Usage

```ts
import { autocompletion } from "@codemirror/autocomplete";
import { lintGutter, setDiagnostics } from "@codemirror/lint";
import { EditorView } from "@codemirror/view";
import { ribaltaCompletions } from "@ribaltalang/codemirror/completions";
import { toEditorDiagnostics } from "@ribaltalang/codemirror/diagnostics";
import { ribaltaHighlight, ribaltaLanguage } from "@ribaltalang/codemirror/language";
import { completionsEn } from "@ribaltalang/codemirror/messages";

const view = new EditorView({
  parent: document.querySelector("#editor")!,
  extensions: [
    ribaltaLanguage,
    ribaltaHighlight,
    lintGutter(),
    autocompletion({
      override: [ribaltaCompletions(() => ({ theme: null, styles: [], components: [], labels: completionsEn }))],
    }),
  ],
});

view.dispatch(setDiagnostics(view.state, toEditorDiagnostics(view.state.doc, diagnostics)));
```

- Pass `externs` in the completion context (`{ name, detail, declaration }`) to suggest the values your page provides after `@extern`; picking one writes the full declaration, including example tables.
- `variables` lists names for `$` completion; `theme` (from a compiled document) enables color and font suggestions.

## License

MIT or Apache-2.0, at your option.
