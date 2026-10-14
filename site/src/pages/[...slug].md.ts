import type { APIRoute, GetStaticPaths } from "astro";
import type { CollectionEntry } from "astro:content";
import { englishDocs, markdownOf, slugOf, textResponse } from "../llms";

export const getStaticPaths = (async () =>
  (await englishDocs()).map((entry) => ({ params: { slug: slugOf(entry) === "en" ? "index" : slugOf(entry) }, props: { entry } }))) satisfies GetStaticPaths;

export const GET: APIRoute<{ entry: CollectionEntry<"docs"> }> = ({ props }) => textResponse(markdownOf(props.entry), "text/markdown");
