package com.archieapps.ribalta.ir

sealed interface Scalar {
    data class Bool(val value: Boolean) : Scalar
    data class Number(val value: Double) : Scalar
    data class Text(val value: String) : Scalar
    data class List(val items: kotlin.collections.List<String>) : Scalar
    data class Records(val rows: kotlin.collections.List<Map<String, Scalar>>) : Scalar
}

enum class HAlign { Left, Center, Right }
enum class VAlign { Top, Center, Bottom }
enum class ShadowSize { None, Sm, Md, Lg }
enum class EnterKind { Fade, Up, Down, Left, Right, Zoom, None }
enum class TransitionKind { Fade, Slide, Zoom, None }
enum class LayoutKind { Stack, Row, Columns, Grid, Box }
enum class ImageFit { Cover, Contain }
enum class ColumnAlign { Left, Center, Right }

sealed interface Length {
    data class Px(val value: Float) : Length
    data class Pct(val value: Float) : Length
    data object Auto : Length
}

data class GradientStop(val color: Long, val at: Float)

sealed interface Paint {
    data class Solid(val color: Long) : Paint
    data class Linear(val angle: Float, val stops: List<GradientStop>) : Paint
}

sealed interface ImageSource {
    data class Asset(val id: String) : ImageSource
    data class Url(val url: String) : ImageSource
}

data class Border(val width: Float, val color: Long)

data class Style(
    val color: Long? = null,
    val bg: Paint? = null,
    val bgImage: ImageSource? = null,
    val font: String? = null,
    val size: Float? = null,
    val weight: Int? = null,
    val italic: Boolean? = null,
    val uppercase: Boolean? = null,
    val underline: Boolean? = null,
    val align: HAlign? = null,
    val valign: VAlign? = null,
    val pad: Pair<Float, Float>? = null,
    val gap: Float? = null,
    val radius: Float? = null,
    val border: Border? = null,
    val shadow: ShadowSize? = null,
    val w: Length? = null,
    val h: Length? = null,
    val x: Length? = null,
    val y: Length? = null,
    val opacity: Float? = null,
    val leading: Float? = null,
    val tracking: Float? = null,
    val enter: EnterKind? = null,
) {
    infix fun over(base: Style) = Style(
        color = color ?: base.color,
        bg = bg ?: base.bg,
        bgImage = bgImage ?: base.bgImage,
        font = font ?: base.font,
        size = size ?: base.size,
        weight = weight ?: base.weight,
        italic = italic ?: base.italic,
        uppercase = uppercase ?: base.uppercase,
        underline = underline ?: base.underline,
        align = align ?: base.align,
        valign = valign ?: base.valign,
        pad = pad ?: base.pad,
        gap = gap ?: base.gap,
        radius = radius ?: base.radius,
        border = border ?: base.border,
        shadow = shadow ?: base.shadow,
        w = w ?: base.w,
        h = h ?: base.h,
        x = x ?: base.x,
        y = y ?: base.y,
        opacity = opacity ?: base.opacity,
        leading = leading ?: base.leading,
        tracking = tracking ?: base.tracking,
        enter = enter ?: base.enter,
    )

    val positioned get() = x != null || y != null
}

enum class Builtin { Round, Min, Max, Len, Random, Pick }

sealed interface Expr {
    data class Lit(val value: Scalar) : Expr
    data class State(val name: String) : Expr
    data class Item(val name: String, val field: String?) : Expr
    data object Step : Expr
    data object Elapsed : Expr
    data object Hover : Expr
    data class Not(val arg: Expr) : Expr
    data class Neg(val arg: Expr) : Expr
    data class Binary(val operator: String, val left: Expr, val right: Expr) : Expr
    data class Cond(val test: Expr, val then: Expr, val otherwise: Expr) : Expr
    data class Call(val name: Builtin, val args: List<Expr>) : Expr
}

