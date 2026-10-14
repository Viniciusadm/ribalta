import type { CompileOptions, Compilation, Conflict, DefRef, Definition, Diagnostic, Extraction, ItemKind, Locale, MergeResult, Resolution } from "./types";

interface RibaltaExports {
  memory: WebAssembly.Memory;
  ribalta_alloc(length: number): number;
  ribalta_free(pointer: number, length: number): void;
  ribalta_compile(preludePointer: number, preludeLength: number, pointer: number, length: number, locale: number, optionsPointer: number, optionsLength: number): bigint;
  ribalta_check_prelude(pointer: number, length: number, locale: number): bigint;
  ribalta_definitions(pointer: number, length: number): bigint;
  ribalta_extract(sourcePointer: number, sourceLength: number, rootsPointer: number, rootsLength: number): bigint;
  ribalta_conflicts(themePointer: number, themeLength: number, snippetPointer: number, snippetLength: number): bigint;
  ribalta_merge(
    themePointer: number,
    themeLength: number,
    snippetPointer: number,
    snippetLength: number,
    resolutionsPointer: number,
    resolutionsLength: number,
    labelPointer: number,
    labelLength: number,
  ): bigint;
  ribalta_validate_item(
    kindPointer: number,
    kindLength: number,
    sourcePointer: number,
    sourceLength: number,
    entryPointer: number,
    entryLength: number,
    locale: number,
  ): bigint;
}

export interface LibraryCompiler {
  definitions(source: string): Definition[];
  extract(source: string, roots: DefRef[]): Extraction;
  conflicts(theme: string, snippet: string): Conflict[];
  merge(theme: string, snippet: string, resolutions: Resolution[], label?: string): MergeResult;
  validateItem(kind: ItemKind, source: string, entry?: string | null, locale?: Locale): Diagnostic[];
}

export interface Compiler {
  (source: string, prelude?: string, locale?: Locale, options?: CompileOptions): Compilation;
  checkPrelude(prelude: string, locale?: Locale): Diagnostic[];
  library: LibraryCompiler;
}

export function compilerFromInstance(instance: WebAssembly.Instance): Compiler {
  const exports = instance.exports as unknown as RibaltaExports;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  const write = (text: string) => {
    const bytes = encoder.encode(text);
    const pointer = exports.ribalta_alloc(bytes.length);
    new Uint8Array(exports.memory.buffer, pointer, bytes.length).set(bytes);
    return { pointer, length: bytes.length };
  };

  const read = <T>(packed: bigint): T => {
    const pointer = Number(packed >> 32n);
    const length = Number(packed & 0xffffffffn);
    const json = decoder.decode(new Uint8Array(exports.memory.buffer, pointer, length));
    exports.ribalta_free(pointer, length);
    return JSON.parse(json) as T;
  };

  const localeCode = (locale: Locale | undefined) => (locale === "en" ? 1 : 0);

  const call = <T>(texts: string[], run: (inputs: { pointer: number; length: number }[]) => bigint): T => {
    const inputs = texts.map(write);
    const packed = run(inputs);
    for (const input of inputs) exports.ribalta_free(input.pointer, input.length);
    return read<T>(packed);
  };

  const compile = ((source: string, prelude = "", locale?: Locale, options: CompileOptions = {}) =>
    call([prelude, source, JSON.stringify(options)], ([base, text, settings]) =>
      exports.ribalta_compile(base!.pointer, base!.length, text!.pointer, text!.length, localeCode(locale), settings!.pointer, settings!.length),
    )) as Compiler;

  compile.checkPrelude = (prelude: string, locale?: Locale) => {
    const input = write(prelude);
    const packed = exports.ribalta_check_prelude(input.pointer, input.length, localeCode(locale));
    exports.ribalta_free(input.pointer, input.length);
    return read<Diagnostic[]>(packed);
  };

  compile.library = {
    definitions: (source) => call([source], ([input]) => exports.ribalta_definitions(input!.pointer, input!.length)),
    extract: (source, roots) =>
      call([source, JSON.stringify(roots)], ([text, list]) => exports.ribalta_extract(text!.pointer, text!.length, list!.pointer, list!.length)),
    conflicts: (theme, snippet) =>
      call([theme, snippet], ([base, incoming]) => exports.ribalta_conflicts(base!.pointer, base!.length, incoming!.pointer, incoming!.length)),
    merge: (theme, snippet, resolutions, label = "") =>
      call([theme, snippet, JSON.stringify(resolutions), label], ([base, incoming, choices, title]) =>
        exports.ribalta_merge(base!.pointer, base!.length, incoming!.pointer, incoming!.length, choices!.pointer, choices!.length, title!.pointer, title!.length),
      ),
    validateItem: (kind, source, entry, locale) =>
      call([kind, source, entry ?? ""], ([type, text, name]) =>
        exports.ribalta_validate_item(type!.pointer, type!.length, text!.pointer, text!.length, name!.pointer, name!.length, localeCode(locale)),
      ),
  };

  return compile;
}

export async function instantiateCompiler(bytes: BufferSource): Promise<Compiler> {
  const { instance } = await WebAssembly.instantiate(bytes, {});
  return compilerFromInstance(instance);
}

export async function loadCompiler(wasm: string | URL = new URL("../ribalta.wasm", import.meta.url)): Promise<Compiler> {
  const response = await fetch(wasm);
  return instantiateCompiler(await response.arrayBuffer());
}

let shared: Promise<Compiler> | null = null;

export function getCompiler(wasm?: string | URL) {
  shared ??= loadCompiler(wasm).catch((error: unknown) => {
    shared = null;
    throw error;
  });
  return shared;
}
