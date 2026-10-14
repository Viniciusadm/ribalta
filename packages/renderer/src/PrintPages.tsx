import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Document } from "@ribaltalang/compiler";
import { PageView } from "./PageView";

async function waitForAssets(root: HTMLElement) {
  await document.fonts?.ready;
  const images = [...root.querySelectorAll("img")].filter((image) => !image.complete);
  await Promise.all(
    images.map(
      (image) =>
        new Promise<void>((resolve) => {
          image.addEventListener("load", () => resolve(), { once: true });
          image.addEventListener("error", () => resolve(), { once: true });
          setTimeout(resolve, 5000);
        }),
    ),
  );
}

export function PrintPages({
  doc,
  onDone,
  print = () => window.print(),
  printWidth = 1280,
}: {
  doc: Document;
  onDone(): void;
  print?: () => void;
  printWidth?: number;
}) {
  const root = useRef<HTMLDivElement>(null);
  const contents = useRef<(HTMLDivElement | null)[]>([]);
  const [heights, setHeights] = useState<(number | null)[] | null>(null);

  useLayoutEffect(() => {
    setHeights(doc.pages.map((page, index) => page.height ?? contents.current[index]?.offsetHeight ?? page.width));
  }, [doc]);

  useEffect(() => {
    if (!heights) return;
    let cancelled = false;
    const finish = () => {
      if (!cancelled) onDone();
    };
    window.addEventListener("afterprint", finish, { once: true });
    void (async () => {
      if (root.current) await waitForAssets(root.current);
      if (cancelled) return;
      const previousTitle = document.title;
      if (doc.meta.title) document.title = doc.meta.title;
      print();
      document.title = previousTitle;
    })();
    return () => {
      cancelled = true;
      window.removeEventListener("afterprint", finish);
    };
  }, [heights]);

  const sizes = doc.pages.map((page, index) => {
    const scale = Math.min(1, printWidth / page.width);
    const height = heights?.[index] ?? page.height ?? page.width;
    return { scale, width: Math.round(page.width * scale), height: Math.round(height * scale) };
  });
  const names = [...new Set(sizes.map(({ width, height }) => `${width}x${height}`))];
  const rules = names.map((name, index) => {
    const [width, height] = name.split("x");
    return `@page ribalta-${index} { size: ${width}px ${height}px; margin: 0; }`;
  });

  return createPortal(
    <div ref={root} className="print-root" aria-hidden>
      <style>{`@page { margin: 0; }\n${rules.join("\n")}`}</style>
      {doc.pages.map((page, index) => {
        const size = sizes[index]!;
        return (
          <div
            key={`${page.id}-${index}`}
            className="print-page"
            style={{ width: size.width, height: size.height, page: `ribalta-${names.indexOf(`${size.width}x${size.height}`)}` }}
          >
            <div
              ref={(element) => {
                contents.current[index] = element;
              }}
              style={{ width: page.width, height: page.height ?? undefined, transform: `scale(${size.scale})`, transformOrigin: "top left" }}
            >
              <PageView doc={doc} page={page} />
            </div>
          </div>
        );
      })}
    </div>,
    document.body,
  );
}
