package com.archieapps.ribaltareader.render

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import com.archieapps.ribaltareader.ir.Block
import com.archieapps.ribaltareader.ir.BlockKind
import com.archieapps.ribaltareader.ir.Document
import com.archieapps.ribaltareader.ir.Length
import com.archieapps.ribaltareader.ir.Paint
import com.archieapps.ribaltareader.ir.Page
import com.archieapps.ribaltareader.ir.Theme

private data class Positioned(val block: Block, val env: TextEnv)

private fun positioned(blocks: List<Block>, env: TextEnv, interaction: Interaction, theme: Theme, out: MutableList<Positioned>) {
    for (block in blocks) {
        if (!interaction.holds(block.behavior.whenExpr)) continue
        val style = interaction.withVariants(block.style, block.behavior.variants)
        if (style.positioned) {
            out += Positioned(block, env)
            continue
        }
        val childEnv = env.with(style)
        when (val kind = block.kind) {
            is BlockKind.Layout -> positioned(kind.children, childEnv, interaction, theme, out)
            is BlockKind.Quote -> positioned(kind.children, childEnv, interaction, theme, out)
            is BlockKind.BulletList -> kind.items.forEach { item -> positioned(item.children, childEnv.with(interaction.withVariants(item.style, item.behavior.variants)), interaction, theme, out) }
            else -> Unit
        }
    }
}

fun slideBackground(document: Document, slide: Page): Color {
    val merged = slide.style over document.theme.page
    val solid = (merged.bg as? Paint.Solid)?.color
    return color(solid ?: document.theme.colors["bg"] ?: 0xffffffff)
}

@Composable
fun PageView(document: Document, slide: Page, render: RenderContext) {
    val theme = document.theme
    val interaction = LocalInteraction.current
    val merged = slide.style over theme.page
    val env = TextEnv.Initial.with(theme.text).with(merged)
    val pad = merged.pad ?: (0f to 0f)
    val gap = merged.gap ?: 0f
    val image by rememberImage(render.images, merged.bgImage)
    val base = (theme.page.bg as? Paint.Solid)?.color ?: theme.colors["bg"]
    val ownBg = slide.style.bg
    val banded = slide.header != null || slide.footer != null
    val band = theme.layout.band
    val bandHeight = maxOf(0f, pad.first * band.heightScale - band.gap)
    val fixed = slide.height != null
    val area = if (fixed) Modifier.fillMaxSize() else Modifier.fillMaxWidth()
    val floating = mutableListOf<Positioned>().also { list ->
        listOfNotNull(slide.header, slide.footer).let { positioned(it, env, interaction, theme, list) }
        positioned(slide.children, env, interaction, theme, list)
        slide.overlay?.let { positioned(listOf(it), env, interaction, theme, list) }
    }
    CompositionLocalProvider(LocalRender provides render) {
        Box(
            (if (fixed) Modifier.size(slide.width.dp, slide.height!!.dp).clipToBounds() else Modifier.width(slide.width.dp)).drawBehind {
                if (ownBg is Paint.Solid && base != null && image == null) {
                    drawRect(color(base))
                    drawRect(color(ownBg.color))
                } else {
                    merged.bg?.let { drawRect(paintBrush(it, size)) }
                    image?.let { drawCover(it) }
                }
            },
        ) {
            if (banded) {
                Column(
                    area.padding(
                        start = pad.second.dp,
                        end = pad.second.dp,
                        top = (if (slide.header != null) pad.first * band.edgeScale else pad.first).dp,
                        bottom = (if (slide.footer != null) pad.first * band.edgeScale else pad.first).dp,
                    ),
                ) {
                    Band(slide.header, env, if (slide.header != null) bandHeight else 0f)
                    Box(Modifier.padding(vertical = band.gap.dp).let { if (fixed) it.weight(1f) else it }.fillMaxWidth()) {
                        FlowColumn(gap, merged.align, merged.valign, area, fillHeight = fixed) { Blocks(slide.children, env) }
                    }
                    Band(slide.footer, env, if (slide.footer != null) bandHeight else 0f)
                }
            } else {
                FlowColumn(gap, merged.align, merged.valign, area.padding(horizontal = pad.second.dp, vertical = pad.first.dp), fillHeight = fixed) {
                    Blocks(slide.children, env)
                }
            }
            slide.overlay?.let { overlay ->
                Box(Modifier.matchParentSize().padding(horizontal = pad.second.dp, vertical = pad.first.dp), propagateMinConstraints = true) { BlockView(overlay, env) }
            }
            for (item in floating) Floating(item, slide.width, slide.height ?: slide.width)
        }
    }
}

@Composable
private fun Band(block: Block?, env: TextEnv, minHeight: Float) {
    Box(Modifier.fillMaxWidth().heightIn(min = minHeight.dp), contentAlignment = Alignment.CenterStart) {
        if (block != null) FlowColumn(0f, null, null, Modifier.fillMaxWidth()) { BlockView(block, env) }
    }
}

@Composable
private fun Floating(item: Positioned, width: Float, height: Float) {
    val style = item.block.style
    fun resolve(length: Length?, reference: Float) = when (length) {
        is Length.Px -> length.value
        is Length.Pct -> reference * length.value / 100f
        else -> 0f
    }
    val x = resolve(style.x, width)
    val y = resolve(style.y, height)
    Layout(content = { BlockView(item.block, item.env, absolute = true) }, modifier = Modifier.fillMaxSize()) { measurables, constraints ->
        val left = x.dp.roundToPx()
        val top = y.dp.roundToPx()
        val placeables = measurables.map { it.measure(Constraints(maxWidth = (constraints.maxWidth - left).coerceAtLeast(0))) }
        layout(constraints.maxWidth, if (constraints.hasBoundedHeight) constraints.maxHeight else constraints.minHeight) { placeables.forEach { it.place(left, top) } }
    }
}

@Composable
fun ScaledPage(width: Float, height: Float?, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    BoxWithConstraints(modifier, contentAlignment = if (height == null) Alignment.TopCenter else Alignment.Center) {
        val outer = LocalDensity.current
        if (height == null) {
            val scale = with(outer) { maxWidth.toPx() / width }
            Box(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                CompositionLocalProvider(LocalDensity provides Density(scale, 1f)) {
                    Box(Modifier.width(width.dp)) { content() }
                }
            }
        } else {
            val scale = with(outer) { minOf(maxWidth.toPx() / width, maxHeight.toPx() / height) }
            CompositionLocalProvider(LocalDensity provides Density(scale, 1f)) {
                Box(Modifier.size(width.dp, height.dp)) { content() }
            }
        }
    }
}
