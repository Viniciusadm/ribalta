import { useCallback, useRef } from "react";
import type { Action, Document, Scalar } from "@ribaltalang/compiler";
import { scrollToAnchor } from "./anchors";
import { PageView, ScaledPage } from "./PageView";

export function DocumentView({
  doc,
  state,
  onAction,
  gap = 24,
  className,
}: {
  doc: Document;
  state?: Record<string, Scalar>;
  onAction?: (actions: Action[]) => void;
  gap?: number;
  className?: string;
}) {
  const root = useRef<HTMLDivElement>(null);
  const run = useCallback(
    (actions: Action[]) => {
      const forwarded = actions.filter((action) => {
        if (action.type !== "go" || !root.current) return true;
        const isAnchor = doc.anchors?.[action.target] !== undefined && !doc.pages.some((page) => page.id === action.target);
        return !(isAnchor && scrollToAnchor(root.current, action.target));
      });
      if (forwarded.length) onAction?.(forwarded);
    },
    [doc, onAction],
  );
  return (
    <div ref={root} className={className} style={{ display: "flex", flexDirection: "column", gap }}>
      {doc.pages
        .filter((page) => !page.hidden)
        .map((page, index) => (
          <ScaledPage key={`${page.id}-${index}`} width={page.width} height={page.height} fit="width">
            <PageView doc={doc} page={page} state={state} onAction={run} />
          </ScaledPage>
        ))}
    </div>
  );
}
