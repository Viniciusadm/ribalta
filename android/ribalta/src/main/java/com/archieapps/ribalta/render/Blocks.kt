package com.archieapps.ribalta.render

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.border
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.layoutId
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import com.archieapps.ribalta.ir.Action
import com.archieapps.ribalta.ir.Behavior
import com.archieapps.ribalta.ir.Block
import com.archieapps.ribalta.ir.Expr
import com.archieapps.ribalta.ir.Scalar
import com.archieapps.ribalta.runtime.Evaluator
import com.archieapps.ribalta.runtime.Scope
import com.archieapps.ribalta.ir.BlockKind
import com.archieapps.ribalta.ir.HAlign
import com.archieapps.ribalta.ir.ImageFit
import com.archieapps.ribalta.ir.ImageSource
import com.archieapps.ribalta.ir.Inline
import com.archieapps.ribalta.ir.Length
import com.archieapps.ribalta.ir.LayoutKind
import com.archieapps.ribalta.ir.ListItem
import com.archieapps.ribalta.ir.Paint
import com.archieapps.ribalta.ir.Style
import kotlin.math.roundToInt

private const val SYMBOL_MARKER_PADDING = 7f

enum class Axis { Column, Row }

val LocalAxis = compositionLocalOf { Axis.Column }

@Composable
fun Blocks(blocks: List<Block>, env: TextEnv) {
    for (block in blocks) BlockView(block, env)
}

@Composable
fun Paragraph(nodes: List<Inline>, env: TextEnv, modifier: Modifier = Modifier, heading: Boolean = false, fill: Boolean = false) {
    val render = LocalRender.current
    val interaction = LocalInteraction.current
    val measurer = rememberTextMeasurer()
    val density = LocalDensity.current.density
    val sources = remember(nodes) { imageSources(nodes) }
    val images by produceState(emptyMap<ImageSource, ImageBitmap?>(), sources) { value = sources.associateWith { render.images.load(it) } }
    val inlined = InlineBuilder(render, interaction, measurer, images, density).build(nodes, env)
    val style = render.textStyle(env).copy(lineBreak = if (heading) LineBreak.Heading else LineBreak.Simple)
    CssText(inlined.text, style, env.size * env.leading, if (fill) modifier.fillMaxWidth() else modifier, inlined.content)
}

private fun Modifier.onClick(actions: List<com.archieapps.ribalta.ir.Action>, interaction: Interaction, source: MutableInteractionSource, hoverable: Boolean): Modifier = when {
    actions.isNotEmpty() -> clickable(source, null) { interaction.run(actions) }
    hoverable -> hoverable(source)
    else -> this
}

@Composable
fun rememberHover(behavior: Behavior): Pair<MutableInteractionSource, Boolean> {
    val source = remember { MutableInteractionSource() }
    val hovered by source.collectIsHoveredAsState()
    val focused by source.collectIsFocusedAsState()
    return source to (behavior.hoverable && (hovered || focused))
}

private fun repeatedScopes(each: BlockKind.Each, scope: Scope): List<Scope> {
    fun positioned(items: Map<String, Scalar>, position: Int) = if (each.index == null) items else items + (each.index to Scalar.Number(position + 1.0))
    return when (val list = Evaluator.evaluate(each.list, scope)) {
        is Scalar.Records -> list.rows.mapIndexed { position, row -> scope.copy(items = positioned(scope.items, position), rows = scope.rows + (each.item to row)) }
        is Scalar.List -> list.items.mapIndexed { position, item -> scope.copy(items = positioned(scope.items + (each.item to Scalar.Text(item)), position)) }
        else -> emptyList()
    }
}

private data class Cell(val block: Block, val scope: Scope, val width: Float?)

private fun cells(children: List<Block>, scope: Scope, widths: List<Float>?): List<Cell> = children.flatMapIndexed { index, child ->
    when (val kind = child.kind) {
        is BlockKind.Each -> repeatedScopes(kind, scope).flatMap { inner -> cells(kind.children, inner, null) }
        else -> listOf(Cell(child, scope, widths?.getOrNull(index)))
    }
}

private fun Interaction.with(scope: Scope) = Interaction(scope, live, shortcut, run)

@Composable
private fun CellView(cell: Cell, env: TextEnv) {
    val interaction = LocalInteraction.current
    CompositionLocalProvider(LocalInteraction provides interaction.with(cell.scope)) { BlockView(cell.block, env) }
}

@Composable
fun EachView(each: BlockKind.Each, env: TextEnv) {
    val interaction = LocalInteraction.current
    for (scope in repeatedScopes(each, interaction.scope)) {
        CompositionLocalProvider(LocalInteraction provides interaction.with(scope)) { Blocks(each.children, env) }
    }
}

