import type { Completion, CompletionContext, CompletionResult } from "@codemirror/autocomplete";
import type { EditorView } from "@codemirror/view";
import type { Theme } from "@ribaltalang/compiler";
import { icons } from "@ribaltalang/renderer/icons";
import type { CompletionMessages } from "./messages";
import { snippetsFor } from "./snippets";

const attributeNames = ["color", "bg", "image", "font", "size", "weight", "italic", "uppercase", "align", "valign", "pad", "gap", "radius", "border", "shadow", "w", "h", "x", "y", "opacity", "leading", "tracking", "step", "fit", "widths", "cols", "type", "header", "footer", "overlay", "cover", "enter", "if", "click", "data", "label", "series", "columns", "hidden", "next", "steps", "transition", "on-enter", "nav", "width", "height"];

const directives: Completion[] = ["title", "author", "theme", "version", "page", "color", "font", "style", "component", "let", "data", "state", "extern", "transition", "mode", "persist", "scorm", "header", "footer", "overlay", "end"].map(
  (label) => ({ label, type: "keyword" }),
);

const enumValues: Record<string, string[]> = {
  align: ["left", "center", "right"],
  valign: ["top", "center", "bottom"],
  size: ["xs", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl", "6xl", "7xl"],
  pad: ["none", "xs", "sm", "md", "lg", "xl", "2xl"],
  gap: ["none", "xs", "sm", "md", "lg", "xl", "2xl"],
  radius: ["none", "sm", "md", "lg", "xl", "full"],
  shadow: ["sm", "md", "lg", "none"],
  weight: ["light", "regular", "medium", "semibold", "bold", "black"],
  fit: ["cover", "contain"],
  type: ["bar", "line", "area", "pie", "donut"],
  header: ["on", "off"],
  footer: ["on", "off"],
  overlay: ["on", "off"],
  cover: ["true", "false"],
  hidden: ["true", "false"],
  enter: ["fade", "up", "down", "left", "right", "zoom", "none"],
  transition: ["fade", "slide", "zoom", "none"],
  nav: ["on", "off"],
  height: ["auto"],
};

const fieldNames = ["page", "pages", "title", "author", "date"] as const;

export interface ExternSuggestion {
  name: string;
  detail?: string;
  declaration: string;
}

function externOption(suggestion: ExternSuggestion): Completion {
  return {
    label: suggestion.name,
    detail: suggestion.detail,
    type: "variable",
    apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
      const line = view.state.doc.lineAt(from);
      const start = line.from + (line.text.length - line.text.trimStart().length);
      view.dispatch({ changes: { from: start, to, insert: suggestion.declaration }, selection: { anchor: start + suggestion.declaration.length } });
    },
  };
}

export function ribaltaCompletions(getContext: () => { theme: Theme | null; styles: string[]; components: string[]; variables?: string[]; externs?: ExternSuggestion[]; labels: CompletionMessages }) {
  return (context: CompletionContext): CompletionResult | null => {
    const line = context.state.doc.lineAt(context.pos);
    const before = line.text.slice(0, context.pos - line.from);
    const { theme, styles, components, variables = [], externs = [], labels } = getContext();
    const { containers, leaves, lines } = snippetsFor(labels);
    const attributes: Completion[] = attributeNames.map((label) => ({ label, detail: labels.attributes[label], type: "property" }));

    const extern = /^\s*@extern\s+([\w-]*)$/.exec(before);
    if (extern) {
      const declared = new Set<string>();
      for (let number = 1; number <= context.state.doc.lines; number++) {
        if (number === line.number) continue;
        const declaration = /^\s*@extern\s+([\w-]+)/.exec(context.state.doc.line(number).text);
        if (declaration) declared.add(declaration[1]!);
      }
      const options = externs.filter((suggestion) => !declared.has(suggestion.name)).map(externOption);
      return options.length ? { from: context.pos - extern[1]!.length, options } : null;
    }
    const icon = /^\s*::\s*icon\s+([\w-]*)$/.exec(before);
    if (icon) {
      return { from: context.pos - icon[1]!.length, options: Object.keys(icons).map((label) => ({ label, type: "constant" })) };
    }
    const block = /^\s*(:{2,3})\s*([\w-]*)$/.exec(before);
    if (block) {
      const container = block[1] === ":::";
      return {
        from: context.pos - block[2]!.length,
        options: [
          ...(container ? containers : leaves),
          ...components.map((label) => ({ label, detail: labels.blocks.component, type: "function" })),
        ],
      };
    }
    const field = /\$([\w-]*)$/.exec(before);
    if (field) {
      return {
        from: context.pos - field[1]!.length,
        options: [
          ...variables.map((label) => ({ label, detail: labels.fields.variable, type: "variable" })),
          { label: "step", detail: labels.fields.step, type: "variable" },
          ...fieldNames.map((label) => ({ label, detail: labels.fields[label], type: "variable" })),
        ],
      };
    }
    const command = /^\s*\/([a-z-]*)$/.exec(before);
    if (command) {
      return { from: context.pos - command[1]!.length - 1, options: lines };
    }
    const directive = /^\s*@(\w*)$/.exec(before);
    if (directive) return { from: context.pos - directive[1]!.length, options: directives };

    const open = before.lastIndexOf("{");
    if (open === -1 || before.lastIndexOf("}") > open) return null;
    const inside = before.slice(open + 1);
    const style = /\.([\w-]*)$/.exec(inside);
    if (style) {
      const colors = theme ? Object.keys(theme.colors) : [];
      return {
        from: context.pos - style[1]!.length,
        options: [...styles.map((label) => ({ label, type: "class" })), ...colors.map((label) => ({ label, detail: labels.blocks.color, type: "class" }))],
      };
    }
    const value = /([\w-]+)=([\w-]*)$/.exec(inside);
    if (value) {
      const key = value[1]!;
      const colors = theme ? Object.keys(theme.colors) : [];
      const fonts = theme ? Object.keys(theme.fonts) : [];
      const options = key === "color" || key === "bg" ? colors : key === "font" ? fonts : (enumValues[key] ?? []);
      return { from: context.pos - value[2]!.length, options: options.map((label) => ({ label, type: "enum" })) };
    }
    const key = /(?:^|\s)([\w-]*)$/.exec(inside);
    if (key) return { from: context.pos - key[1]!.length, options: attributes };
    return null;
  };
}
