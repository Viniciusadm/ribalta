export type Locale = "pt" | "en";
export type HAlign = "left" | "center" | "right";
export type VAlign = "top" | "center" | "bottom";
export type Shadow = "none" | "sm" | "md" | "lg";
export type Enter = "fade" | "up" | "down" | "left" | "right" | "zoom" | "none";
export type Transition = "fade" | "slide" | "zoom" | "none";
export type Field = boolean | number | string;
export type Row = Record<string, Field>;
export type Scalar = boolean | number | string | string[] | Row[];
export type Builtin = "round" | "min" | "max" | "len" | "random" | "pick";
export type ImageSource = { kind: "asset"; id: string } | { kind: "url"; url: string };
export type Length = { unit: "px"; value: number } | { unit: "pct"; value: number } | { unit: "auto" };
export type Paint = { kind: "solid"; color: string } | { kind: "linear"; angle: number; stops: { color: string; at: number }[] };
export interface FontFamily {
  family: string;
  weights: number[];
}
export interface Timing {
  duration: number;
  easing: [number, number, number, number];
}
export interface Pose {
  opacity: number;
  dx?: Length;
  dy?: Length;
  scale?: number;
}
export interface Animation extends Timing {
  from: Pose;
}
export interface Motion {
  enter: Partial<Record<Enter, Animation>>;
  transitions: Partial<Record<Transition, Animation>>;
  reveal: Timing;
  variant: Timing;
}
export interface ShadowSpec {
  x: number;
  y: number;
  blur: number;
  color: string;
}
export interface ThemeElements {
  shadows: Record<Exclude<Shadow, "none">, ShadowSpec>;
  list: { gapEm: number; indentEm: number; markerColor: string; markerWeight: number; nestedGapEm: number; nestedScale: number };
  quote: { gapEm: number; indent: number; barWidth: number; barColor: string; italic: boolean };
  code: { font: string; size: number; leading: number; pad: [number, number]; radius: number; bg: string; color: string; syntax: Partial<Record<TokenKind, { color: string; italic?: boolean }>> };
  inlineCode: { font: string; scale: number; padEm: [number, number]; radiusEm: number; tint: number };
  link: { color: string; underlineEm: number; underlineOffsetEm: number };
  table: { size: number; cellPad: [number, number]; ruleWidth: number; headerRuleWidth: number; ruleColor: string; headerWeight: number };
  divider: { thickness: number; radius: number; color: string };
  icon: { size: number; stroke: number };
}
export type TokenKind = "plain" | "comment" | "keyword" | "string" | "number" | "constant" | "function" | "type" | "operator" | "punctuation" | "tag" | "attribute" | "property" | "variable";
export interface Token {
  kind: TokenKind;
  text: string;
}
export type Ink = { kind: "text" } | { kind: "background" } | { kind: "color"; color: string };
export type Shape =
  | { shape: "rect"; x: number; y: number; w: number; h: number; radius: number; ink: Ink; opacity: number }
  | { shape: "line"; x1: number; y1: number; x2: number; y2: number; width: number; ink: Ink; opacity: number }
  | { shape: "polyline"; points: [number, number][]; width: number; ink: Ink; opacity: number }
  | { shape: "polygon"; points: [number, number][]; ink: Ink; opacity: number }
  | { shape: "circle"; cx: number; cy: number; r: number; ink: Ink; opacity: number }
  | { shape: "sector"; cx: number; cy: number; outer: number; inner: number; start: number; end: number; ink: Ink; stroke: Ink; strokeWidth: number }
  | { shape: "text"; x: number; y: number; text: string; anchor: "start" | "middle" | "end"; baseline: "alphabetic" | "middle"; size: number; weight: number; ink: Ink; opacity: number };
export interface ChartScene {
  width: number;
  height: number;
  shapes: Shape[];
}
export interface IconElement {
  tag: string;
  attrs: Record<string, string>;
}
export type BinaryOperator = "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "and" | "or" | "in";

export type Expr =
  | { op: "lit"; value: Scalar }
  | { op: "state"; name: string }
  | { op: "item"; name: string; field?: string }
  | { op: "step" }
  | { op: "elapsed" }
  | { op: "hover" }
  | { op: "not"; arg: Expr }
  | { op: "neg"; arg: Expr }
  | { op: "binary"; operator: BinaryOperator; left: Expr; right: Expr }
  | { op: "cond"; test: Expr; then: Expr; otherwise: Expr }
  | { op: "call"; name: Builtin; args: Expr[] };

export type Action =
  | { type: "set"; name: string; value: Expr }
  | { type: "toggle"; name: string }
  | { type: "add"; name: string; value: Expr }
  | { type: "remove"; name: string; value: Expr }
  | { type: "clear"; name: string }
  | { type: "play"; name: string }
  | { type: "stop"; name: string }
  | { type: "go"; target: string }
  | { type: "back" }
  | { type: "next" }
  | { type: "prev" };

export interface Variant {
  when: Expr;
  style: Style;
}

export interface Behavior {
  when?: Expr;
  variants?: Variant[];
  click?: Action[];
  key?: string;
}

export interface Style {
  color?: string;
  bg?: Paint;
  bgImage?: ImageSource;
  font?: string;
  size?: number;
  weight?: number;
  italic?: boolean;
  uppercase?: boolean;
  underline?: boolean;
  align?: HAlign;
  valign?: VAlign;
  pad?: [number, number];
  gap?: number;
  radius?: number;
  border?: { width: number; color: string };
  shadow?: Shadow;
  w?: Length;
  h?: Length;
  x?: Length;
  y?: Length;
  opacity?: number;
  leading?: number;
  tracking?: number;
  enter?: Enter;
}

