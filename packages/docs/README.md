# @ribaltalang/docs

The documentation of the [Ribalta](https://github.com/Viniciusadm/ribalta) language as Markdown pages: writing, page sizes and documents, layout, styling, themes, components, data, interactivity, charts, images and the reference of syntax, attributes, blocks, themes, icons and errors.

Pages live under `pages/`, in Portuguese at the root and in English under `pages/en/`, with the same relative paths in both languages. A site that hosts them resolves the folder from the package:

```js
import { createRequire } from "node:module";
import { dirname, join } from "node:path";

const pages = join(dirname(createRequire(import.meta.url).resolve("@ribaltalang/docs/package.json")), "pages");
```

## License

Licensed under either of MIT or Apache-2.0, at your option.
