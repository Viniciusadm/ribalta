package com.archieapps.ribalta.render

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.unit.dp
import androidx.compose.ui.graphics.vector.PathParser
import com.archieapps.ribalta.ir.IconElement

private fun numbers(value: String?) = value?.split(Regex("[\\s,]+"))?.mapNotNull { it.toFloatOrNull() } ?: emptyList()

private fun IconElement.number(name: String) = attrs[name]?.toFloatOrNull() ?: 0f

fun iconPaths(elements: List<IconElement>): List<Path> = elements.map { element ->
    val path = Path()
    run {
        when (element.tag) {
            "path" -> element.attrs["d"]?.let { data -> path.addPath(PathParser().parsePathString(data).toPath()) }
            "circle" -> element.number("r").let { r -> path.addOval(Rect(Offset(element.number("cx"), element.number("cy")), r)) }
            "ellipse" -> path.addOval(Rect(element.number("cx") - element.number("rx"), element.number("cy") - element.number("ry"), element.number("cx") + element.number("rx"), element.number("cy") + element.number("ry")))
            "rect" -> {
                val rx = element.attrs["rx"]?.toFloatOrNull() ?: element.attrs["ry"]?.toFloatOrNull() ?: 0f
                val x = element.number("x")
                val y = element.number("y")
                path.addRoundRect(RoundRect(x, y, x + element.number("width"), y + element.number("height"), CornerRadius(rx, rx)))
            }
            "line" -> {
                path.moveTo(element.number("x1"), element.number("y1"))
                path.lineTo(element.number("x2"), element.number("y2"))
            }
            "polyline", "polygon" -> {
                val points = numbers(element.attrs["points"]).chunked(2).filter { it.size == 2 }
                points.firstOrNull()?.let { path.moveTo(it[0], it[1]) }
                points.drop(1).forEach { path.lineTo(it[0], it[1]) }
                if (element.tag == "polygon") path.close()
            }
        }
    }
    path
}

@Composable
fun IconView(elements: List<IconElement>, size: Float, stroke: Float, color: Color, modifier: Modifier = Modifier) {
    val paths = remember(elements) { iconPaths(elements) }
    Canvas(modifier.size(size.dp)) {
        val factor = this.size.width / 24f
        scale(factor, factor, pivot = Offset.Zero) {
            for (path in paths) drawPath(path, color, style = Stroke(width = stroke, cap = StrokeCap.Round, join = StrokeJoin.Round))
        }
    }
}