export type Inline =
  | { type: "text"; value: string }
  | { type: "strong"; children: Inline[] }
  | { type: "emphasis"; children: Inline[] }
  | { type: "strike"; children: Inline[] }
  | { type: "code"; value: string }
  | { type: "link"; href: string; children: Inline[] }
  | ({ type: "span"; style: Style; step?: number; sourceAttrs?: string; children: Inline[] } & Behavior)
  | { type: "image"; src: ImageSource; alt: string; style: Style; sourceAttrs?: string }
  | { type: "field"; name: "date" }
  | { type: "expr"; expr: Expr }
  | { type: "break" };

export interface SourceRange {
  line: number;
  spans: [number, number][];
}

export interface ListItem extends Behavior {
  content: Inline[];
  children: Block[];
  id?: string;
  style: Style;
  step?: number;
  line: number;
  source?: SourceRange;
}

export interface Series {
  name: string;
  color: string;
  values: number[];
}

export type LayoutKind = "stack" | "row" | "columns" | "grid" | "box";
export type ChartKind = "bar" | "line" | "area" | "pie" | "donut";
export type ColumnAlign = "left" | "center" | "right";

export type BlockKind =
  | { type: "heading"; level: number; content: Inline[] }
  | { type: "paragraph"; content: Inline[] }
  | { type: "list"; ordered: boolean; start: number; items: ListItem[] }
  | { type: "quote"; children: Block[] }
  | { type: "each"; list: Expr; item: string; index?: string; children: Block[] }
  | { type: "code"; lang: string | null; code: string; lines: Token[][] }
  | { type: "table"; header: Inline[][]; aligns: ColumnAlign[]; rows: Inline[][][] }
  | { type: "image"; src: ImageSource; alt: string; fit: "cover" | "contain" }
  | { type: "layout"; layout: LayoutKind; cols?: number; widths?: number[]; wrap: boolean; children: Block[] }
  | { type: "chart"; chart: ChartKind; labels: string[]; series: Series[]; legend: boolean; stacked: boolean; values: boolean; scene: ChartScene }
  | { type: "icon"; name: string }
  | { type: "spacer"; size: number }
  | { type: "divider" }
  | { type: "input"; name: string; numeric?: boolean; placeholder?: string; onSubmit?: Action[] }
  | { type: "sound"; name?: string; src: ImageSource; autoplay?: boolean; loop?: boolean; volume?: number };

export type Block = BlockKind & Behavior & { id?: string; style: Style; step?: number; line: number; endLine?: number; source?: SourceRange };

export interface Scorm {
  score?: string;
  complete?: string[];
  max: number;
  pass?: number;
}

export interface PageSize {
  width: number;
  height: number | null;
}

export interface Page {
  id: string;
  title: string;
  line: number;
  endLine: number;
  width: number;
  height: number | null;
  steps: number;
  style: Style;
  notes: string | null;
  children: Block[];
  header?: Block;
  footer?: Block;
  overlay?: Block;
  hidden?: boolean;
  next?: string;
  state?: Record<string, Scalar>;
  transition?: Transition;
  onEnter?: Action[];
  locked?: boolean;
  timeout?: number;
  onTimeout?: Action[];
  clock?: boolean;
}

export interface Theme {
  name: string;
  colors: Record<string, string>;
  fonts: Record<string, string>;
  fontFamilies: FontFamily[];
  text: Style;
  headings: Style[];
  page: Style;
  palette: string[];
  motion: Motion;
  elements: ThemeElements;
  layout: LayoutDefaults;
}

export interface LayoutDefaults {
  gaps: Record<LayoutKind, number>;
  gridColumns: number;
  band: { gap: number; edgeScale: number; heightScale: number };
  chartMaxHeight: number;
}

export interface Document {
  version: number;
  meta: { title: string | null; author: string | null; page: PageSize; mode?: "game"; random?: boolean };
  theme: Theme;
  state?: Record<string, Scalar>;
  externs?: string[];
  icons?: Record<string, IconElement[]>;
  persist?: string[];
  scorm?: Scorm;
  anchors?: Record<string, number>;
  pages: Page[];
}

export interface Diagnostic {
  severity: "error" | "warning";
  code: string;
  message: string;
  line: number;
  column: number;
  endColumn: number;
  hint?: string;
}

export interface Compilation {
  document: Document;
  diagnostics: Diagnostic[];
}

export type DefKind = "setting" | "variable" | "color" | "font" | "style" | "component" | "header" | "footer" | "overlay";

export interface DefRef {
  kind: DefKind;
  name: string;
}

export interface Param {
  name: string;
  default: string | null;
}

export interface Definition {
  kind: DefKind;
  name: string;
  startLine: number;
  endLine: number;
  deps: DefRef[];
  params: Param[];
  slots: string[];
  usesContent: boolean;
}

export interface Extraction {
  source: string;
  included: DefRef[];
  diagnostics: Diagnostic[];
}

export interface Conflict {
  kind: DefKind;
  name: string;
  identical: boolean;
}

export type Resolution =
  | { action: "replace"; kind: DefKind; name: string }
  | { action: "rename"; kind: DefKind; name: string; to: string }
  | { action: "keep"; kind: DefKind; name: string };

export interface Renamed {
  kind: DefKind;
  from: string;
  to: string;
}

export type MergeResult =
  | { ok: true; source: string; renamed: Renamed[]; skipped: DefRef[] }
  | { ok: false; conflicts: Conflict[]; error?: string };

export type ItemKind = "theme" | "component" | "style" | "palette";

export interface CompileOptions {
  page?: { width: number; height?: number | null };
}
