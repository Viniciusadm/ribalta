import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { describe, expect, it } from "vitest";

const docs = resolve(__dirname, "../docs");
const compiler = resolve(__dirname, "../crates/ribalta/src");

function files(directory: string): string[] {
  return readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    return statSync(path).isDirectory() ? files(path) : [path];
  });
}

function compilerCodes() {
  const codes = new Set<string>();
  for (const file of files(compiler)) {
    for (const match of readFileSync(file, "utf8").matchAll(/"(F\d{3})"/g)) codes.add(match[1]!);
  }
  return [...codes].sort();
}

describe("documentation", () => {
  const pages = files(docs)
    .filter((file) => file.endsWith(".md"))
    .map((file) => ({ file, id: relative(docs, file) }));
  const portuguese = pages.filter(({ id }) => !id.startsWith("en/")).map(({ id }) => id);
  const english = pages.filter(({ id }) => id.startsWith("en/")).map(({ id }) => id.slice("en/".length));

  it("has every page in both languages", () => {
    expect(english.sort()).toEqual(portuguese.sort());
  });

  it("has no unfinished pages", () => {
    for (const { file } of pages) expect(readFileSync(file, "utf8"), file).not.toMatch(/\bTODO\b/);
  });

  it.each([
    ["pt", "reference/errors.md"],
    ["en", "en/reference/errors.md"],
  ])("documents every compiler error code (%s)", (_, path) => {
    const text = readFileSync(join(docs, path), "utf8");
    const documented = new Set([...text.matchAll(/^###\s+(F\d{3})\b/gm)].map((match) => match[1]));
    const missing = compilerCodes().filter((code) => !documented.has(code));
    expect(missing).toEqual([]);
  });
});