@Composable
fun BlockView(block: Block, env: TextEnv, absolute: Boolean = false) {
    if (block.kind is BlockKind.Each) {
        EachView(block.kind, env)
        return
    }
    val render = LocalRender.current
    val interaction = LocalInteraction.current
    val (hoverSource, hovered) = rememberHover(block.behavior)
    if (!interaction.holds(block.behavior.whenExpr)) return
    val style = interaction.withVariants(block.style, block.behavior.variants, hovered)
    if (style.positioned && !absolute) return
    val theme = render.theme
    val image by rememberImage(render.images, style.bgImage)
    val enter = style.enter?.takeIf { block.behavior.whenExpr != null && it != com.archieapps.ribalta.ir.EnterKind.None }?.let { theme.motion.enter[it] }
    val stretch = when (val kind = block.kind) {
        is BlockKind.Layout -> kind.layout == LayoutKind.Stack || kind.layout == LayoutKind.Columns || kind.layout == LayoutKind.Grid
        is BlockKind.Chart, BlockKind.Divider -> true
        else -> false
    }
    val shrinkable = block.kind is BlockKind.Layout || block.kind is BlockKind.Image
    val box = Modifier
        .anchored(block.id)
        .then(if (shrinkable) Modifier.layoutId(Shrinkable) else Modifier)
        .reveal(block.step, style.enter, theme.motion, interaction.step)
        .entrance(enter)
        .cssSize(style, null)
        .then(if (stretch && style.w == null && LocalAxis.current == Axis.Column) Modifier.fillMaxWidth() else Modifier)
    val inner = Modifier.cssBox(style, theme, image).onClick(block.behavior.click, interaction, hoverSource, block.behavior.hoverable)
    val blockEnv = env.with(style)
    when (val kind = block.kind) {
        is BlockKind.Heading -> {
            val headingEnv = env.with(theme.headings.getOrElse(kind.level - 1) { Style() }).with(style)
            Box(box.then(inner), propagateMinConstraints = true) { Paragraph(kind.content, headingEnv, heading = true) }
        }
        is BlockKind.Paragraph -> Box(box.then(inner), propagateMinConstraints = true) { Paragraph(kind.content, blockEnv) }
        is BlockKind.BulletList -> Box(box.then(inner), propagateMinConstraints = true) { ListView(kind, blockEnv, nested = false) }
        is BlockKind.Each -> EachView(kind, env)
        is BlockKind.Quote -> {
            val spec = theme.elements.quote
            val quoteEnv = blockEnv.copy(italic = style.italic ?: spec.italic)
            val bar = color(spec.barColor)
            Box(
                box.then(inner).drawBehind { drawRect(bar, Offset.Zero, Size(spec.barWidth.dp.toPx(), size.height)) }.padding(start = (spec.indent + spec.barWidth).dp),
                propagateMinConstraints = true,
            ) {
                FlowColumn(spec.gapEm * quoteEnv.size, null, null) { Blocks(kind.children, quoteEnv) }
            }
        }
        is BlockKind.Code -> CodeView(kind, style, env, box)
        is BlockKind.Table -> Box(box.then(inner), propagateMinConstraints = true) { TableView(kind, blockEnv) }
        is BlockKind.Image -> ImageView(kind, style, box.then(inner))
        is BlockKind.Layout -> LayoutView(kind, style, blockEnv, box.then(inner))
        is BlockKind.Chart -> Box(box.then(inner), propagateMinConstraints = true) { ChartView(kind.scene, style.h, theme.layout.chartMaxHeight, blockEnv, render) }
        is BlockKind.Icon -> {
            val elements = render.icons[kind.name] ?: return
            Box(box.then(inner)) { IconView(elements, style.size ?: theme.elements.icon.size, theme.elements.icon.stroke, blockEnv.color) }
        }
        is BlockKind.Spacer -> Box(if (LocalAxis.current == Axis.Row) Modifier.width(kind.size.dp) else Modifier.height(kind.size.dp))
        BlockKind.Divider -> {
            val spec = theme.elements.divider
            if (LocalAxis.current == Axis.Row) return
            Box(
                box.padding(vertical = (blockEnv.size * 0.5f).dp).height(spec.thickness.dp)
                    .background(style.bg?.let { (it as? Paint.Solid)?.color?.let(::color) } ?: color(spec.color), RoundedCornerShape(spec.radius.dp)),
            )
        }
        is BlockKind.Input -> InputView(kind, blockEnv, box.then(inner))
        is BlockKind.Sound -> Unit
    }
}

private fun reading(text: String, numeric: Boolean): Scalar =
    if (numeric) Scalar.Number(text.trim().toDoubleOrNull()?.takeUnless { it.isNaN() } ?: 0.0) else Scalar.Text(text)

