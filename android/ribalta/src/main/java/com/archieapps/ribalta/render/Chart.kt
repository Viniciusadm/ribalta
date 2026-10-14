package com.archieapps.ribalta.render

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.PlatformTextStyle
import androidx.compose.ui.text.style.TextMotion
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.archieapps.ribalta.ir.ChartScene
import com.archieapps.ribalta.ir.Ink
import com.archieapps.ribalta.ir.Length
import com.archieapps.ribalta.ir.Shape
import com.archieapps.ribalta.ir.TextAnchor

@Composable
fun ChartView(scene: ChartScene, fixedHeight: Length?, maxHeight: Float, env: TextEnv, render: RenderContext) {
    val measurer = rememberTextMeasurer()
    BoxWithConstraints(Modifier.fillMaxWidth()) {
        val width = maxWidth.value
        val height = when (fixedHeight) {
            is Length.Px -> fixedHeight.value
            else -> minOf(width * scene.height / scene.width, maxHeight)
        }
        Canvas(Modifier.fillMaxWidth().height(height.dp)) {
            val scale = minOf(size.width / scene.width, size.height / scene.height)
            val left = (size.width - scene.width * scale) / 2f
            val top = (size.height - scene.height * scale) / 2f
            withTransform({
                translate(left, top)
                scale(scale, scale, pivot = Offset.Zero)
            }) {
                for (shape in scene.shapes) drawShape(shape, env, render, measurer)
            }
        }
    }
}

private fun ink(ink: Ink, env: TextEnv, render: RenderContext): Color = when (ink) {
    Ink.Text -> env.color
    Ink.Background -> render.pageBackground
    is Ink.Color -> color(ink.color)
}

private fun DrawScope.drawShape(shape: Shape, env: TextEnv, render: RenderContext, measurer: TextMeasurer) {
    when (shape) {
        is Shape.Rect -> drawRoundRect(ink(shape.ink, env, render), Offset(shape.x, shape.y), Size(shape.w, shape.h), CornerRadius(minOf(shape.radius, shape.w / 2f, shape.h / 2f)), alpha = shape.opacity)
        is Shape.Line -> drawLine(ink(shape.ink, env, render), Offset(shape.x1, shape.y1), Offset(shape.x2, shape.y2), shape.width, alpha = shape.opacity)
        is Shape.Polyline -> drawPath(polyline(shape.points, false), ink(shape.ink, env, render), shape.opacity, Stroke(shape.width, cap = StrokeCap.Round, join = StrokeJoin.Round))
        is Shape.Polygon -> drawPath(polyline(shape.points, true), ink(shape.ink, env, render), shape.opacity)
        is Shape.Circle -> drawCircle(ink(shape.ink, env, render), shape.r, Offset(shape.cx, shape.cy), shape.opacity)
        is Shape.Sector -> {
            val path = sector(shape)
            drawPath(path, ink(shape.ink, env, render))
            drawPath(path, ink(shape.stroke, env, render), style = Stroke(shape.strokeWidth))
        }
        is Shape.Text -> {
            val style = TextStyle(
                color = ink(shape.ink, env, render).let { it.copy(alpha = it.alpha * shape.opacity) },
                fontFamily = render.family(env.font),
                fontSize = shape.size.sp,
                fontWeight = FontWeight(shape.weight),
                platformStyle = PlatformTextStyle(includeFontPadding = false),
                textMotion = TextMotion.Animated,
            )
            val layout = measurer.measure(shape.text, style, density = Density(1f))
            val x = when (shape.anchor) {
                TextAnchor.Start -> shape.x
                TextAnchor.Middle -> shape.x - layout.size.width / 2f
                TextAnchor.End -> shape.x - layout.size.width
            }
            val y = if (shape.middle) shape.y - layout.size.height / 2f else shape.y - layout.firstBaseline
            translate(x, y) { drawText(layout) }
        }
    }
}

private fun polyline(points: List<Pair<Float, Float>>, closed: Boolean) = Path().apply {
    points.firstOrNull()?.let { moveTo(it.first, it.second) }
    points.drop(1).forEach { lineTo(it.first, it.second) }
    if (closed) close()
}

private fun sector(shape: Shape.Sector): Path {
    val full = shape.end - shape.start >= 360f - 1e-4f
    val sweep = if (full) 359.99f else shape.end - shape.start
    val outer = Rect(Offset(shape.cx, shape.cy), shape.outer)
    return Path().apply {
        if (shape.inner <= 0f) {
            moveTo(shape.cx, shape.cy)
            arcTo(outer, shape.start, sweep, false)
            close()
        } else {
            val inner = Rect(Offset(shape.cx, shape.cy), shape.inner)
            arcTo(outer, shape.start, sweep, true)
            arcTo(inner, shape.start + sweep, -sweep, false)
            close()
        }
    }
}
