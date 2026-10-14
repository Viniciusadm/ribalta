import { createRequire } from "node:module";
import { renderIconGrid, renderPreview } from "../.generated/render.mjs";

const wasmPath = createRequire(import.meta.url).resolve("@ribaltalang/compiler/ribalta.wasm");

function localeOf(file) {
  const path = file?.path ?? file?.history?.[0] ?? "";
  return /[/\\](?:content[/\\]docs|docs)[/\\]en[/\\]/.test(path) ? "en" : "pt";
}

function collect(node, parent, found) {
  if (node.type === "code" && parent) found.push({ node, parent });
  for (const child of node.children ?? []) collect(child, node, found);
}

export function remarkRibalta() {
  return async (tree, file) => {
    const blocks = [];
    collect(tree, null, blocks);
    const locale = localeOf(file);
    for (const { node, parent } of blocks) {
      if (node.lang === "ribalta-icons") {
        const index = parent.children.indexOf(node);
        parent.children.splice(index, 1, { type: "html", value: renderIconGrid() });
        continue;
      }
      if (node.lang !== "ribalta" || !/\bpreview\b/.test(node.meta ?? "")) continue;
      const preview = await renderPreview(wasmPath, node.value, locale);
      const errors = preview.diagnostics.filter((diagnostic) => diagnostic.severity === "error");
      if (errors.length) {
        const details = errors.map((error) => `  line ${error.line}: ${error.message}`).join("\n");
        throw new Error(`Invalid Ribalta example in ${file.path}:\n${details}\n---\n${node.value}`);
      }
      node.meta = (node.meta ?? "").replace(/\bpreview\b/, "").trim() || null;
      const index = parent.children.indexOf(node);
      parent.children.splice(index + 1, 0, { type: "html", value: preview.html });
    }
  };
}
