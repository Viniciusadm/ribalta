import type { Action, Block, Document, Inline } from "@ribaltalang/compiler";
import { holds, type Scope } from "@ribaltalang/renderer/expr";
import type { Session } from "./navigation";

type Shortcuts = Map<string, Action[]>;

export function shortcutKey(event: KeyboardEvent): string | null {
  if (event.ctrlKey || event.metaKey || event.altKey) return null;
  if (event.key === " ") return "Space";
  if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Enter"].includes(event.key)) return event.key;
  return /^[a-z0-9]$/i.test(event.key) ? event.key.toLowerCase() : null;
}

export function pageShortcuts(doc: Document, session: Session, step = session.step): Shortcuts {
  const page = doc.pages[session.index];
  const found: Shortcuts = new Map();
  if (!page) return found;
  const scope: Scope = { state: { ...doc.state, ...page.state, ...session.state }, step, elapsed: session.elapsed };
  const visible = (item: { when?: Block["when"]; step?: number }) => holds(item.when, scope) && (item.step === undefined || item.step <= step);
  const claim = (item: { key?: string; click?: Action[] }) => {
    if (item.key && item.click?.length && !found.has(item.key)) found.set(item.key, item.click);
  };
  const inlines = (nodes: Inline[]) => {
    for (const node of nodes) {
      if (node.type === "span") {
        if (!visible(node)) continue;
        claim(node);
      }
      if ("children" in node) inlines(node.children);
    }
  };
  const blocks = (list: Block[]) => {
    for (const block of list) {
      if (!visible(block)) continue;
      claim(block);
      switch (block.type) {
        case "heading":
        case "paragraph":
          inlines(block.content);
          break;
        case "quote":
        case "layout":
          blocks(block.children);
          break;
        case "list":
          for (const item of block.items) {
            if (!visible(item)) continue;
            claim(item);
            inlines(item.content);
            blocks(item.children);
          }
          break;
        case "table":
          for (const cell of [...block.header, ...block.rows.flat()]) inlines(cell);
          break;
      }
    }
  };
  blocks([page.header, ...page.children, page.footer, page.overlay].filter((block): block is Block => block !== undefined));
  return found;
}

export function runShortcut(doc: Document, session: Session, event: KeyboardEvent, run: (actions: Action[]) => void, step?: number) {
  const key = shortcutKey(event);
  const actions = key ? pageShortcuts(doc, session, step).get(key) : undefined;
  if (!actions) return false;
  event.preventDefault();
  run(actions);
  return true;
}
