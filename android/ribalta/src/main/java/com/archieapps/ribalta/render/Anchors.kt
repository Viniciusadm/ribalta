package com.archieapps.ribalta.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.unit.dp

class AnchorRegistry {
    private val positions = mutableMapOf<String, LayoutCoordinates>()
    var highlighted by mutableStateOf<String?>(null)

    fun register(id: String, coordinates: LayoutCoordinates) {
        positions[id] = coordinates
    }

    fun unregister(id: String, coordinates: LayoutCoordinates?) {
        if (positions[id] === coordinates) positions.remove(id)
    }

    fun coordinates(id: String): LayoutCoordinates? = positions[id]?.takeIf { it.isAttached }
}

val LocalAnchors = compositionLocalOf<AnchorRegistry?> { null }

private val HighlightColor = Color(0x40FFC107)

@Composable
fun Modifier.anchored(id: String?, key: String? = id, highlight: Boolean = true): Modifier {
    val registry = LocalAnchors.current
    if (id == null || key == null || registry == null) return this
    DisposableEffect(registry, key) { onDispose { registry.unregister(key, registry.coordinates(key)) } }
    return this
        .onGloballyPositioned { registry.register(key, it) }
        .drawWithContent {
            drawContent()
            if (highlight && registry.highlighted == id) drawRoundRect(HighlightColor, cornerRadius = CornerRadius(8.dp.toPx()))
        }
}