sealed interface Action {
    data class Set(val name: String, val value: Expr) : Action
    data class Toggle(val name: String) : Action
    data class Add(val name: String, val value: Expr) : Action
    data class Remove(val name: String, val value: Expr) : Action
    data class Clear(val name: String) : Action
    data class Play(val name: String) : Action
    data class Stop(val name: String) : Action
    data class Go(val target: String) : Action
    data object Back : Action
    data object Next : Action
    data object Prev : Action
}

data class Variant(val whenExpr: Expr, val style: Style)

data class Behavior(val whenExpr: Expr? = null, val variants: List<Variant> = emptyList(), val click: List<Action> = emptyList(), val key: String? = null) {
    val hoverable get() = variants.any { it.whenExpr == Expr.Hover }
}

sealed interface Inline {
    data class Text(val value: String) : Inline
    data class Strong(val children: List<Inline>) : Inline
    data class Emphasis(val children: List<Inline>) : Inline
    data class Strike(val children: List<Inline>) : Inline
    data class Code(val value: String) : Inline
    data class Link(val href: String, val children: List<Inline>) : Inline
    data class Span(val style: Style, val step: Int?, val behavior: Behavior, val children: List<Inline>) : Inline
    data class Image(val src: ImageSource, val alt: String, val style: Style) : Inline
    data class Field(val name: String) : Inline
    data class Value(val expr: Expr) : Inline
    data object Break : Inline
}

data class ListItem(val content: List<Inline>, val children: List<Block>, val style: Style, val step: Int?, val behavior: Behavior, val id: String? = null)

data class Series(val name: String, val color: Long, val values: List<Double>)

data class Token(val kind: String, val text: String)

enum class TextAnchor { Start, Middle, End }

sealed interface Ink {
    data object Text : Ink
    data object Background : Ink
    data class Color(val color: Long) : Ink
}

sealed interface Shape {
    data class Rect(val x: Float, val y: Float, val w: Float, val h: Float, val radius: Float, val ink: Ink, val opacity: Float) : Shape
    data class Line(val x1: Float, val y1: Float, val x2: Float, val y2: Float, val width: Float, val ink: Ink, val opacity: Float) : Shape
    data class Polyline(val points: List<Pair<Float, Float>>, val width: Float, val ink: Ink, val opacity: Float) : Shape
    data class Polygon(val points: List<Pair<Float, Float>>, val ink: Ink, val opacity: Float) : Shape
    data class Circle(val cx: Float, val cy: Float, val r: Float, val ink: Ink, val opacity: Float) : Shape
    data class Sector(val cx: Float, val cy: Float, val outer: Float, val inner: Float, val start: Float, val end: Float, val ink: Ink, val stroke: Ink, val strokeWidth: Float) : Shape
    data class Text(val x: Float, val y: Float, val text: String, val anchor: TextAnchor, val middle: Boolean, val size: Float, val weight: Int, val ink: Ink, val opacity: Float) : Shape
}

data class ChartScene(val width: Float, val height: Float, val shapes: List<Shape>)

sealed interface BlockKind {
    data class Heading(val level: Int, val content: List<Inline>) : BlockKind
    data class Paragraph(val content: List<Inline>) : BlockKind
    data class BulletList(val ordered: Boolean, val start: Int, val items: List<ListItem>) : BlockKind
    data class Quote(val children: List<Block>) : BlockKind
    data class Each(val list: Expr, val item: String, val index: String?, val children: List<Block>) : BlockKind
    data class Code(val lines: List<List<Token>>) : BlockKind
    data class Table(val header: List<List<Inline>>, val aligns: List<ColumnAlign>, val rows: List<List<List<Inline>>>) : BlockKind
    data class Image(val src: ImageSource, val alt: String, val fit: ImageFit) : BlockKind
    data class Layout(val layout: LayoutKind, val cols: Int?, val widths: List<Float>?, val wrap: Boolean, val children: List<Block>) : BlockKind
    data class Chart(val scene: ChartScene) : BlockKind
    data class Icon(val name: String) : BlockKind
    data class Spacer(val size: Float) : BlockKind
    data object Divider : BlockKind
    data class Input(val name: String, val numeric: Boolean, val placeholder: String?, val onSubmit: List<Action>) : BlockKind
    data class Sound(val name: String?, val src: ImageSource, val autoplay: Boolean, val loop: Boolean, val volume: Float?) : BlockKind
}

