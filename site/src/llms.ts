import { getCollection, type CollectionEntry } from "astro:content";
import { iconNames } from "../.generated/render.mjs";
import { sections } from "../sidebar.mjs";

export type DocsLocale = "pt" | "en";
type Entry = CollectionEntry<"docs">;

const site = (import.meta.env.SITE ?? "").replace(/\/$/, "");
const base = import.meta.env.BASE_URL.replace(/\/$/, "");

export function slugOf(entry: Entry) {
  return entry.id.replace(/^en\//, "");
}

export function localeOf(entry: Entry): DocsLocale {
  return entry.id === "en" || entry.id.startsWith("en/") ? "en" : "pt";
}

export function markdownPath(slug: string) {
  return `${base}/${slug}.md`;
}

export function markdownUrl(entry: Entry) {
  return `${site}${markdownPath(slugOf(entry))}`;
}

export async function englishDocs() {
  return docsFor("en");
}

export async function docsFor(locale: DocsLocale) {
  const entries = (await getCollection("docs")).filter((entry) => localeOf(entry) === locale);
  const order = sections.flatMap((section) => section.items);
  return entries.sort((a, b) => rank(order, slugOf(a)) - rank(order, slugOf(b)));
}

function rank(order: string[], slug: string) {
  const index = order.indexOf(slug);
  return index === -1 ? (slug === "index" ? -1 : order.length) : index;
}

export function markdownOf(entry: Entry) {
  const body = (entry.body ?? "")
    .replaceAll("](/", `](${site}${base}/`)
    .replace(/```ribalta preview\b/g, "```ribalta")
    .replace(/```ribalta-icons\s*\n[\s\S]*?```/g, iconNames().map((name: string) => `- \`${name}\``).join("\n"));
  const description = entry.data.description ? `> ${entry.data.description}\n\n` : "";
  return `# ${entry.data.title}\n\n${description}${body.trim()}\n`;
}

const summary =
  "Ribalta is a Markdown-inspired text language for themed visuals, with `:::` blocks, `{...}` attributes, themes, components, charts and interactivity. Its compiler turns the text into a JSON document that renderers draw.";

export async function llmsIndex() {
  const entries = await englishDocs();
  const lines = ["# Ribalta", "", `> ${summary}`, ""];
  for (const section of sections) {
    lines.push(`## ${section.en}`, "");
    for (const slug of section.items) {
      const entry = entries.find((item) => slugOf(item) === slug);
      if (entry) lines.push(`- [${entry.data.title}](${markdownUrl(entry)}): ${entry.data.description ?? ""}`.trimEnd());
    }
    lines.push("");
  }
  lines.push("## Optional", "", `- [Full documentation in one file](${site}${base}/llms-full.txt)`, "");
  return lines.join("\n");
}

export async function llmsFull() {
  const entries = await englishDocs();
  return [`# Ribalta documentation\n\n> ${summary}\n`, ...entries.map(markdownOf)].join("\n---\n\n");
}

export function textResponse(body: string, type = "text/plain") {
  return new Response(body, { headers: { "Content-Type": `${type}; charset=utf-8` } });
}
