import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { instantiateCompiler, type Compiler } from "@ribaltalang/compiler";

let compiler: Promise<Compiler> | null = null;

export function testCompiler() {
  compiler ??= instantiateCompiler(readFileSync(createRequire(import.meta.url).resolve("@ribaltalang/compiler/ribalta.wasm")));
  return compiler;
}
