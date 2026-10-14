import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { instantiateCompiler, type Compiler, type Document } from "@ribaltalang/compiler";
import { goTo, startSession } from "@ribaltalang/player/navigation";
import { DocumentView } from "@ribaltalang/renderer/DocumentView";
import { PageView } from "@ribaltalang/renderer/PageView";

const source = [
  "# Photosynthesis {#photosynthesis}",
  "",
  "[Go to chlorophyll](#chlorophyll)",
  "---",
  "Second page {#second}",
  "",
  "- Chlorophyll absorbs light {#chlorophyll}",
  "- Other",
].join("\n");

let compiler: Compiler;
let doc: Document;
const scrolled: Element[] = [];

beforeAll(async () => {
  compiler = await instantiateCompiler(readFileSync(resolve(__dirname, "../packages/compiler/ribalta.wasm")));
  doc = compiler(source).document!;
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      disconnect() {}
    },
  );
  Element.prototype.scrollIntoView = function () {
    scrolled.push(this);
  };
});

afterEach(() => {
  cleanup();
  scrolled.length = 0;
});

describe("block anchors", () => {
  it("renders the block id as data-anchor on blocks and list items", () => {
    const { container } = render(<DocumentView doc={doc} />);
    expect(container.querySelector("h1")!.getAttribute("data-anchor")).toBe("photosynthesis");
    expect(container.querySelector("li")!.getAttribute("data-anchor")).toBe("chlorophyll");
    expect(container.querySelector("[id]")).toBeNull();
  });

  it("scrolls to the block when an internal link targets an anchor in the continuous view", () => {
    const { container } = render(<DocumentView doc={doc} />);
    fireEvent.click(container.querySelector("a.ribalta-link")!);
    expect(scrolled).toEqual([container.querySelector('[data-anchor="chlorophyll"]')]);
  });

  it("forwards page targets to onAction unchanged", () => {
    const onAction = vi.fn();
    const pageDoc = compiler("# One\n\n[go](#two)\n---\n@page two\n# Two");
    const { container } = render(<DocumentView doc={pageDoc.document!} onAction={onAction} />);
    fireEvent.click(container.querySelector("a.ribalta-link")!);
    expect(onAction).toHaveBeenCalledWith([{ type: "go", target: "two" }]);
    expect(scrolled).toEqual([]);
  });

  it("keeps interaction disabled without onAction while links still scroll", () => {
    const interactive = compiler("@state name = \"\"\n\n[jump](#target)\n\n:: input {bind=$name}\n\nClick me {click=\"toggle $flag\"}\n\n# Title {#target}\n").document!;
    const { container } = render(<DocumentView doc={interactive} />);
    const input = container.querySelector("input");
    expect(input!.disabled).toBe(true);
    expect(container.querySelector("[data-interactive]:not(a):not(input)")).toBeNull();
    fireEvent.click(container.querySelector("a.ribalta-link")!);
    expect(scrolled).toEqual([container.querySelector('[data-anchor="target"]')]);
  });

  it("scrolls a paged view to the requested anchor", () => {
    const { container, rerender } = render(<PageView doc={doc} page={doc.pages[1]!} />);
    expect(scrolled).toEqual([]);
    rerender(<PageView doc={doc} page={doc.pages[1]!} anchor="chlorophyll" />);
    expect(scrolled).toEqual([container.querySelector('[data-anchor="chlorophyll"]')]);
  });
});

describe("navigation to anchors", () => {
  it("re-scrolls a paged view on repeated navigation to the same anchor", () => {
    const first = goTo(doc, startSession(doc), "chlorophyll");
    const { container, rerender } = render(<PageView doc={doc} page={doc.pages[first.index]!} anchor={first.anchor} anchorToken={first.anchorToken} />);
    expect(scrolled).toHaveLength(1);
    const second = goTo(doc, first, "chlorophyll");
    expect(second.anchorToken).not.toBe(first.anchorToken);
    rerender(<PageView doc={doc} page={doc.pages[second.index]!} anchor={second.anchor} anchorToken={second.anchorToken} />);
    expect(scrolled).toEqual(Array(2).fill(container.querySelector('[data-anchor="chlorophyll"]')));
  });

  it("go to an anchor moves to its page and records the anchor", () => {
    const session = goTo(doc, startSession(doc), "chlorophyll");
    expect(session.index).toBe(1);
    expect(session.anchor).toBe("chlorophyll");
  });

  it("go to a page clears the anchor", () => {
    const anchored = goTo(doc, startSession(doc), "chlorophyll");
    const session = goTo(doc, anchored, doc.pages[0]!.id);
    expect(session.index).toBe(0);
    expect(session.anchor).toBeUndefined();
  });

  it("go to an unknown target keeps the session", () => {
    const session = startSession(doc);
    expect(goTo(doc, session, "missing")).toBe(session);
  });
});
