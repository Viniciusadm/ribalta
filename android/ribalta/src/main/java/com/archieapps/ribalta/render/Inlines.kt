package com.archieapps.ribalta.render

import androidx.compose.foundation.Image
import com.archieapps.ribalta.ir.Style
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.draw.alpha
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.foundation.text.appendInlineContent
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.Placeholder
import androidx.compose.ui.text.PlaceholderVerticalAlign
import androidx.compose.ui.text.PlatformTextStyle
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextMotion
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.archieapps.ribalta.ir.Action
import com.archieapps.ribalta.ir.HAlign
import com.archieapps.ribalta.ir.ImageSource
import com.archieapps.ribalta.ir.Inline
import com.archieapps.ribalta.runtime.Evaluator
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

fun bolder(weight: Int) = when {
    weight < 350 -> 400
    weight < 550 -> 700
    else -> 900
}

fun RenderContext.spanStyle(env: TextEnv, decoration: TextDecoration? = null, alpha: Float = 1f) = SpanStyle(
    color = env.color.copy(alpha = env.color.alpha * alpha),
    fontFamily = family(env.font),
    fontSize = env.size.sp,
    fontWeight = FontWeight(env.weight),
    fontStyle = if (env.italic) FontStyle.Italic else FontStyle.Normal,
    letterSpacing = env.trackingPx.sp,
    textDecoration = listOfNotNull(if (env.underline) TextDecoration.Underline else null, decoration).takeIf { it.isNotEmpty() }?.let { TextDecoration.combine(it) },
)

fun RenderContext.textStyle(env: TextEnv) = TextStyle(
    color = env.color,
    fontFamily = family(env.font),
    fontSize = env.size.sp,
    fontWeight = FontWeight(env.weight),
    fontStyle = if (env.italic) FontStyle.Italic else FontStyle.Normal,
    letterSpacing = env.trackingPx.sp,
    textDecoration = if (env.underline) TextDecoration.Underline else null,
    lineHeight = (env.size * env.leading).sp,
    lineHeightStyle = LineHeightStyle(LineHeightStyle.Alignment.Center, LineHeightStyle.Trim.None),
    platformStyle = PlatformTextStyle(includeFontPadding = false),
    textMotion = TextMotion.Animated,
    textAlign = when (env.align) {
        HAlign.Left -> androidx.compose.ui.text.style.TextAlign.Left
        HAlign.Center -> androidx.compose.ui.text.style.TextAlign.Center
        HAlign.Right -> androidx.compose.ui.text.style.TextAlign.Right
    },
)

private val plainLink = TextLinkStyles(SpanStyle())

class Inlined(val text: AnnotatedString, val content: Map<String, InlineTextContent>)

