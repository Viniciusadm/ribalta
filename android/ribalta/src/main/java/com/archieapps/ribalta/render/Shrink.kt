package com.archieapps.ribalta.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.SubcomposeLayout

private const val SEARCH_STEPS = 10
private const val FINAL_SLOT = "final"
private const val TOLERANCE = 1

class OverflowProbe {
    var overflowed = false
        private set

    fun check(content: Int, available: Int) {
        if (content > available + TOLERANCE) overflowed = true
    }
}

val LocalOverflowProbe = staticCompositionLocalOf<OverflowProbe?> { null }

fun searchShrink(fits: (Float) -> Boolean, steps: Int = SEARCH_STEPS): Float {
    if (fits(1f)) return 1f
    if (!fits(0f)) return 0f
    var fitting = 0f
    var tooBig = 1f
    repeat(steps) {
        val middle = (fitting + tooBig) / 2f
        if (fits(middle)) fitting = middle else tooBig = middle
    }
    return fitting
}

@Composable
fun ShrinkToFit(enabled: Boolean, modifier: Modifier = Modifier, content: @Composable (Float) -> Unit) {
    SubcomposeLayout(modifier) { constraints ->
        val factor = if (enabled && constraints.hasBoundedHeight && constraints.hasBoundedWidth) {
            var slot = 0
            searchShrink({ candidate ->
                val probe = OverflowProbe()
                subcompose(slot++) { CompositionLocalProvider(LocalAnchors provides null, LocalOverflowProbe provides probe) { content(candidate) } }.forEach { it.measure(constraints) }
                !probe.overflowed
            })
        } else {
            1f
        }
        val placeables = subcompose(FINAL_SLOT) { content(factor) }.map { it.measure(constraints) }
        val width = placeables.maxOfOrNull { it.width }?.coerceIn(constraints.minWidth, constraints.maxWidth) ?: constraints.minWidth
        val height = placeables.maxOfOrNull { it.height }?.coerceIn(constraints.minHeight, constraints.maxHeight) ?: constraints.minHeight
        layout(width, height) { placeables.forEach { it.place(0, 0) } }
    }
}
