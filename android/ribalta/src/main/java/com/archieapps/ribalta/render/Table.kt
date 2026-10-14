package com.archieapps.ribalta.render

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import com.archieapps.ribalta.ir.BlockKind
import com.archieapps.ribalta.ir.ColumnAlign
import com.archieapps.ribalta.ir.HAlign
import kotlin.math.roundToInt

private class Rule(val y: Float, val width: Float)

private class TableGeometry {
    var rules: List<Rule> = emptyList()
}

@Composable
fun TableView(table: BlockKind.Table, env: TextEnv, modifier: Modifier = Modifier) {
    val render = LocalRender.current
    val spec = render.theme.elements.table
    val tableEnv = env.copy(size = spec.size, trackingPx = env.trackingPx)
    val columns = maxOf(table.header.size, table.rows.maxOfOrNull { it.size } ?: 0)
    val hasHeader = table.header.isNotEmpty()
    val rows = (if (hasHeader) listOf(table.header) else emptyList()) + table.rows
    val geometry = remember { TableGeometry() }
    val ruleColor = color(spec.ruleColor)
    Layout(
        content = {
            rows.forEachIndexed { rowIndex, row ->
                for (column in 0 until columns) {
                    val header = hasHeader && rowIndex == 0
                    val align = when (table.aligns.getOrNull(column)) {
                        ColumnAlign.Center -> HAlign.Center
                        ColumnAlign.Right -> HAlign.Right
                        else -> HAlign.Left
                    }
                    val cellEnv = tableEnv.copy(align = align, weight = if (header) spec.headerWeight else tableEnv.weight)
                    Box(Modifier.padding(horizontal = spec.cellPad.second.dp, vertical = spec.cellPad.first.dp)) {
                        Paragraph(row.getOrNull(column) ?: emptyList(), cellEnv, fill = true)
                    }
                }
            }
        },
        modifier = modifier.drawBehind {
            for (rule in geometry.rules) {
                val width = rule.width.dp.toPx()
                drawRect(ruleColor, Offset(0f, rule.y - width / 2f), Size(size.width, width))
            }
        },
    ) { measurables, constraints ->
        val cells = measurables.chunked(columns.coerceAtLeast(1))
        val maxWidths = IntArray(columns) { column -> cells.maxOfOrNull { it[column].maxIntrinsicWidth(Constraints.Infinity) } ?: 0 }
        val minWidths = IntArray(columns) { column -> cells.maxOfOrNull { it[column].minIntrinsicWidth(Constraints.Infinity) } ?: 0 }
        val natural = maxWidths.sum()
        val widths = when {
            constraints.hasBoundedWidth && natural > constraints.maxWidth -> {
                val minimum = minWidths.sum()
                val room = (constraints.maxWidth - minimum).coerceAtLeast(0)
                val flexible = (natural - minimum).coerceAtLeast(1)
                IntArray(columns) { minWidths[it] + ((maxWidths[it] - minWidths[it]) * room / flexible.toFloat()).roundToInt() }
            }
            natural < constraints.minWidth && natural > 0 -> IntArray(columns) { (maxWidths[it] * constraints.minWidth / natural.toFloat()).roundToInt() }
            else -> maxWidths
        }
        val placeables = cells.map { row -> row.mapIndexed { column, cell -> cell.measure(Constraints(minWidth = widths[column], maxWidth = widths[column])) } }
        val headerRule = spec.headerRuleWidth.dp.toPx()
        val bodyRule = spec.ruleWidth.dp.toPx()
        val bottoms = rows.indices.map { if (hasHeader && it == 0) headerRule else bodyRule }
        val tops = rows.indices.map { if (it == 0) 0f else bottoms[it - 1] }
        val contentHeights = placeables.map { row -> row.maxOfOrNull { it.height } ?: 0 }
        var y = 0f
        val starts = rows.indices.map { index -> y.also { y += contentHeights[index] + tops[index] / 2f + bottoms[index] / 2f } }
        val height = (y + (bottoms.lastOrNull() ?: 0f) / 2f).roundToInt()
        geometry.rules = rows.indices.map { Rule(starts[it] + tops[it] / 2f + contentHeights[it] + bottoms[it] / 2f, if (hasHeader && it == 0) spec.headerRuleWidth else spec.ruleWidth) }
        val width = widths.sum()
        layout(width, height) {
            placeables.forEachIndexed { rowIndex, row ->
                var x = 0
                row.forEachIndexed { column, placeable ->
                    val top = starts[rowIndex] + tops[rowIndex] / 2f + (contentHeights[rowIndex] - placeable.height) / 2f
                    placeable.place(x, top.roundToInt())
                    x += widths[column]
                }
            }
        }
    }
}