class InlineBuilder(
    private val render: RenderContext,
    private val interaction: Interaction,
    private val measurer: TextMeasurer,
    private val images: Map<ImageSource, androidx.compose.ui.graphics.ImageBitmap?>,
    private val density: Float,
) {
    private val content = mutableMapOf<String, InlineTextContent>()
    private var counter = 0

    fun build(nodes: List<Inline>, env: TextEnv): Inlined {
        val text = buildAnnotatedString { append(nodes, env, 1f) }
        return Inlined(collapse(text), content)
    }

    private fun collapse(text: AnnotatedString): AnnotatedString {
        val start = text.text.indexOfFirst { !it.isWhitespace() || it == '\n' }.coerceAtLeast(0)
        val end = text.text.indexOfLast { !it.isWhitespace() || it == '\n' } + 1
        return if (end <= start) AnnotatedString("") else text.subSequence(start, end)
    }

    private fun shown(value: String, env: TextEnv): String {
        val collapsed = value.replace(Regex("[ \\t\\r\\n]+"), " ")
        return if (env.uppercase) collapsed.uppercase(Locale.ROOT) else collapsed
    }

    private fun AnnotatedString.Builder.append(nodes: List<Inline>, env: TextEnv, alpha: Float) {
        for (node in nodes) append(node, env, alpha)
    }

    private fun AnnotatedString.Builder.append(node: Inline, env: TextEnv, alpha: Float) {
        when (node) {
            is Inline.Text -> withStyle(render.spanStyle(env, alpha = alpha)) { append(shown(node.value, env)) }
            Inline.Break -> append("\n")
            is Inline.Strong -> append(node.children, env.copy(weight = bolder(env.weight)), alpha)
            is Inline.Emphasis -> append(node.children, env.copy(italic = true), alpha)
            is Inline.Strike -> withStyle(SpanStyle(textDecoration = TextDecoration.LineThrough)) { append(node.children, env, alpha) }
            is Inline.Field -> withStyle(render.spanStyle(env, alpha = alpha)) { append(shown(today(render.english), env)) }
            is Inline.Value -> withStyle(render.spanStyle(env, alpha = alpha)) { append(shown(Evaluator.format(Evaluator.evaluate(node.expr, interaction.scope)), env)) }
            is Inline.Link -> {
                val link = render.theme.elements.link
                val linkEnv = env.copy(color = color(link.color), underline = true)
                val annotation = if (node.href.startsWith("#")) {
                    LinkAnnotation.Clickable(node.href) { interaction.run(listOf(Action.Go(node.href.removePrefix("#")))) }
                } else {
                    LinkAnnotation.Url(node.href)
                }
                withLink(annotation) { append(node.children, linkEnv, alpha) }
            }
            is Inline.Span -> {
                if (!interaction.holds(node.behavior.whenExpr)) return
                val style = interaction.withVariants(node.style, node.behavior.variants)
                val spanEnv = env.with(style)
                val visible = if (node.step != null && interaction.step < node.step) 0f else 1f
                val spanAlpha = alpha * visible * (style.opacity ?: 1f)
                if (style.pad != null || style.radius != null || style.border != null || (style.shadow != null && style.shadow != com.archieapps.ribalta.ir.ShadowSize.None)) {
                    boxed(node, style, env, spanEnv, spanAlpha)
                    return
                }
                val background = (style.bg as? com.archieapps.ribalta.ir.Paint.Solid)?.let { color(it.color) }
                val wrapped: AnnotatedString.Builder.() -> Unit = {
                    if (background != null) {
                        withStyle(SpanStyle(background = background.copy(alpha = background.alpha * spanAlpha))) { append(node.children, spanEnv, spanAlpha) }
                    } else {
                        append(node.children, spanEnv, spanAlpha)
                    }
                }
                if (node.behavior.click.isNotEmpty()) {
                    withLink(LinkAnnotation.Clickable("span${counter++}", hoverLink(node, env)) { interaction.run(node.behavior.click) }) { wrapped() }
                } else {
                    wrapped()
                }
            }
            is Inline.Code -> code(node.value, env, alpha)
            is Inline.Image -> image(node, env)
        }
    }

    private fun hoverLink(node: Inline.Span, env: TextEnv): TextLinkStyles {
        if (!node.behavior.hoverable) return plainLink
        val hovered = interaction.withVariants(node.style, node.behavior.variants, hovered = true)
        val background = (hovered.bg as? com.archieapps.ribalta.ir.Paint.Solid)?.let { color(it.color) }
        val hoveredStyle = SpanStyle(color = hovered.color?.let(::color) ?: androidx.compose.ui.graphics.Color.Unspecified, background = background ?: androidx.compose.ui.graphics.Color.Unspecified)
        return TextLinkStyles(SpanStyle(), focusedStyle = hoveredStyle, hoveredStyle = hoveredStyle)
    }

    private fun AnnotatedString.Builder.boxed(node: Inline.Span, style: Style, env: TextEnv, spanEnv: TextEnv, alpha: Float) {
        val inner = InlineBuilder(render, interaction, measurer, images, density).build(node.children, spanEnv)
        val textStyle = render.textStyle(spanEnv).copy(lineHeight = androidx.compose.ui.unit.TextUnit.Unspecified)
        val measured = measurer.measure(inner.text, textStyle, softWrap = false)
        val pad = style.pad ?: (0f to 0f)
        val border = style.border?.width ?: 0f
        val width = measured.size.width / density + (pad.second + border) * 2
        val height = measured.size.height / density + (pad.first + border) * 2
        val line = env.size * env.leading
        val id = "box${counter++}"
        val box = style.copy(w = null, h = null, x = null, y = null, opacity = null)
        content[id] = InlineTextContent(Placeholder(width.sp, minOf(height, line).sp, PlaceholderVerticalAlign.TextCenter)) {
            Box(Modifier.fillMaxSize().wrapContentSize(unbounded = true).alpha(alpha), contentAlignment = Alignment.Center) {
                val (source, hovered) = rememberHover(node.behavior)
                val shown = if (hovered) interaction.withVariants(node.style, node.behavior.variants, hovered = true).copy(w = null, h = null, x = null, y = null, opacity = null) else box
                val clickable = when {
                    node.behavior.click.isNotEmpty() -> Modifier.clickable(source, null) { interaction.run(node.behavior.click) }
                    node.behavior.hoverable -> Modifier.hoverable(source)
                    else -> Modifier
                }
                Box(Modifier.requiredSize(width.dp, height.dp).then(clickable).cssBox(shown, render.theme, null), contentAlignment = Alignment.CenterStart) {
                    Text(inner.text, style = textStyle, softWrap = false, maxLines = 1, inlineContent = inner.content)
                }
            }
        }
        appendInlineContent(id, node.children.joinToString("") { (it as? Inline.Text)?.value ?: " " })
    }

    private fun AnnotatedString.Builder.code(value: String, env: TextEnv, alpha: Float) {
        val spec = render.theme.elements.inlineCode
        val size = env.size * spec.scale
        val codeEnv = env.copy(font = spec.font, size = size, trackingPx = 0f)
        val style = render.textStyle(codeEnv).copy(color = env.color.copy(alpha = env.color.alpha * alpha), lineHeight = androidx.compose.ui.unit.TextUnit.Unspecified)
        val measured = measurer.measure(value, style)
        val padX = spec.padEm.second * size
        val padY = spec.padEm.first * size
        val width = measured.size.width / density + padX * 2
        val height = measured.size.height / density + padY * 2
        val id = "code${counter++}"
        val tint = env.color.copy(alpha = env.color.alpha * spec.tint * alpha)
        content[id] = InlineTextContent(Placeholder(width.sp, height.sp, PlaceholderVerticalAlign.TextCenter)) {
            Box(
                Modifier.fillMaxSize().background(tint, RoundedCornerShape((spec.radiusEm * size).dp)).padding(horizontal = padX.dp, vertical = padY.dp),
                contentAlignment = Alignment.CenterStart,
            ) {
                Text(value, style = style, softWrap = false, maxLines = 1)
            }
        }
        appendInlineContent(id, value)
    }

    private fun AnnotatedString.Builder.image(node: Inline.Image, env: TextEnv) {
        val bitmap = images[node.src]
        val height = env.size
        val width = bitmap?.let { height * it.width / it.height.coerceAtLeast(1) } ?: height
        val id = "image${counter++}"
        content[id] = InlineTextContent(Placeholder(width.sp, height.sp, PlaceholderVerticalAlign.TextCenter)) {
            if (bitmap != null) Image(bitmap, node.alt, Modifier.fillMaxSize(), contentScale = ContentScale.Fit)
        }
        appendInlineContent(id, node.alt)
    }

}

fun today(english: Boolean): String {
    val locale = if (english) Locale.US else Locale.forLanguageTag("pt-BR")
    return LocalDate.now().format(DateTimeFormatter.ofLocalizedDate(FormatStyle.LONG).withLocale(locale))
}

fun imageSources(nodes: List<Inline>): List<ImageSource> = nodes.flatMap { node ->
    when (node) {
        is Inline.Image -> listOf(node.src)
        is Inline.Strong -> imageSources(node.children)
        is Inline.Emphasis -> imageSources(node.children)
        is Inline.Strike -> imageSources(node.children)
        is Inline.Link -> imageSources(node.children)
        is Inline.Span -> imageSources(node.children)
        else -> emptyList()
    }
}
