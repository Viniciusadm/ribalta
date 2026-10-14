import { createContext, useContext } from "react";
import type { Locale } from "@ribaltalang/compiler";

export const defaultLocale: Locale = "pt";
export const htmlLang: Record<Locale, string> = { pt: "pt-BR", en: "en" };

export const LocaleContext = createContext<Locale>(defaultLocale);

export function useLocale() {
  return useContext(LocaleContext);
}
