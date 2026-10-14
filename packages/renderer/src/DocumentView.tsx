import { useCallback, useRef } from "react";
import type { Action, Document, Scalar } from "@ribaltalang/compiler";
import { isAnchorTarget, scrollToAnchor } from "./anchors";
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
  const scrollTo = useCallback((id: string) => Boolean(root.current) && scrollToAnchor(root.current!, id), []);
  const run = useCallback(
    (actions: Action[]) => {
      const forwarded = actions.filter((action) => !(action.type === "go" && isAnchorTarget(doc, action.target) && scrollTo(action.target)));
      if (forwarded.length) onAction?.(forwarded);
    },
    [doc, onAction, scrollTo],
  );
  return (
    <div ref={root} className={className} style={{ display: "flex", flexDirection: "column", gap }}>
      {doc.pages
        .filter((page) => !page.hidden)
        .map((page, index) => (
          <ScaledPage key={`${page.id}-${index}`} width={page.width} height={page.height} fit="width">
            <PageView doc={doc} page={page} state={state} onAction={onAction ? run : undefined} scrollTo={scrollTo} />
          </ScaledPage>
        ))}
    </div>
  );
}