@Composable
private fun InputView(kind: BlockKind.Input, env: TextEnv, modifier: Modifier) {
    val render = LocalRender.current
    val interaction = LocalInteraction.current
    val current = interaction.scope.state[kind.name]?.let(Evaluator::format) ?: ""
    var text by remember(kind.name) { mutableStateOf(current) }
    LaunchedEffect(current) {
        if (Evaluator.format(reading(text, kind.numeric)) != current) text = current
    }
    val colors = render.theme.colors
    val textStyle = render.textStyle(env)
    val submit = {
        if (kind.onSubmit.isNotEmpty()) interaction.run(kind.onSubmit) else interaction.shortcut("Enter")
        Unit
    }
    BasicTextField(
        value = text,
        onValueChange = { typed ->
            val value = typed.take(200)
            text = value
            interaction.run(listOf(Action.Set(kind.name, Expr.Lit(reading(value, kind.numeric)))))
        },
        enabled = interaction.live,
        singleLine = true,
        textStyle = textStyle,
        cursorBrush = SolidColor(env.color),
        keyboardOptions = KeyboardOptions(keyboardType = if (kind.numeric) KeyboardType.Decimal else KeyboardType.Text, imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { submit() }),
        modifier = modifier
            .widthIn(max = (18 * env.size).dp)
            .fillMaxWidth()
            .background(colors["surface"]?.let(::color) ?: Color.Transparent, RoundedCornerShape(16.dp))
            .border(3.dp, colors["line"]?.let(::color) ?: env.color, RoundedCornerShape(16.dp))
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown && (event.key == Key.Enter || event.key == Key.NumPadEnter)) {
                    submit()
                    true
                } else {
                    false
                }
            },
        decorationBox = { field ->
            Box(Modifier.padding(horizontal = (0.6f * env.size).dp, vertical = (0.3f * env.size).dp)) {
                if (text.isEmpty() && kind.placeholder != null) Text(kind.placeholder, style = textStyle.copy(color = env.color.copy(alpha = env.color.alpha * 0.5f)), maxLines = 1)
                field()
            }
        },
    )
}

@Composable
private fun LayoutView(kind: BlockKind.Layout, style: Style, env: TextEnv, modifier: Modifier) {
    val gap = style.gap ?: LocalRender.current.theme.layout.gaps[kind.layout] ?: 0f
    val children = kind.children
    when (kind.layout) {
        LayoutKind.Stack, LayoutKind.Box -> FlowColumn(gap, style.align, style.valign, modifier) {
            androidx.compose.runtime.CompositionLocalProvider(LocalAxis provides Axis.Column) { Blocks(children, env) }
        }
        LayoutKind.Row -> FlexRow(gap, style.align, style.valign, kind.wrap, modifier) {
            androidx.compose.runtime.CompositionLocalProvider(LocalAxis provides Axis.Row) { Blocks(children, env) }
        }
        LayoutKind.Columns -> {
            val interaction = LocalInteraction.current
            val visible = cells(children, interaction.scope, kind.widths).filter { cell -> interaction.with(cell.scope).let { it.holds(cell.block.behavior.whenExpr) && !it.withVariants(cell.block.style, cell.block.behavior.variants).positioned } }
            ColumnsRow(gap, visible.map { it.width ?: 1f }, style.valign, modifier) {
                androidx.compose.runtime.CompositionLocalProvider(LocalAxis provides Axis.Column) {
                    for (cell in visible) FlowColumn(0f, null, null) { CellView(cell, env) }
                }
            }
        }
        LayoutKind.Grid -> {
            val interaction = LocalInteraction.current
            val visible = cells(children, interaction.scope, null).filter { cell -> interaction.with(cell.scope).let { it.holds(cell.block.behavior.whenExpr) && !it.withVariants(cell.block.style, cell.block.behavior.variants).positioned } }
            GridLayout(kind.cols ?: LocalRender.current.theme.layout.gridColumns, gap, style.valign, visible.size, modifier) { index ->
                androidx.compose.runtime.CompositionLocalProvider(LocalAxis provides Axis.Column) { visible.getOrNull(index)?.let { CellView(it, env) } }
            }
        }
    }
}

@Composable
private fun ListView(list: BlockKind.BulletList, env: TextEnv, nested: Boolean) {
    val render = LocalRender.current
    val interaction = LocalInteraction.current
    val spec = render.theme.elements.list
    val listEnv = if (nested) env.copy(size = env.size * spec.nestedScale) else env
    val top = if (nested) spec.nestedGapEm * listEnv.size else 0f
    FlowColumn(spec.gapEm * listEnv.size, null, null, Modifier.padding(start = (spec.indentEm * listEnv.size).dp, top = top.dp)) {
        list.items.forEachIndexed { index, item ->
            if (!interaction.holds(item.behavior.whenExpr)) return@forEachIndexed
            val marker = if (list.ordered) "${list.start + index}. " else "• "
            ListItemView(item, listEnv, marker, if (list.ordered) 0f else SYMBOL_MARKER_PADDING)
        }
    }
}

