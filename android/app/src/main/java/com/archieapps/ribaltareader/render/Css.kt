package com.archieapps.ribaltareader.render

import android.graphics.BlurMaskFilter
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredHeightIn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.archieapps.ribaltareader.ir.HAlign
import com.archieapps.ribaltareader.ir.Length
import com.archieapps.ribaltareader.ir.Paint
import com.archieapps.ribaltareader.ir.ShadowSize
import com.archieapps.ribaltareader.ir.Style
import com.archieapps.ribaltareader.ir.Theme
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.roundToInt
import kotlin.math.sin

fun color(value: Long) = Color(value.toInt())

data class TextEnv(
    val color: Color,
    val font: String?,
    val size: Float,
    val weight: Int,
    val italic: Boolean,
    val uppercase: Boolean,
    val underline: Boolean,
    val align: HAlign,
    val leading: Float,
    val trackingPx: Float,
) {
    fun with(style: Style): TextEnv {
        val size = style.size ?: size
        return TextEnv(
            color = style.color?.let(::color) ?: color,
            font = style.font ?: font,
            size = size,
            weight = style.weight ?: weight,
            italic = style.italic ?: italic,
            uppercase = style.uppercase ?: uppercase,
            underline = style.underline ?: underline,
            align = style.align ?: align,
            leading = style.leading ?: leading,
            trackingPx = style.tracking?.let { it * size } ?: trackingPx,
        )
    }

    companion object {
        val Initial = TextEnv(Color.Black, null, 16f, 400, false, false, false, HAlign.Left, 1.2f, 0f)
    }
}

class RenderContext(val theme: Theme, val fonts: Fonts, val icons: Map<String, List<com.archieapps.ribaltareader.ir.IconElement>>, val images: Images, val pageBackground: Color, val english: Boolean) {
    fun family(name: String?): FontFamily = fonts.family(name)
}

val LocalRender = compositionLocalOf<RenderContext> { error("RenderContext missing") }

fun Modifier.cssSize(style: Style, parentWidth: Float?): Modifier = layout { measurable, constraints ->
    fun resolve(length: Length?, max: Int, reference: Float?): Int? = when (length) {
        is Length.Px -> length.value.dp.roundToPx()
        is Length.Pct -> reference?.let { (it * length.value / 100f).dp.roundToPx() } ?: if (max != Constraints.Infinity) (max * length.value / 100f).roundToInt() else null
        else -> null
    }
    val width = resolve(style.w, constraints.maxWidth, parentWidth)
    val height = resolve(style.h, constraints.maxHeight, null)
    val inner = Constraints(
        minWidth = width ?: constraints.minWidth,
        maxWidth = width ?: constraints.maxWidth,
        minHeight = height ?: constraints.minHeight,
        maxHeight = height ?: constraints.maxHeight,
    )
    val placeable = measurable.measure(inner)
    val reportedWidth = placeable.width.coerceIn(constraints.minWidth, constraints.maxWidth)
    val reportedHeight = placeable.height.coerceIn(constraints.minHeight, constraints.maxHeight)
    layout(reportedWidth, reportedHeight) { placeable.place(0, 0) }
}

fun DrawScope.paintBrush(paint: Paint, size: Size): Brush = when (paint) {
    is Paint.Solid -> androidx.compose.ui.graphics.SolidColor(color(paint.color))
    is Paint.Linear -> {
        val radians = paint.angle * PI.toFloat() / 180f
        val dx = sin(radians)
        val dy = -cos(radians)
        val half = (abs(size.width * dx) + abs(size.height * dy)) / 2f
        val center = Offset(size.width / 2f, size.height / 2f)
        Brush.linearGradient(
            colorStops = paint.stops.map { it.at to color(it.color) }.toTypedArray(),
            start = center - Offset(dx * half, dy * half),
            end = center + Offset(dx * half, dy * half),
        )
    }
}

private fun roundRect(size: Size, radius: Float, inset: Float = 0f): Path {
    val limit = minOf(size.width, size.height) / 2f
    val r = (minOf(radius, limit) - inset).coerceAtLeast(0f)
    return Path().apply { addRoundRect(RoundRect(inset, inset, size.width - inset, size.height - inset, CornerRadius(r, r))) }
}

fun Modifier.cssBox(style: Style, theme: Theme, image: ImageBitmap?): Modifier {
    val radius = style.radius ?: 0f
    val shadow = style.shadow?.takeIf { it != ShadowSize.None }?.let { theme.elements.shadows[it] }
    val border = style.border
    var modifier: Modifier = this
    if (style.opacity != null) modifier = modifier.alpha(style.opacity)
    if (shadow != null || style.bg != null || image != null || border != null) {
        modifier = modifier.drawWithContent {
            val r = radius.dp.toPx()
            if (shadow != null) {
                val frameworkPaint = android.graphics.Paint().apply {
                    isAntiAlias = true
                    this.color = color(shadow.color).toArgb()
                    blurRadius(shadow.blur.dp.toPx())?.let { maskFilter = BlurMaskFilter(it, BlurMaskFilter.Blur.NORMAL) }
                }
                val left = shadow.x.dp.toPx()
                val top = shadow.y.dp.toPx()
                val limit = minOf(size.width, size.height) / 2f
                drawContext.canvas.nativeCanvas.drawRoundRect(left, top, left + size.width, top + size.height, minOf(r, limit), minOf(r, limit), frameworkPaint)
            }
            val clip = roundRect(size, r)
            clipPath(clip) {
                style.bg?.let { drawRect(paintBrush(it, size)) }
                if (image != null) drawCover(image)
            }
            drawContent()
            if (border != null && border.width > 0) {
                val w = border.width.dp.toPx()
                drawPath(roundRect(size, r, w / 2f), color(border.color), style = Stroke(w))
            }
        }
    }
    val pad = style.pad
    val inset = (border?.width ?: 0f)
    return modifier.padding(horizontal = ((pad?.second ?: 0f) + inset).dp, vertical = ((pad?.first ?: 0f) + inset).dp)
}

private fun blurRadius(cssBlur: Float): Float? {
    val sigma = cssBlur / 2f
    return ((sigma - 0.5f) / 0.57735f).takeIf { it > 0f }
}

fun DrawScope.drawCover(image: ImageBitmap) {
    val scale = maxOf(size.width / image.width, size.height / image.height)
    val width = image.width * scale
    val height = image.height * scale
    drawImage(
        image,
        dstOffset = IntOffset(((size.width - width) / 2f).roundToInt(), ((size.height - height) / 2f).roundToInt()),
        dstSize = IntSize(width.roundToInt(), height.roundToInt()),
    )
}

@Composable
fun MinHeight(height: Float, content: @Composable () -> Unit) {
    Box(Modifier.requiredHeightIn(min = height.dp)) { content() }
}

fun Modifier.drawRule(color: Color, thickness: Float, radius: Float): Modifier = drawBehind {
    val r = radius.dp.toPx()
    drawRoundRect(color, cornerRadius = CornerRadius(r, r))
}
