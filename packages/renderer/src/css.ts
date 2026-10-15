import type { CSSProperties } from "react";
import type { HAlign, LayoutDefaults, LayoutKind, Length, Paint, ShadowSpec, Style, ThemeElements, VAlign } from "@ribaltalang/compiler";
import type { SourceResolver } from "./assets";
import { localFontFace } from "./bundledFonts";
import { shrinkFontSize } from "./shrink";


const start = { left: "flex-start", center: "center", right: "flex-end" } as const;
const vertical = { top: "flex-start", center: "center", bottom: "flex-end" } as const;

export function fontStack(family: string | undefined) {
  return family ? `"${localFontFace(family) ?? family}", system-ui, sans-serif` : undefined;
}

export function lengthCss(length: Length) {
  if (length.unit === "px") return length.value;
  if (length.unit === "pct") return `${length.value}%`;
  return "auto";
}

export function gradientCss(paint: Extract<Paint, { kind: "linear" }>) {
  const stops = paint.stops.map((stop) => `${stop.color} ${stop.at * 100}%`).join(", ");
  return `linear-gradient(${paint.angle}deg, ${stops})`;
}

export function solidColor(paint: Paint | undefined) {
  return paint?.kind === "solid" ? paint.color : undefined;
}

function shadowCss({ x, y, blur, color }: ShadowSpec) {
  return `${x}px ${y}px ${blur}px ${color}`;
}

export function elementVars({ shadows, list, quote, code, inlineCode, link, table, divider }: ThemeElements): Record<string, string> {
  return {
    "--ribalta-shadow-sm": shadowCss(shadows.sm),
    "--ribalta-shadow-md": shadowCss(shadows.md),
    "--ribalta-shadow-lg": shadowCss(shadows.lg),
    "--ribalta-list-gap": `${list.gapEm}em`,
    "--ribalta-list-indent": `${list.indentEm}em`,
    "--ribalta-list-marker": list.markerColor,
    "--ribalta-list-marker-weight": String(list.markerWeight),
    "--ribalta-list-nested-gap": `${list.nestedGapEm}em`,
    "--ribalta-list-nested-size": `${list.nestedScale}em`,
    "--ribalta-quote-gap": `${quote.gapEm}em`,
    "--ribalta-quote-indent": `${quote.indent}px`,
    "--ribalta-quote-bar": `${quote.barWidth}px solid ${quote.barColor}`,
    "--ribalta-quote-style": quote.italic ? "italic" : "normal",
    "--ribalta-code-font": fontStack(code.font)!,
    "--ribalta-code-size": `${code.size}px`,
    "--ribalta-code-leading": String(code.leading),
    "--ribalta-code-pad": `${code.pad[0]}px ${code.pad[1]}px`,
    "--ribalta-code-radius": `${code.radius}px`,
    "--ribalta-code-bg": code.bg,
    "--ribalta-code-color": code.color,
    "--ribalta-inline-code-font": fontStack(inlineCode.font)!,
    "--ribalta-inline-code-size": `${inlineCode.scale}em`,
    "--ribalta-inline-code-pad": `${inlineCode.padEm[0]}em ${inlineCode.padEm[1]}em`,
    "--ribalta-inline-code-radius": `${inlineCode.radiusEm}em`,
    "--ribalta-inline-code-tint": `${inlineCode.tint * 100}%`,
    "--ribalta-link": link.color,
    "--ribalta-link-underline": `${link.underlineEm}em`,
    "--ribalta-link-offset": `${link.underlineOffsetEm}em`,
    "--ribalta-table-size": `${table.size}px`,
    "--ribalta-table-pad": `${table.cellPad[0]}px ${table.cellPad[1]}px`,
    "--ribalta-table-rule": `${table.ruleWidth}px solid ${table.ruleColor}`,
    "--ribalta-table-header-rule": `${table.headerRuleWidth}px`,
    "--ribalta-table-header-weight": String(table.headerWeight),
    "--ribalta-divider-thickness": `${divider.thickness}px`,
    "--ribalta-divider-radius": `${divider.radius}px`,
    "--ribalta-divider-color": divider.color,
  };
}

