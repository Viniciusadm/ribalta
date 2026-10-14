import { readFileSync } from "node:fs";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import { defaultClientConditions, defaultServerConditions } from "vite";
import { remarkBaseLinks } from "./plugins/remark-base-links.mjs";
import { remarkRibalta } from "./plugins/remark-ribalta.mjs";
import { sections } from "./sidebar.mjs";

const grammar = JSON.parse(readFileSync(new URL("../editors/vscode/syntaxes/ribalta.tmLanguage.json", import.meta.url), "utf8"));
const base = "/ribalta";

export default defineConfig({
  vite: {
    resolve: { conditions: ["source", ...defaultClientConditions] },
    ssr: { resolve: { conditions: ["source", ...defaultServerConditions], externalConditions: ["source"] } },
  },
  site: "https://viniciusadm.github.io",
  base,
  trailingSlash: "always",
  markdown: { remarkPlugins: [remarkRibalta, [remarkBaseLinks, { base }]] },
  integrations: [
    starlight({
      title: "Ribalta",
      description: "Documentação da Ribalta, uma linguagem de texto para visuais com tema.",
      logo: { src: "./src/assets/logo.svg", alt: "Ribalta" },
      favicon: "/favicon.svg",
      defaultLocale: "root",
      locales: {
        root: { label: "Português", lang: "pt-BR" },
        en: { label: "English", lang: "en" },
      },
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/Viniciusadm/ribalta" }],
      customCss: ["@fontsource-variable/inter", "@fontsource-variable/jetbrains-mono", "@ribaltalang/renderer/styles.css", "./src/styles/custom.css"],
      components: { PageTitle: "./src/components/PageTitle.astro" },
      expressiveCode: { shiki: { langs: [{ ...grammar, name: "ribalta" }] } },
      lastUpdated: false,
      sidebar: sections.map(({ label, en, items }) => ({ label, translations: { en }, items })),
    }),
  ],
});
