# @ribaltalang/compiler

Compiler for [Ribalta](https://folioslides.com.br/docs/), a text language for themed visuals, packaged as WebAssembly for the browser and Node.js. It turns Ribalta source into a JSON document (the IR) plus localized diagnostics.

```sh
npm install @ribaltalang/compiler
```

## Usage

```ts
import { loadCompiler } from "@ribaltalang/compiler";

const compile = await loadCompiler();
const { document, diagnostics } = compile('@extern title = "Hello"\n\n# $title', "", "en");
```

- `loadCompiler(url?)` fetches `ribalta.wasm`, which ships next to the code. Modern bundlers (Vite, webpack 5, Rollup, esbuild) find it through `new URL(..., import.meta.url)`. With Vite, exclude the package from dependency pre-bundling, or pass the address yourself:

  ```ts
  import wasmUrl from "@ribaltalang/compiler/ribalta.wasm?url";
  const compile = await loadCompiler(wasmUrl);
  ```

- In Node.js, read the file and call `instantiateCompiler`:

  ```ts
  import { readFile } from "node:fs/promises";
  import { createRequire } from "node:module";
  import { instantiateCompiler } from "@ribaltalang/compiler";

  const wasm = await readFile(createRequire(import.meta.url).resolve("@ribaltalang/compiler/ribalta.wasm"));
  const compile = await instantiateCompiler(wasm);
  ```

- `compile(source, prelude?, locale?, options?)` returns `{ document, diagnostics }`. `prelude` is an optional theme applied before the source; `locale` is `"pt"` (default) or `"en"`; `options.page` is the host's default page size, such as `{ width: 1920, height: 1080 }` for slides (without it, pages are 1920 wide and as tall as their content).
- The IR is documented in [IR.md](https://github.com/Viniciusadm/ribalta/blob/master/crates/ribalta/IR.md). Draw it with [`@ribaltalang/renderer`](https://www.npmjs.com/package/@ribaltalang/renderer).
- Compiling on a server written in Rust? Use the [`ribalta`](https://crates.io/crates/ribalta) crate instead.

## License

MIT or Apache-2.0, at your option.
