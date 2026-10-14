import { mkdirSync, readdirSync, rmSync, statSync, symlinkSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const site = fileURLToPath(new URL("..", import.meta.url));
const source = join(site, "../docs");
const target = join(site, "src/content/docs");

function files(directory) {
  return readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    return statSync(path).isDirectory() ? files(path) : [path];
  });
}

rmSync(target, { recursive: true, force: true });
const pages = files(source);
for (const file of pages) {
  const link = join(target, relative(source, file));
  mkdirSync(dirname(link), { recursive: true });
  symlinkSync(relative(dirname(link), file), link);
}
console.log(`docs: ${pages.length} pages`);
