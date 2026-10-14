import { useLayoutEffect, useRef, useState } from "react";
import type { Document, Page } from "@ribaltalang/compiler";
import { PageView } from "./PageView";

export function StaticPage({ doc, page, fallbackHeight }: { doc: Document; page: Page; fallbackHeight?: number }) {
  const content = useRef<HTMLDivElement>(null);
  const [measured, setMeasured] = useState<number | null>(null);
  const width = page.width;
  const height = page.height ?? measured ?? fallbackHeight ?? Math.round((width * 9) / 16);

  useLayoutEffect(() => {
    if (page.height !== null || !content.current) return;
    const element = content.current;
    const update = () => setMeasured(element.offsetHeight);
    update();
    const observer = new ResizeObserver(update);
    observer.observe(element);
    return () => observer.disconnect();
  }, [page.height]);

  return (
    <svg className="ribalta-static" viewBox={`0 0 ${width} ${height}`} role="img" aria-label={page.title}>
      <foreignObject x={0} y={0} width={width} height={height} aria-hidden="true">
        <div ref={content}>
          <PageView doc={doc} page={page} />
        </div>
      </foreignObject>
    </svg>
  );
}
