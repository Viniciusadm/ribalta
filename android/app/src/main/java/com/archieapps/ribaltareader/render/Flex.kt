package com.archieapps.ribaltareader.render

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.layoutId
import androidx.compose.ui.layout.SubcomposeLayout
import androidx.compose.ui.layout.Placeable
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import com.archieapps.ribaltareader.ir.HAlign
import com.archieapps.ribaltareader.ir.VAlign
import kotlin.math.max
import kotlin.math.roundToInt

private fun offset(free: Int, align: HAlign?) = when (align) {
    HAlign.Center -> free / 2
    HAlign.Right -> free
    else -> 0
}

private fun offset(free: Int, align: VAlign?) = when (align) {
    VAlign.Center -> free / 2
    VAlign.Bottom -> free
    else -> 0
}

object Shrinkable

@Composable
fun FlowColumn(
    gap: Float,
    align: HAlign?,
    valign: VAlign?,
    modifier: Modifier = Modifier,
    fillHeight: Boolean = false,
    content: @Composable () -> Unit,
) {
    Layout(content, modifier) { measurables, constraints ->
        val gapPx = gap.dp.roundToPx()
        val stretch = align == null && constraints.hasBoundedWidth
        val childConstraints = Constraints(minWidth = if (stretch) constraints.maxWidth else 0, maxWidth = constraints.maxWidth)
        val placeables = measurables.map { it.measure(childConstraints) }
        val gaps = gapPx * (placeables.size - 1).coerceAtLeast(0)
        val allocated = placeables.map { it.height }.toMutableList()
        if (fillHeight && constraints.hasBoundedHeight) {
            val shrinkable = measurables.indices.filter { measurables[it].layoutId == Shrinkable }
            val overflow = allocated.sum() + gaps - constraints.maxHeight
            val weight = shrinkable.sumOf { allocated[it] }
            if (overflow > 0 && weight > 0) {
                val cut = minOf(overflow, weight)
                for (index in shrinkable) allocated[index] -= (cut.toLong() * allocated[index] / weight).toInt()
            }
        }
        val contentHeight = allocated.sum() + gaps
        val width = if (stretch) constraints.maxWidth else (placeables.maxOfOrNull { it.width } ?: 0).coerceIn(constraints.minWidth, constraints.maxWidth)
        val height = if (fillHeight && constraints.hasBoundedHeight) constraints.maxHeight else contentHeight.coerceIn(constraints.minHeight, constraints.maxHeight)
        layout(width, height) {
            var y = offset(height - contentHeight, valign)
            placeables.forEachIndexed { index, placeable ->
                placeable.place(offset(width - placeable.width, align), y)
                y += allocated[index] + gapPx
            }
        }
    }
}

private class Line(val items: MutableList<Int> = mutableListOf())

@Composable
fun FlexRow(gap: Float, justify: HAlign?, valign: VAlign?, wrap: Boolean, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    Layout(content, modifier) { measurables, constraints ->
        val gapPx = gap.dp.roundToPx()
        val available = if (constraints.hasBoundedWidth) constraints.maxWidth else Int.MAX_VALUE
        val basis = measurables.map { it.maxIntrinsicWidth(Constraints.Infinity) }
        val minimum = measurables.map { it.minIntrinsicWidth(Constraints.Infinity) }
        val lines = mutableListOf(Line())
        var used = 0
        measurables.indices.forEach { index ->
            val line = lines.last()
            val extra = if (line.items.isEmpty()) basis[index] else gapPx + basis[index]
            if (wrap && line.items.isNotEmpty() && used + extra > available) {
                lines += Line(mutableListOf(index))
                used = basis[index]
            } else {
                line.items += index
                used += extra
            }
        }
        val widths = IntArray(measurables.size)
        for (line in lines) shrink(line.items, basis, minimum, available - gapPx * (line.items.size - 1).coerceAtLeast(0), widths)
        val placeables = arrayOfNulls<Placeable>(measurables.size)
        for (index in measurables.indices) placeables[index] = measurables[index].measure(Constraints.fixedWidth(widths[index]))
        val lineHeights = lines.map { line -> line.items.maxOfOrNull { placeables[it]!!.height } ?: 0 }
        val lineWidths = lines.map { line -> line.items.sumOf { widths[it] } + gapPx * (line.items.size - 1).coerceAtLeast(0) }
        val width = (lineWidths.maxOrNull() ?: 0).coerceIn(constraints.minWidth, constraints.maxWidth)
        val height = (lineHeights.sum() + gapPx * (lines.size - 1).coerceAtLeast(0)).coerceIn(constraints.minHeight, constraints.maxHeight)
        layout(max(width, 0), height) {
            var y = 0
            lines.forEachIndexed { lineIndex, line ->
                var x = offset(width - lineWidths[lineIndex], justify)
                for (index in line.items) {
                    val placeable = placeables[index]!!
                    placeable.place(x, y + offset(lineHeights[lineIndex] - placeable.height, valign ?: VAlign.Center))
                    x += widths[index] + gapPx
                }
                y += lineHeights[lineIndex] + gapPx
            }
        }
    }
}