export function visualCss(style: Style, resolve: SourceResolver): CSSProperties {
  const css: CSSProperties = {};
  if (style.color) css.color = style.color;
  const images: string[] = [];
  if (style.bg?.kind === "linear") images.push(gradientCss(style.bg));
  else if (style.bg) css.backgroundColor = style.bg.color;
  if (style.bgImage) {
    images.unshift(`url("${resolve(style.bgImage)}")`);
    css.backgroundSize = "cover";
    css.backgroundPosition = "center";
  }
  if (images.length) css.backgroundImage = images.join(", ");
  if (style.font) css.fontFamily = fontStack(style.font);
  if (style.size !== undefined) css.fontSize = style.size;
  if (style.shrink !== undefined) {
    css.fontSize = shrinkFontSize(style.size, style.shrink);
    css.flexShrink = 0;
  }
  if (style.weight !== undefined) css.fontWeight = style.weight;
  if (style.italic !== undefined) css.fontStyle = style.italic ? "italic" : "normal";
  if (style.uppercase !== undefined) css.textTransform = style.uppercase ? "uppercase" : "none";
  if (style.underline !== undefined) css.textDecorationLine = style.underline ? "underline" : "none";
  if (style.align) css.textAlign = style.align;
  if (style.pad) css.padding = `${style.pad[0]}px ${style.pad[1]}px`;
  if (style.gap !== undefined) css.gap = style.gap;
  if (style.radius !== undefined) css.borderRadius = style.radius;
  if (style.border) css.border = `${style.border.width}px solid ${style.border.color}`;
  if (style.shadow) css.boxShadow = style.shadow === "none" ? "none" : `var(--ribalta-shadow-${style.shadow})`;
  if (style.w) css.width = lengthCss(style.w);
  if (style.h) css.height = lengthCss(style.h);
  if (style.x !== undefined || style.y !== undefined) {
    css.position = "absolute";
    css.left = style.x ? lengthCss(style.x) : 0;
    css.top = style.y ? lengthCss(style.y) : 0;
  }
  if (style.opacity !== undefined) css.opacity = style.opacity;
  if (style.leading !== undefined) css.lineHeight = style.leading;
  if (style.tracking !== undefined) css.letterSpacing = `${style.tracking}em`;
  return css;
}

export function columnFlow(align: HAlign | undefined, valign: VAlign | undefined): CSSProperties {
  return {
    display: "flex",
    flexDirection: "column",
    alignItems: align ? start[align] : "stretch",
    justifyContent: valign ? vertical[valign] : "flex-start",
  };
}

export function layoutCss(kind: LayoutKind, style: Style, cols: number | undefined, wrap: boolean, defaults: LayoutDefaults): CSSProperties {
  const gap = style.gap ?? defaults.gaps[kind];
  switch (kind) {
    case "row":
      return {
        display: "flex",
        flexDirection: "row",
        flexWrap: wrap ? "wrap" : "nowrap",
        justifyContent: style.align ? start[style.align] : "flex-start",
        alignItems: style.valign ? vertical[style.valign] : "center",
        gap,
      };
    case "columns":
      return {
        display: "flex",
        flexDirection: "row",
        alignItems: style.valign ? vertical[style.valign] : "center",
        alignSelf: "stretch",
        gap,
      };
    case "grid":
      return {
        display: "grid",
        gridTemplateColumns: `repeat(${cols ?? defaults.gridColumns}, minmax(0, 1fr))`,
        alignItems: style.valign ? vertical[style.valign] : "stretch",
        alignSelf: "stretch",
        gap,
      };
    case "box":
      return { ...columnFlow(style.align, style.valign), gap };
    case "stack":
      return { ...columnFlow(style.align, style.valign), alignSelf: "stretch", gap };
  }
}
