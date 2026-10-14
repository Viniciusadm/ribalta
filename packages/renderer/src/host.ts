import { createContext, useContext } from "react";
import type { FontFamily } from "@ribaltalang/compiler";

export interface RendererHost {
  asset(id: string): string;
  fontStylesheet?(families: FontFamily[]): string;
}

export const RendererHostContext = createContext<RendererHost | null>(null);

export function useRendererHost() {
  return useContext(RendererHostContext);
}
