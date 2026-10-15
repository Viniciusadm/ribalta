package com.archieapps.ribalta.render

import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.IntrinsicMeasurable
import androidx.compose.ui.layout.IntrinsicMeasureScope
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.Measurable
import androidx.compose.ui.layout.MeasurePolicy
import androidx.compose.ui.layout.MeasureResult
import androidx.compose.ui.layout.MeasureScope
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import kotlin.math.roundToInt

private class CssTextPolicy(private val excess: Int, private val softWrap: Boolean, private val align: TextAlign?, private val probe: OverflowProbe?) : MeasurePolicy {
    override fun MeasureScope.measure(measurables: List<Measurable>, constraints: Constraints): MeasureResult {
        val text = measurables[0]
        val widest = if (softWrap && constraints.hasBoundedWidth) text.minIntrinsicWidth(Constraints.Infinity) else 0
        val inner = if (widest > constraints.maxWidth) constraints.copy(maxWidth = widest, minWidth = minOf(constraints.minWidth, widest)) else constraints.copy(minHeight = 0)
        val placeable = text.measure(inner)
        if (constraints.hasBoundedWidth) probe?.check(placeable.width, constraints.maxWidth)
        val width = placeable.width.coerceIn(constraints.minWidth, constraints.maxWidth)
        val height = (placeable.height - excess).coerceIn(constraints.minHeight, constraints.maxHeight)
        val overflow = placeable.width - width
        val x = when {
            overflow <= 0 -> 0
            align == TextAlign.Center -> -overflow / 2
            align == TextAlign.Right -> -overflow
            else -> 0
        }
        return layout(width, height) { placeable.place(x, -excess / 2) }
    }

    private fun unbroken(measurable: IntrinsicMeasurable, width: Int) = if (softWrap) maxOf(width, measurable.minIntrinsicWidth(Constraints.Infinity)) else width

    override fun IntrinsicMeasureScope.minIntrinsicHeight(measurables: List<IntrinsicMeasurable>, width: Int) = (measurables[0].minIntrinsicHeight(unbroken(measurables[0], width)) - excess).coerceAtLeast(0)

    override fun IntrinsicMeasureScope.maxIntrinsicHeight(measurables: List<IntrinsicMeasurable>, width: Int) = (measurables[0].maxIntrinsicHeight(unbroken(measurables[0], width)) - excess).coerceAtLeast(0)

    override fun IntrinsicMeasureScope.minIntrinsicWidth(measurables: List<IntrinsicMeasurable>, height: Int) = measurables[0].minIntrinsicWidth(height)

    override fun IntrinsicMeasureScope.maxIntrinsicWidth(measurables: List<IntrinsicMeasurable>, height: Int) = measurables[0].maxIntrinsicWidth(height)
}

@Composable
fun CssText(
    text: AnnotatedString,
    style: TextStyle,
    lineHeight: Float,
    modifier: Modifier = Modifier,
    inlineContent: Map<String, InlineTextContent> = emptyMap(),
    softWrap: Boolean = true,
    maxLines: Int = Int.MAX_VALUE,
) {
    val measurer = rememberTextMeasurer()
    val density = LocalDensity.current
    val excess = remember(style, lineHeight, density) {
        val natural = measurer.measure(AnnotatedString("Hg"), style).size.height
        (natural - lineHeight * density.density).roundToInt().coerceAtLeast(0)
    }
    val probe = LocalOverflowProbe.current
    val policy = remember(excess, softWrap, style.textAlign, probe) { CssTextPolicy(excess, softWrap, style.textAlign, probe) }
    Layout(
        content = { Text(text, style = style, inlineContent = inlineContent, softWrap = softWrap, maxLines = maxLines, overflow = TextOverflow.Visible) },
        modifier = modifier,
        measurePolicy = policy,
    )
}
