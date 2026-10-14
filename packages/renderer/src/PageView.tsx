import { memo, useEffect, useMemo, useRef, useState, type CSSProperties, type ReactNode } from "react";
import type { Action, Document, ImageSource, Page, Paint, Scalar } from "@ribaltalang/compiler";
import { useResolveSource } from "./assets";
import { Blocks, IconsContext, ThemeContext } from "./Blocks";
import { scrollToAnchor } from "./anchors";
import { columnFlow, elementVars, solidColor, visualCss } from "./css";
import { InteractionContext } from "./interaction";
import { MotionStyles } from "./motion";
import { StepContext } from "./step";

function layeredBackground(bg: Paint | undefined, image: ImageSource | undefined, base: string | undefined): CSSProperties {
  if (bg?.kind !== "solid" || !base || image) return {};
  return { backgroundColor: base, backgroundImage: `linear-gradient(${bg.color}, ${bg.color})` };
}

const noIcons = {};

export interface PageViewProps {
  doc: Document;
  page: Page;
  step?: number;
  state?: Record<string, Scalar>;
  elapsed?: number;
  onAction?: (actions: Action[]) => void;
  anchor?: string;
  anchorToken?: number;
  scrollTo?: (id: string) => boolean;
}

export const PageView = memo(function PageView({ doc, page, step = Number.POSITIVE_INFINITY, state, elapsed = 0, onAction, anchor, anchorToken, scrollTo }: PageViewProps) {
  const root = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (anchor && root.current) scrollToAnchor(root.current, anchor);
  }, [anchor, anchorToken, page]);
  const { theme } = doc;
  const fixed = page.height !== null;
  const resolve = useResolveSource();
  const interaction = useMemo(
    () => ({
      scope: { state: { ...doc.state, ...page.state, ...state }, step: Number.isFinite(step) ? step : page.steps, elapsed },
      run: onAction ?? null,
      scrollTo,
    }),
    [doc.state, page.state, page.steps, state, step, elapsed, onAction, scrollTo],
  );
  const merged = { ...theme.page, ...page.style };
  const pad = merged.pad ?? [0, 0];
  const gap = merged.gap ?? 0;
  const banded = Boolean(page.header || page.footer);
  const { band } = theme.layout;
  const bandGap = band.gap;
  const bandHeight = Math.max(0, pad[0] * band.heightScale - bandGap);
  const flow: CSSProperties = { ...columnFlow(merged.align, merged.valign), gap };
  const { padding: _pagePadding, gap: _pageGap, ...visual } = visualCss(merged, resolve);
  const style: CSSProperties = {
    ...visualCss(theme.text, resolve),
    ...visual,
    ...(banded
      ? {
          display: "grid",
          gridTemplateRows: fixed ? "auto minmax(0, 1fr) auto" : "auto 1fr auto",
          rowGap: bandGap,
          columnGap: 0,
          paddingTop: page.header ? pad[0] * band.edgeScale : pad[0],
          paddingBottom: page.footer ? pad[0] * band.edgeScale : pad[0],
        }
      : { ...columnFlow(merged.align, merged.valign), rowGap: gap, columnGap: gap, paddingTop: pad[0], paddingBottom: pad[0] }),
    paddingLeft: pad[1],
    paddingRight: pad[1],
    ...layeredBackground(page.style.bg, page.style.bgImage, solidColor(theme.page.bg) ?? theme.colors["bg"]),
    position: "relative",
    boxSizing: "border-box",
    width: page.width,
    ...(fixed ? { height: page.height!, overflow: "hidden" } : { minHeight: pad[0] * 2 }),
    ...elementVars(theme.elements),
    ["--ribalta-bg" as string]: solidColor(merged.bg)?.startsWith("#") ? solidColor(merged.bg) : theme.colors["bg"],
  };
  return (
    <ThemeContext.Provider value={theme}>
      <IconsContext.Provider value={doc.icons ?? noIcons}>
        <InteractionContext.Provider value={interaction}>
          <StepContext.Provider value={step}>
            <MotionStyles motion={theme.motion} />
            <div ref={root} className="ribalta-page" data-page={page.id} style={style}>
              {banded ? (
                <>
                  <div className="ribalta-band ribalta-header" style={{ minHeight: page.header ? bandHeight : 0 }}>
                    {page.header && <Blocks blocks={[page.header]} />}
                  </div>
                  <div className="ribalta-content" style={{ ...flow, minHeight: 0 }}>
                    <Blocks blocks={page.children} />
                  </div>
                  <div className="ribalta-band ribalta-footer" style={{ minHeight: page.footer ? bandHeight : 0 }}>
                    {page.footer && <Blocks blocks={[page.footer]} />}
                  </div>
                </>
              ) : (
                <Blocks blocks={page.children} />
              )}
              {page.overlay && (
                <div className="ribalta-overlay" style={{ padding: `${pad[0]}px ${pad[1]}px` }}>
                  <Blocks blocks={[page.overlay]} />
                </div>
              )}
            </div>
          </StepContext.Provider>
        </InteractionContext.Provider>
      </IconsContext.Provider>
    </ThemeContext.Provider>
  );
});

export type PageFit = "contain" | "width";

export function ScaledPage({
  width,
  height,
  fit = height === null ? "width" : "contain",
  children,
  className,
}: {
  width: number;
  height: number | null;
  fit?: PageFit;
  children: ReactNode;
  className?: string;
}) {
  const outer = useRef<HTMLDivElement>(null);
  const inner = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(0);
  const [natural, setNatural] = useState(height ?? 0);

  useEffect(() => {
    const element = outer.current;
    const content = inner.current;
    if (!element || !content) return;
    const update = () => {
      const bounds = element.getBoundingClientRect();
      setNatural(height ?? content.offsetHeight);
      setScale(fit === "width" || height === null ? bounds.width / width : Math.min(bounds.width / width, bounds.height / height));
    };
    update();
    const observer = new ResizeObserver(update);
    observer.observe(element);
    observer.observe(content);
    return () => observer.disconnect();
  }, [width, height, fit]);

  const visibility = scale ? "visible" : "hidden";
  if (fit === "width") {
    return (
      <div ref={outer} className={className} style={{ position: "relative", width: "100%", height: natural * scale, overflow: "hidden" }}>
        <div
          ref={inner}
          style={{ position: "absolute", left: 0, top: 0, width, height: height ?? undefined, transform: `scale(${scale})`, transformOrigin: "top left", visibility }}
        >
          {children}
        </div>
      </div>
    );
  }

  return (
    <div ref={outer} className={className} style={{ position: "relative", width: "100%", height: "100%", overflow: "hidden" }}>
      <div
        ref={inner}
        style={{
          position: "absolute",
          left: "50%",
          top: "50%",
          width,
          height: natural,
          marginLeft: -width / 2,
          marginTop: -natural / 2,
          transform: `scale(${scale})`,
          visibility,
        }}
      >
        {children}
      </div>
    </div>
  );
}
