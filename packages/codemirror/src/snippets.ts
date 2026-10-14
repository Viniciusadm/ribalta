import { snippetCompletion, type Completion } from "@codemirror/autocomplete";
import type { CompletionMessages } from "./messages";

export function snippetsFor(labels: CompletionMessages) {
  const s = labels.snippets;
  const blocks = labels.blocks;
  const containerTemplates: Record<string, string> = {
    columns: `columns {gap=lg}\n::: stack\n\${1:${s.columnsLeft}}\n:::\n::: stack\n\${2:${s.columnsRight}}\n:::\n:::`,
    stack: "stack {gap=md}\n${1}\n:::",
    row: "row {gap=md}\n${1}\n:::",
    grid: `grid {cols=\${1:3} gap=lg}\n::: box {.card}\n\${2:${s.item}}\n:::\n:::`,
    box: "box {.card}\n${1}\n:::",
    steps: `steps\n- \${1:${s.first}}\n- \${2:${s.second}}\n- \${3:${s.third}}\n:::`,
    chart: `chart {type=\${1:bar}}\n| \${2:${s.label}} | \${3:${s.value}} |\n| --- | ---: |\n| A | 10 |\n| B | 20 |\n:::`,
    notes: `notes\n\${1:${s.notes}}\n:::`,
    slot: `slot \${1:${s.slotName}}\n\${2}\n:::`,
    if: `if \${1:${s.condition}}\n\${2}\n:::`,
    else: "else\n${1}\n:::",
    for: `for \${1:${s.itemName}} in \${2:${s.list}}\n\${3}\n:::`,
  };
  const containers: Completion[] = Object.entries(containerTemplates).map(([label, template]) =>
    snippetCompletion(template, { label, detail: blocks[label as keyof typeof blocks], type: "class" }),
  );
  const leaves: Completion[] = [
    snippetCompletion("icon ${1:star} {size=${2:3xl} color=${3:accent}}", { label: "icon", detail: blocks.icon, type: "class" }),
    snippetCompletion("spacer ${1:lg}", { label: "spacer", detail: blocks.spacer, type: "class" }),
    { label: "divider", detail: blocks.divider, type: "class" },
    snippetCompletion("input {bind=$${1:nome} placeholder=\"${2:}\"}", { label: "input", detail: blocks.input, type: "class" }),
    snippetCompletion("sound ${1:https://} {name=${2:som}}", { label: "sound", detail: blocks.sound, type: "class" }),
    snippetCompletion(`table {data=\${1:${s.list}}}`, { label: "table", detail: blocks.table, type: "class" }),
  ];
  const cards = Array.from(
    { length: 3 },
    (_, index) => `::: box {.card align=center}\n## \${${index * 2 + 1}:${s.cards.value}}\n\${${index * 2 + 2}:${s.cards.caption}} {.muted}\n:::`,
  ).join("\n");
  const lines: Completion[] = [
    snippetCompletion(`--- {\${1:align=center}}\n# \${2:${s.page.title}}\n\n\${3}`, { label: s.page.label, detail: s.page.detail, type: "text" }),
    snippetCompletion(
      `::: columns {gap=xl widths="3 2"}\n::: stack {gap=md}\n# \${1:${s.textAndImage.title}}\n\${2:${s.textAndImage.text}}\n:::\n![\${3:${s.textAndImage.alt}}](\${4:https://})\n:::`,
      { label: s.textAndImage.label, detail: s.textAndImage.detail, type: "text" },
    ),
    snippetCompletion(`::: grid {cols=3 gap=lg}\n${cards}\n:::`, { label: s.cards.label, detail: s.cards.detail, type: "text" }),
    snippetCompletion(`> \${1:${s.quote.quote}}\n> — \${2:${s.quote.author}}`, { label: s.quote.label, detail: s.quote.detail, type: "text" }),
    snippetCompletion("```${1:js}\n${2}\n```", { label: s.code.label, detail: s.code.detail, type: "text" }),
    snippetCompletion(`| \${1:${s.table.column}} | \${2:${s.table.column}} |\n| --- | --- |\n| \${3} | \${4} |`, {
      label: s.table.label,
      detail: s.table.detail,
      type: "text",
    }),
    snippetCompletion(`![\${1:${s.image.alt}}](\${2:https://}){fit=\${3:cover} radius=lg}`, { label: s.image.label, detail: s.image.detail, type: "text" }),
    snippetCompletion(`@header\n::: columns {valign=center}\n\${1:${s.header.text}}\n\n$title {align=right}\n:::\n@end`, {
      label: s.header.label,
      detail: s.header.detail,
      type: "keyword",
    }),
    snippetCompletion(`@footer\n::: columns\n\${1:${s.footer.text}}\n\n$page / $pages {align=right}\n:::\n@end`, {
      label: s.footer.label,
      detail: s.footer.detail,
      type: "keyword",
    }),
    snippetCompletion(`@overlay {align=right}\n\${1:${s.overlay.text}} {color=accent weight=bold uppercase}\n@end`, {
      label: s.overlay.label,
      detail: s.overlay.detail,
      type: "keyword",
    }),
    snippetCompletion(`@component \${1:${s.component.name}} {\${2:${s.component.param}}}\n::: box {.card}\n## $\${2:${s.component.param}}\n$content\n:::\n@end`, {
      label: s.component.label,
      detail: s.component.detail,
      type: "keyword",
    }),
  ];
  lines.push(
    snippetCompletion(`@data \${1:${s.data.name}}\n| \${2:${s.data.column}} |\n| --- |\n| \${3} |\n@end`, { label: s.data.label, detail: s.data.detail, type: "keyword" }),
    snippetCompletion(`@state \${1:${s.state.name}} = false\n\n[\${2:${s.state.text}}]{click="toggle $\${1:${s.state.name}}"}\n::: if $\${1:${s.state.name}}\n\${3}\n:::`, {
      label: s.state.label,
      detail: s.state.detail,
      type: "keyword",
    }),
  );
  return { containers, leaves, lines };
}
