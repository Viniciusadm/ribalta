import type { Action, Document, Scalar } from "@ribaltalang/compiler";
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
  return (
    <div className={className} style={{ display: "flex", flexDirection: "column", gap }}>
      {doc.pages
        .filter((page) => !page.hidden)
        .map((page, index) => (
          <ScaledPage key={`${page.id}-${index}`} width={page.width} height={page.height} fit="width">
            <PageView doc={doc} page={page} state={state} onAction={onAction} />
          </ScaledPage>
        ))}
    </div>
  );
}
