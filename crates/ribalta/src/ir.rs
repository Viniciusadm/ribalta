use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::block::ColumnAlign;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shadow {
    None,
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ImageSource {
    Asset { id: String },
    Url { url: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "unit", rename_all = "lowercase")]
pub enum Length {
    Px { value: f32 },
    Pct { value: f32 },
    Auto,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GradientStop {
    pub color: String,
    pub at: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Paint {
    Solid { color: String },
    Linear { angle: f32, stops: Vec<GradientStop> },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Border {
    pub width: f32,
    pub color: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<Paint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg_image: Option<ImageSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uppercase: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<HAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valign: Option<VAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<Border>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<Shadow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leading: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enter: Option<Enter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Enter {
    Fade,
    Up,
    Down,
    Left,
    Right,
    Zoom,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Transition {
    Fade,
    Slide,
    Zoom,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Timing {
    pub duration: u32,
    pub easing: [f64; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Pose {
    pub opacity: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dx: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dy: Option<Length>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Animation {
    #[serde(flatten)]
    pub timing: Timing,
    pub from: Pose,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Motion {
    pub enter: BTreeMap<Enter, Animation>,
    pub transitions: BTreeMap<Transition, Animation>,
    pub reveal: Timing,
    pub variant: Timing,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Scalar {
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<String>),
    Records(Vec<BTreeMap<String, Field>>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Field {
    Bool(bool),
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BinaryOp {
    #[serde(rename = "+")]
    Add,
    #[serde(rename = "-")]
    Sub,
    #[serde(rename = "*")]
    Mul,
    #[serde(rename = "/")]
    Div,
    #[serde(rename = "%")]
    Mod,
    #[serde(rename = "==")]
    Eq,
    #[serde(rename = "!=")]
    Ne,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    Le,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    Ge,
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
    #[serde(rename = "in")]
    In,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Builtin {
    Round,
    Min,
    Max,
    Len,
    Random,
    Pick,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Expr {
    Lit { value: Scalar },
    State { name: String },
    Item {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<String>,
    },
    Step,
    Elapsed,
    Hover,
    Not { arg: Box<Expr> },
    Neg { arg: Box<Expr> },
    Binary { operator: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Cond { test: Box<Expr>, then: Box<Expr>, otherwise: Box<Expr> },
    Call { name: Builtin, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    Set { name: String, value: Expr },
    Toggle { name: String },
    Add { name: String, value: Expr },
    Remove { name: String, value: Expr },
    Clear { name: String },
    Play { name: String },
    Stop { name: String },
    Go { target: String },
    Back,
    Next,
    Prev,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub when: Expr,
    pub style: Style,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Behavior {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<Expr>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<Variant>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub click: Vec<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl Behavior {
    pub fn is_empty(&self) -> bool {
        self.when.is_none() && self.variants.is_empty() && self.click.is_empty() && self.key.is_none()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub version: u32,
    pub meta: Meta,
    pub theme: ThemeIr,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub state: BTreeMap<String, Scalar>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub externs: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub icons: BTreeMap<String, Vec<IconElement>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub persist: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scorm: Option<Scorm>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub anchors: BTreeMap<String, usize>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scorm {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub complete: Vec<String>,
    pub max: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IconElement {
    pub tag: String,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub title: Option<String>,
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub page: PageSize,
    #[serde(skip_serializing_if = "Mode::is_default")]
    pub mode: Mode,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub random: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageSize {
    pub width: u32,
    pub height: Option<u32>,
}

impl PageSize {
    pub const DEFAULT: PageSize = PageSize { width: 1920, height: None };
}

impl Default for PageSize {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Default,
    Game,
}

impl Mode {
    pub fn is_default(&self) -> bool {
        *self == Mode::Default
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeIr {
    pub name: String,
    pub colors: BTreeMap<String, String>,
    pub fonts: BTreeMap<String, String>,
    pub font_families: Vec<FontFamily>,
    pub text: Style,
    pub headings: Vec<Style>,
    pub page: Style,
    pub palette: Vec<String>,
    pub motion: Motion,
    pub elements: ThemeElements,
    pub layout: LayoutDefaults,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LayoutGaps {
    pub stack: f32,
    pub row: f32,
    pub columns: f32,
    pub grid: f32,
    #[serde(rename = "box")]
    pub boxed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BandSpec {
    pub gap: f32,
    pub edge_scale: f64,
    pub height_scale: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutDefaults {
    pub gaps: LayoutGaps,
    pub grid_columns: u32,
    pub band: BandSpec,
    pub chart_max_height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShadowSpec {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shadows {
    pub sm: ShadowSpec,
    pub md: ShadowSpec,
    pub lg: ShadowSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSpec {
    pub gap_em: f32,
    pub indent_em: f32,
    pub marker_color: String,
    pub marker_weight: u16,
    pub nested_gap_em: f32,
    pub nested_scale: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSpec {
    pub gap_em: f32,
    pub indent: f32,
    pub bar_width: f32,
    pub bar_color: String,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSpec {
    pub font: String,
    pub size: f32,
    pub leading: f32,
    pub pad: [f32; 2],
    pub radius: f32,
    pub bg: String,
    pub color: String,
    pub syntax: BTreeMap<TokenKind, TokenStyle>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineCodeSpec {
    pub font: String,
    pub scale: f32,
    pub pad_em: [f32; 2],
    pub radius_em: f32,
    pub tint: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkSpec {
    pub color: String,
    pub underline_em: f32,
    pub underline_offset_em: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSpec {
    pub size: f32,
    pub cell_pad: [f32; 2],
    pub rule_width: f32,
    pub header_rule_width: f32,
    pub rule_color: String,
    pub header_weight: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DividerSpec {
    pub thickness: f32,
    pub radius: f32,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IconSpec {
    pub size: f32,
    pub stroke: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeElements {
    pub shadows: Shadows,
    pub list: ListSpec,
    pub quote: QuoteSpec,
    pub code: CodeSpec,
    pub inline_code: InlineCodeSpec,
    pub link: LinkSpec,
    pub table: TableSpec,
    pub divider: DividerSpec,
    pub icon: IconSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FontFamily {
    pub family: String,
    pub weights: Vec<u16>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    pub title: String,
    pub line: usize,
    pub end_line: usize,
    pub width: u32,
    pub height: Option<u32>,
    pub steps: u32,
    pub style: Style,
    pub notes: Option<String>,
    pub children: Vec<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay: Option<Block>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub state: BTreeMap<String, Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub on_enter: Vec<Action>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub on_timeout: Vec<Action>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub clock: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    #[serde(flatten)]
    pub kind: BlockKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub style: Style,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
    #[serde(flatten)]
    pub behavior: Behavior,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRange {
    pub line: usize,
    pub spans: Vec<[usize; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutKind {
    Stack,
    Row,
    Columns,
    Grid,
    Box,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartKind {
    Bar,
    Line,
    Area,
    Pie,
    Donut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFit {
    Cover,
    Contain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenKind {
    Plain,
    Comment,
    Keyword,
    String,
    Number,
    Constant,
    Function,
    Type,
    Operator,
    Punctuation,
    Tag,
    Attribute,
    Property,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TokenStyle {
    pub color: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BlockKind {
    Heading { level: u8, content: Vec<Inline> },
    Paragraph { content: Vec<Inline> },
    List { ordered: bool, start: u32, items: Vec<ListItem> },
    Quote { children: Vec<Block> },
    Each {
        list: Expr,
        item: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<String>,
        children: Vec<Block>,
    },
    Code { lang: Option<String>, code: String, lines: Vec<Vec<Token>> },
    Table { header: Vec<Vec<Inline>>, aligns: Vec<ColumnAlign>, rows: Vec<Vec<Vec<Inline>>> },
    Image { src: ImageSource, alt: String, fit: ImageFit },
    Layout {
        layout: LayoutKind,
        #[serde(skip_serializing_if = "Option::is_none")]
        cols: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        widths: Option<Vec<f32>>,
        wrap: bool,
        children: Vec<Block>,
    },
    Chart { chart: ChartKind, labels: Vec<String>, series: Vec<Series>, legend: bool, stacked: bool, values: bool, scene: ChartScene },
    Icon { name: String },
    Spacer { size: f32 },
    Divider,
    Sound {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        src: ImageSource,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        autoplay: bool,
        #[serde(rename = "loop", skip_serializing_if = "std::ops::Not::not")]
        repeat: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        volume: Option<f64>,
    },
    #[serde(rename_all = "camelCase")]
    Input {
        name: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        numeric: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        on_submit: Vec<Action>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Ink {
    Text,
    Background,
    Color { color: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextBaseline {
    Alphabetic,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "shape", rename_all = "lowercase")]
pub enum Shape {
    Rect { x: f64, y: f64, w: f64, h: f64, radius: f64, ink: Ink, opacity: f64 },
    Line { x1: f64, y1: f64, x2: f64, y2: f64, width: f64, ink: Ink, opacity: f64 },
    Polyline { points: Vec<[f64; 2]>, width: f64, ink: Ink, opacity: f64 },
    Polygon { points: Vec<[f64; 2]>, ink: Ink, opacity: f64 },
    Circle { cx: f64, cy: f64, r: f64, ink: Ink, opacity: f64 },
    Sector {
        cx: f64,
        cy: f64,
        outer: f64,
        inner: f64,
        start: f64,
        end: f64,
        ink: Ink,
        stroke: Ink,
        #[serde(rename = "strokeWidth")]
        stroke_width: f64,
    },
    Text { x: f64, y: f64, text: String, anchor: TextAnchor, baseline: TextBaseline, size: f64, weight: u16, ink: Ink, opacity: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChartScene {
    pub width: f64,
    pub height: f64,
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub content: Vec<Inline>,
    pub children: Vec<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub style: Style,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
    #[serde(flatten)]
    pub behavior: Behavior,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRange>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub name: String,
    pub color: String,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Inline {
    Text {
        value: String,
    },
    Strong {
        children: Vec<Inline>,
    },
    Emphasis {
        children: Vec<Inline>,
    },
    Strike {
        children: Vec<Inline>,
    },
    Code {
        value: String,
    },
    Link {
        href: String,
        children: Vec<Inline>,
    },
    Span {
        style: Style,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<u32>,
        #[serde(flatten)]
        behavior: Behavior,
        #[serde(rename = "sourceAttrs", skip_serializing_if = "String::is_empty")]
        source_attrs: String,
        children: Vec<Inline>,
    },
    Image {
        src: ImageSource,
        alt: String,
        style: Style,
        #[serde(rename = "sourceAttrs", skip_serializing_if = "String::is_empty")]
        source_attrs: String,
    },
    Field {
        name: String,
    },
    Expr {
        expr: Expr,
    },
    Break,
}
