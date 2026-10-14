import { useCallback } from "react";
import type { ImageSource } from "@ribaltalang/compiler";
import { useRendererHost, type RendererHost } from "./host";

export type SourceResolver = (src: ImageSource) => string;

export function resolveSource(src: ImageSource, host: RendererHost | null) {
  if (src.kind === "url") return src.url;
  return host ? host.asset(src.id) : "";
}

export function useResolveSource(): SourceResolver {
  const host = useRendererHost();
  return useCallback((src: ImageSource) => resolveSource(src, host), [host]);
}

export function sourceText(src: ImageSource) {
  return src.kind === "asset" ? `asset:${src.id}` : src.url;
}