data class Block(val kind: BlockKind, val style: Style, val step: Int?, val behavior: Behavior, val id: String? = null)

data class Page(
    val id: String,
    val width: Float,
    val height: Float?,
    val steps: Int,
    val style: Style,
    val children: List<Block>,
    val header: Block?,
    val footer: Block?,
    val overlay: Block?,
    val hidden: Boolean,
    val next: String?,
    val state: Map<String, Scalar>,
    val transition: TransitionKind?,
    val onEnter: List<Action> = emptyList(),
    val locked: Boolean = false,
    val timeout: Double? = null,
    val onTimeout: List<Action> = emptyList(),
    val clock: Boolean = false,
)

data class Timing(val duration: Int, val easing: List<Float>)
data class Pose(val opacity: Float, val dx: Length?, val dy: Length?, val scale: Float?)
data class Animation(val timing: Timing, val from: Pose)
data class Motion(val enter: Map<EnterKind, Animation>, val transitions: Map<TransitionKind, Animation>, val reveal: Timing, val variant: Timing)

data class ShadowSpec(val x: Float, val y: Float, val blur: Float, val color: Long)
data class ListSpec(val gapEm: Float, val indentEm: Float, val markerColor: Long, val markerWeight: Int, val nestedGapEm: Float, val nestedScale: Float)
data class QuoteSpec(val gapEm: Float, val indent: Float, val barWidth: Float, val barColor: Long, val italic: Boolean)
data class TokenStyle(val color: Long, val italic: Boolean)
data class CodeSpec(val font: String, val size: Float, val leading: Float, val pad: Pair<Float, Float>, val radius: Float, val bg: Long, val color: Long, val syntax: Map<String, TokenStyle>)
data class InlineCodeSpec(val font: String, val scale: Float, val padEm: Pair<Float, Float>, val radiusEm: Float, val tint: Float)
data class LinkSpec(val color: Long, val underlineEm: Float, val underlineOffsetEm: Float)
data class TableSpec(val size: Float, val cellPad: Pair<Float, Float>, val ruleWidth: Float, val headerRuleWidth: Float, val ruleColor: Long, val headerWeight: Int)
data class DividerSpec(val thickness: Float, val radius: Float, val color: Long)
data class IconSpec(val size: Float, val stroke: Float)

data class Elements(
    val shadows: Map<ShadowSize, ShadowSpec>,
    val list: ListSpec,
    val quote: QuoteSpec,
    val code: CodeSpec,
    val inlineCode: InlineCodeSpec,
    val link: LinkSpec,
    val table: TableSpec,
    val divider: DividerSpec,
    val icon: IconSpec,
)

data class Band(val gap: Float, val edgeScale: Float, val heightScale: Float)
data class LayoutDefaults(val gaps: Map<LayoutKind, Float>, val gridColumns: Int, val band: Band, val chartMaxHeight: Float)

data class FontFamilySpec(val family: String, val weights: List<Int>)

data class Theme(
    val colors: Map<String, Long>,
    val fontFamilies: List<FontFamilySpec>,
    val text: Style,
    val headings: List<Style>,
    val page: Style,
    val motion: Motion,
    val elements: Elements,
    val layout: LayoutDefaults,
)

data class IconElement(val tag: String, val attrs: Map<String, String>)

data class Meta(val title: String?, val game: Boolean = false, val random: Boolean = false)

data class Document(
    val meta: Meta,
    val theme: Theme,
    val state: Map<String, Scalar>,
    val icons: Map<String, List<IconElement>>,
    val pages: List<Page>,
    val persist: List<String> = emptyList(),
    val anchors: Map<String, Int> = emptyMap(),
)

data class Diagnostic(val severity: String, val code: String, val line: Int, val message: String)

data class Compilation(val document: Document, val diagnostics: List<Diagnostic>)