private fun shrink(items: List<Int>, basis: List<Int>, minimum: List<Int>, space: Int, widths: IntArray) {
    items.forEach { widths[it] = basis[it] }
    var overflow = items.sumOf { widths[it] } - space
    val frozen = mutableSetOf<Int>()
    while (overflow > 0) {
        val flexible = items.filter { it !in frozen && widths[it] > minimum[it] }
        val weight = flexible.sumOf { basis[it] }.toFloat()
        if (flexible.isEmpty() || weight <= 0f) break
        var taken = 0
        for (index in flexible) {
            val share = (overflow * basis[index] / weight).roundToInt()
            val next = max(minimum[index], widths[index] - share)
            if (next == minimum[index]) frozen += index
            taken += widths[index] - next
            widths[index] = next
        }
        if (taken == 0) break
        overflow -= taken
    }
}

@Composable
fun ColumnsRow(gap: Float, weights: List<Float>, valign: VAlign?, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    Layout(content, modifier) { measurables, constraints ->
        val gapPx = gap.dp.roundToPx()
        val count = measurables.size
        val total = (constraints.maxWidth - gapPx * (count - 1).coerceAtLeast(0)).coerceAtLeast(0)
        val weightSum = (0 until count).sumOf { (weights.getOrNull(it) ?: 1f).toDouble() }.toFloat().takeIf { it > 0f } ?: 1f
        var assigned = 0
        val widths = (0 until count).map { index ->
            val width = if (index == count - 1) total - assigned else (total * (weights.getOrNull(index) ?: 1f) / weightSum).roundToInt()
            assigned += width
            width
        }
        val placeables = measurables.mapIndexed { index, measurable -> measurable.measure(Constraints.fixedWidth(widths[index])) }
        val height = (placeables.maxOfOrNull { it.height } ?: 0).coerceIn(constraints.minHeight, constraints.maxHeight)
        layout(constraints.maxWidth, height) {
            var x = 0
            placeables.forEachIndexed { index, placeable ->
                placeable.place(x, offset(height - placeable.height, valign ?: VAlign.Center))
                x += widths[index] + gapPx
            }
        }
    }
}

@Composable
fun GridLayout(columns: Int, gap: Float, valign: VAlign?, count: Int, modifier: Modifier = Modifier, item: @Composable (Int) -> Unit) {
    SubcomposeLayout(modifier) { constraints ->
        val gapPx = gap.dp.roundToPx()
        val cols = columns.coerceAtLeast(1)
        val columnWidth = ((constraints.maxWidth - gapPx * (cols - 1)) / cols.toFloat()).coerceAtLeast(0f)
        val width = columnWidth.roundToInt()
        val natural = (0 until count).map { index -> subcompose("natural$index") { item(index) }.map { it.measure(Constraints.fixedWidth(width)) } }
        val rows = (0 until count).chunked(cols)
        val rowHeights = rows.map { row -> row.maxOf { index -> natural[index].maxOfOrNull { it.height } ?: 0 } }
        val placeables = if (valign == null) {
            (0 until count).map { index ->
                val rowHeight = rowHeights[index / cols]
                subcompose("stretched$index") { item(index) }.map { it.measure(Constraints.fixed(width, rowHeight)) }
            }
        } else {
            natural
        }
        val height = (rowHeights.sum() + gapPx * (rows.size - 1).coerceAtLeast(0)).coerceIn(constraints.minHeight, constraints.maxHeight)
        layout(constraints.maxWidth, height) {
            var y = 0
            rows.forEachIndexed { rowIndex, row ->
                row.forEachIndexed { column, index ->
                    for (placeable in placeables[index]) {
                        placeable.place((column * (columnWidth + gapPx)).roundToInt(), y + offset(rowHeights[rowIndex] - placeable.height, valign))
                    }
                }
                y += rowHeights[rowIndex] + gapPx
            }
        }
    }
}