@Composable
private fun ListItemView(item: ListItem, env: TextEnv, marker: String, markerPadding: Float) {
    val render = LocalRender.current
    val interaction = LocalInteraction.current
    val spec = render.theme.elements.list
    val (hoverSource, hovered) = rememberHover(item.behavior)
    val style = interaction.withVariants(item.style, item.behavior.variants, hovered)
    val itemEnv = env.with(style)
    val markerStyle = render.textStyle(itemEnv.copy(color = color(spec.markerColor), weight = spec.markerWeight, italic = false, underline = false))
    Layout(
        content = {
            CssText(AnnotatedString(marker), markerStyle, itemEnv.size * itemEnv.leading, softWrap = false, maxLines = 1)
            Column(Modifier.cssBox(style, render.theme, null).onClick(item.behavior.click, interaction, hoverSource, item.behavior.hoverable)) {
                if (item.content.isNotEmpty()) Paragraph(item.content, itemEnv)
                for (child in item.children) {
                    val kind = child.kind
                    if (kind is BlockKind.BulletList) ListView(kind, itemEnv.with(child.style), nested = true) else BlockView(child, itemEnv)
                }
            }
        },
        modifier = Modifier.anchored(item.id).reveal(item.step, style.enter, render.theme.motion, interaction.step),
    ) { measurables, constraints ->
        val markerPlaceable = measurables[0].measure(Constraints())
        val body = measurables[1].measure(constraints.copy(minHeight = 0))
        layout(body.width, body.height) {
            markerPlaceable.place(-markerPlaceable.width - markerPadding.dp.roundToPx(), 0)
            body.place(0, 0)
        }
    }
}

@Composable
private fun CodeView(code: BlockKind.Code, style: Style, env: TextEnv, box: Modifier) {
    val render = LocalRender.current
    val spec = render.theme.elements.code
    val boxStyle = style over Style(bg = Paint.Solid(spec.bg), pad = spec.pad, radius = spec.radius)
    val codeEnv = env.copy(font = spec.font, size = spec.size, leading = spec.leading, color = color(spec.color), align = HAlign.Left).with(style).copy(align = HAlign.Left)
    val textStyle = render.textStyle(codeEnv)
    Column(box.then(Modifier.cssBox(boxStyle, render.theme, null)).clipToBounds()) {
        for (line in code.lines) {
            val text = buildAnnotatedString {
                for (token in line) {
                    val tokenStyle = spec.syntax[token.kind]
                    withStyle(SpanStyle(color = tokenStyle?.color?.let(::color) ?: codeEnv.color, fontStyle = if (tokenStyle?.italic == true) FontStyle.Italic else null)) { append(token.text) }
                }
            }
            Text(text, style = textStyle, softWrap = false, maxLines = 1, overflow = TextOverflow.Clip, inlineContent = emptyMap<String, InlineTextContent>())
        }
    }
}

@Composable
private fun ImageView(block: BlockKind.Image, style: Style, modifier: Modifier) {
    val render = LocalRender.current
    val bitmap by rememberImage(render.images, block.src)
    val image = bitmap ?: return
    Layout(
        content = {
            Image(image, block.alt, contentScale = if (block.fit == ImageFit.Cover) ContentScale.Crop else ContentScale.Fit)
        },
        modifier = modifier,
    ) { measurables, constraints ->
        val ratio = image.height / image.width.coerceAtLeast(1).toFloat()
        val naturalWidth = image.width.dp.roundToPx()
        val explicitWidth = style.w.takeUnless { it == null || it is Length.Auto } != null
        val explicitHeight = style.h.takeUnless { it == null || it is Length.Auto } != null
        val width: Int
        val height: Int
        when {
            explicitWidth && explicitHeight -> { width = constraints.maxWidth; height = constraints.maxHeight }
            explicitWidth -> { width = constraints.maxWidth; height = (width * ratio).roundToInt() }
            explicitHeight -> { height = constraints.maxHeight; width = minOf((height / ratio).roundToInt(), constraints.maxWidth) }
            else -> { width = minOf(naturalWidth, constraints.maxWidth); height = (width * ratio).roundToInt() }
        }
        val placeable = measurables[0].measure(Constraints.fixed(width.coerceAtLeast(0), height.coerceAtLeast(0)))
        layout(placeable.width, placeable.height) { placeable.place(0, 0) }
    }
}
