import { useEffect } from "react";
import type { FontFamily, Theme } from "@ribaltalang/compiler";
import { localFontFace } from "./bundledFonts";
import { useRendererHost, type RendererHost } from "./host";

const safeFamily = /^[A-Za-z0-9 ]+$/;

function familySpec({ family, weights }: FontFamily) {
  const name = family.replaceAll(" ", "+");
  return weights.length ? `${name}:wght@${weights.join(";")}` : name;
}

export function googleFontsQuery(families: FontFamily[]) {
  return families.map((family) => `family=${familySpec(family)}`).join("&");
}

export function fontStylesheetUrl(families: FontFamily[], host: RendererHost | null) {
  const remote = families.filter(({ family, weights }) => safeFamily.test(family) && weights.every(Number.isInteger) && !localFontFace(family));
  if (!remote.length || !host?.fontStylesheet) return null;
  return host.fontStylesheet(remote);
}

export function useThemeFonts(theme: Theme | undefined) {
  const host = useRendererHost();
  const url = theme ? fontStylesheetUrl(theme.fontFamilies, host) : null;
  useEffect(() => {
    if (!url || document.querySelector(`link[data-ribalta-fonts="${CSS.escape(url)}"]`)) return;
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = url;
    link.dataset["ribaltaFonts"] = url;
    document.head.appendChild(link);
  }, [url]);
}
